//! Stream abstractions for rapidjson-rs.

use crate::error::Error;

/// Input stream abstraction similar to RapidJSON's Stream concept.
pub trait InputStream {
    /// Returns the current character without advancing the stream.
    fn peek(&self) -> Option<u8>;

    /// Returns the current character and advances the stream.
    fn take(&mut self) -> Option<u8>;

    /// Returns the number of characters that have been read so far.
    fn tell(&self) -> usize;
}

/// In-memory input stream backed by a byte slice.
pub struct SliceInputStream<'a> {
    buffer: &'a [u8],
    cursor: usize,
}

impl<'a> SliceInputStream<'a> {
    /// Creates a new input stream over the given byte slice.
    pub fn new(buffer: &'a [u8]) -> Self {
        Self { buffer, cursor: 0 }
    }
}

impl<'a> InputStream for SliceInputStream<'a> {
    fn peek(&self) -> Option<u8> {
        self.buffer.get(self.cursor).copied()
    }

    fn take(&mut self) -> Option<u8> {
        let ch = self.peek();
        if ch.is_some() {
            self.cursor += 1;
        }
        ch
    }

    fn tell(&self) -> usize {
        self.cursor
    }
}

/// Input stream backed by a UTF-8 string.
pub struct StringInputStream<'a> {
    inner: SliceInputStream<'a>,
}

impl<'a> StringInputStream<'a> {
    /// Creates a new input stream over the given string.
    pub fn new(input: &'a str) -> Self {
        Self {
            inner: SliceInputStream::new(input.as_bytes()),
        }
    }
}

impl<'a> InputStream for StringInputStream<'a> {
    fn peek(&self) -> Option<u8> {
        self.inner.peek()
    }

    fn take(&mut self) -> Option<u8> {
        self.inner.take()
    }

    fn tell(&self) -> usize {
        self.inner.tell()
    }
}

/// Output stream abstraction for writing bytes.
pub trait OutputStream {
    /// Writes a single byte to the stream.
    fn put(&mut self, ch: u8) -> Result<(), Error>;

    /// Flushes any buffered data to the underlying sink.
    fn flush(&mut self) -> Result<(), Error>;
}

/// In-memory output stream backed by a `Vec<u8>`.
pub struct MemoryOutputStream {
    buffer: Vec<u8>,
}

impl MemoryOutputStream {
    /// Creates a new, empty memory output stream.
    pub fn new() -> Self {
        Self { buffer: Vec::new() }
    }

    /// Returns the underlying buffer, consuming the stream.
    pub fn into_inner(self) -> Vec<u8> {
        self.buffer
    }
}

impl Default for MemoryOutputStream {
    fn default() -> Self {
        Self::new()
    }
}

impl OutputStream for MemoryOutputStream {
    fn put(&mut self, ch: u8) -> Result<(), Error> {
        self.buffer.push(ch);
        Ok(())
    }

    fn flush(&mut self) -> Result<(), Error> {
        Ok(())
    }
}

/// Input stream backed by a file.
pub struct FileInputStream {
    file: std::fs::File,
    position: u64,
    buffer: [u8; 1],
    buffered: bool,
}

impl FileInputStream {
    /// Opens a file at the given path for reading.
    pub fn open(path: &std::path::Path) -> Result<Self, Error> {
        let file = std::fs::File::open(path).map_err(|_| Error::Io)?;
        Ok(Self {
            file,
            position: 0,
            buffer: [0],
            buffered: false,
        })
    }
}

impl InputStream for FileInputStream {
    fn peek(&self) -> Option<u8> {
        if self.buffered {
            Some(self.buffer[0])
        } else {
            None
        }
    }

    fn take(&mut self) -> Option<u8> {
        use std::io::Read;
        if self.buffered {
            self.buffered = false;
            self.position += 1;
            return Some(self.buffer[0]);
        }

        match self.file.read(&mut self.buffer) {
            Ok(0) => None,
            Ok(_) => {
                self.position += 1;
                Some(self.buffer[0])
            }
            Err(_) => None,
        }
    }

    fn tell(&self) -> usize {
        self.position as usize
    }
}

