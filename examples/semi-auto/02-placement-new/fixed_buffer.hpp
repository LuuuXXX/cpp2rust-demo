#pragma once
// fixed_buffer.hpp — semi-auto/02-placement-new
//
// A class that owns a fixed-size buffer.
// Demonstrates the ⚙️ placement-new semi-auto workflow:
// cpp2rust-demo generates a commented-out @placement_new skeleton in
// entry.rs; the user uncomments it (or passes
// --enable-placement-new) to let Rust construct the object in
// Rust-managed aligned memory.

class FixedBuffer {
public:
    /// Construct a buffer with the given capacity (bytes).
    explicit FixedBuffer(int capacity);
    ~FixedBuffer();

    /// Write `size` bytes from `data` into the buffer.
    /// Returns the number of bytes actually written.
    int write(const char* data, int size);

    /// Return a pointer to the stored bytes.
    const char* data() const;

    /// Number of bytes currently stored.
    int size() const;

    /// Allocated capacity.
    int capacity() const;

    /// Reset the write cursor to the beginning.
    void reset();

private:
    char* buf_;
    int   capacity_;
    int   used_;
};
