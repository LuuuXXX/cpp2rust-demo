//! Internal byte-level stack abstraction with automatic growth.

/// A simple byte-oriented stack used by internal algorithms.
///
/// This mirrors the behaviour of the original C++ internal stack,
/// but exposes a safe API over a contiguous `Vec<u8>` buffer.
pub struct Stack {
    buffer: Vec<u8>,
}

impl Stack {
    /// Creates an empty stack.
    pub fn new() -> Self {
        Self { buffer: Vec::new() }
    }

    /// Clears the stack without releasing the underlying capacity.
    pub fn clear(&mut self) {
        self.buffer.clear();
    }

    /// Returns the number of bytes currently stored in the stack.
    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    /// Returns true if the stack contains no bytes.
    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }

    /// Ensures that the stack can hold at least `additional` more bytes
    /// without reallocating.
    pub fn reserve(&mut self, additional: usize) {
        self.buffer.reserve(additional);
    }

    /// Pushes raw bytes onto the stack.
    ///
    /// Returns the offset at which the bytes were inserted.
    pub fn push_bytes(&mut self, bytes: &[u8]) -> usize {
        let offset = self.buffer.len();
        self.buffer.extend_from_slice(bytes);
        offset
    }

    /// Pops `count` bytes from the top of the stack.
    pub fn pop_bytes(&mut self, count: usize) -> Option<()> {
        if count > self.buffer.len() {
            return None;
        }
        let new_len = self.buffer.len() - count;
        self.buffer.truncate(new_len);
        Some(())
    }

    /// Returns a slice to the last `count` bytes on the stack.
    pub fn top_bytes(&self, count: usize) -> Option<&[u8]> {
        if count > self.buffer.len() {
            return None;
        }
        let start = self.buffer.len() - count;
        Some(&self.buffer[start..])
    }

    /// Returns a reference to the entire underlying buffer.
    pub fn as_slice(&self) -> &[u8] {
        &self.buffer
    }
}

impl Default for Stack {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::Stack;

    #[test]
    fn should_push_and_pop_when_stack() {
        let mut stack = Stack::new();
        assert!(stack.is_empty());

        let offset = stack.push_bytes(&[1, 2, 3]);
        assert_eq!(offset, 0);
        assert_eq!(stack.len(), 3);
        assert_eq!(stack.top_bytes(1), Some(&[3][..]));

        stack.pop_bytes(3).expect("pop should succeed");
        assert!(stack.is_empty());
    }

    #[test]
    fn should_reserve_and_reuse_capacity_when_stack() {
        let mut stack = Stack::new();
        stack.reserve(16);

        let initial_capacity = stack.as_slice().len();
        assert_eq!(initial_capacity, 0);

        stack.push_bytes(&[1, 2, 3, 4]);
        assert_eq!(stack.len(), 4);

        stack.clear();
        assert!(stack.is_empty());

        stack.push_bytes(&[5, 6]);
        assert_eq!(stack.len(), 2);
        assert_eq!(stack.top_bytes(2), Some(&[5, 6][..]));
    }

    #[test]
    fn should_return_none_when_popping_more_than_len() {
        let mut stack = Stack::new();
        stack.push_bytes(&[1, 2]);
        assert!(stack.pop_bytes(3).is_none());
        assert_eq!(stack.len(), 2);
    }
}