/// Output stream backed by a file.
pub struct FileOutputStream {
    file: std::fs::File,
}

impl FileOutputStream {
    /// Opens a file at the given path for writing, truncating any
    /// existing contents.
    pub fn create(path: &std::path::Path) -> Result<Self, Error> {
        let file = std::fs::File::create(path).map_err(|_| Error::Io)?;
        Ok(Self { file })
    }
}

impl OutputStream for FileOutputStream {
    fn put(&mut self, ch: u8) -> Result<(), Error> {
        use std::io::Write;

        self.file.write_all(&[ch]).map_err(|_| Error::Io)
    }

    fn flush(&mut self) -> Result<(), Error> {
        use std::io::Write;

        self.file.flush().map_err(|_| Error::Io)
    }
}

#[cfg(test)]
mod tests {
    use super::{
        FileInputStream, FileOutputStream, InputStream, MemoryOutputStream, OutputStream,
        SliceInputStream, StringInputStream,
    };
    use std::io::Write as _;

    #[test]
    fn should_peek_without_advancing_when_slice_input_stream() {
        let data = b"abc";
        let stream = SliceInputStream::new(data);
        assert_eq!(stream.peek(), Some(b'a'));
        assert_eq!(stream.tell(), 0);
    }

    #[test]
    fn should_take_and_advance_when_slice_input_stream() {
        let data = b"abc";
        let mut stream = SliceInputStream::new(data);

        assert_eq!(stream.take(), Some(b'a'));
        assert_eq!(stream.tell(), 1);
        assert_eq!(stream.take(), Some(b'b'));
        assert_eq!(stream.tell(), 2);
        assert_eq!(stream.take(), Some(b'c'));
        assert_eq!(stream.tell(), 3);
        assert_eq!(stream.take(), None);
    }

    #[test]
    fn should_iterate_over_bytes_when_string_input_stream() {
        let input = "abc";
        let mut stream = StringInputStream::new(input);

        assert_eq!(stream.peek(), Some(b'a'));
        assert_eq!(stream.take(), Some(b'a'));
        assert_eq!(stream.take(), Some(b'b'));
        assert_eq!(stream.take(), Some(b'c'));
        assert_eq!(stream.take(), None);
    }

    #[test]
    fn should_collect_bytes_when_memory_output_stream() {
        let mut stream = MemoryOutputStream::new();

        stream.put(b'a').unwrap();
        stream.put(b'b').unwrap();
        stream.put(b'c').unwrap();
        stream.flush().unwrap();

        let data = stream.into_inner();
        assert_eq!(data, b"abc");
    }

    #[test]
    fn should_support_default_for_memory_output_stream() {
        let mut stream = MemoryOutputStream::default();
        stream.put(b'x').unwrap();
        let data = stream.into_inner();
        assert_eq!(data, b"x");
    }

    #[test]
    fn should_read_bytes_from_file_when_file_input_stream() {
        let dir = std::env::temp_dir();
        let path = dir.join("rapidjson_rs_file_input_stream_test.txt");

        {
            let mut file = std::fs::File::create(&path).expect("create file");
            file.write_all(b"xyz").expect("write file");
        }

        let mut stream = FileInputStream::open(&path).expect("open file");

        assert_eq!(stream.peek(), None); // nothing buffered yet
        assert_eq!(stream.take(), Some(b'x'));
        assert_eq!(stream.tell(), 1);
        assert_eq!(stream.take(), Some(b'y'));
        assert_eq!(stream.take(), Some(b'z'));
        assert_eq!(stream.take(), None);

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn should_write_bytes_to_file_when_file_output_stream() {
        let dir = std::env::temp_dir();
        let path = dir.join("rapidjson_rs_file_output_stream_test.txt");

        {
            let mut stream = FileOutputStream::create(&path).expect("create file");
            stream.put(b'a').unwrap();
            stream.put(b'b').unwrap();
            stream.put(b'c').unwrap();
            stream.flush().unwrap();
        }

        let content = std::fs::read(&path).expect("read file");
        assert_eq!(content, b"abc");

        let _ = std::fs::remove_file(&path);
    }
}
