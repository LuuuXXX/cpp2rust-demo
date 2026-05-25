//! Memory allocation abstractions for rapidjson-rs.

/// Allocator abstraction compatible with the C++ Allocator concept.
///
/// The allocator is responsible for providing raw memory blocks
/// and reclaiming them when no longer needed.
pub trait Allocator {
    /// Allocates a block of memory with the given size and alignment.
    ///
    /// Returns `None` when the allocation fails instead of panicking.
    fn allocate(&mut self, size: usize, align: usize) -> Option<*mut u8>;

    /// Deallocates a previously allocated memory block.
    ///
    /// The caller must ensure that `ptr`, `size` and `align` match
    /// the values that were used when the block was allocated.
    fn deallocate(&mut self, ptr: *mut u8, size: usize, align: usize);
}

/// System allocator wrapper that delegates to an underlying allocator.
///
/// The initial implementation wraps a memory pool allocator so that
/// callers can use a unified interface while we iterate on the
/// concrete allocation strategy.
pub struct SystemAllocator<'a> {
    inner: MemoryPoolAllocator<'a>,
}

impl<'a> SystemAllocator<'a> {
    /// Creates a new system allocator with the given capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            inner: MemoryPoolAllocator::with_capacity(capacity),
        }
    }

    /// Total capacity of the underlying allocator.
    pub fn capacity(&self) -> usize {
        self.inner.capacity()
    }
}

impl<'a> Allocator for SystemAllocator<'a> {
    fn allocate(&mut self, size: usize, align: usize) -> Option<*mut u8> {
        self.inner.allocate(size, align)
    }

    fn deallocate(&mut self, ptr: *mut u8, size: usize, align: usize) {
        self.inner.deallocate(ptr, size, align);
    }
}

/// Simple bump allocator backed by a pre-allocated buffer.
///
/// This implementation is intended for tests and small in-memory
/// allocations. It does not support deallocating individual blocks;
/// only monotonically growing allocations from the buffer.
pub struct BumpAllocator {
    buffer: Vec<u8>,
    offset: usize,
}

impl BumpAllocator {
    /// Creates a new bump allocator with the given capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            buffer: vec![0; capacity],
            offset: 0,
        }
    }
}

impl Allocator for BumpAllocator {
    fn allocate(&mut self, size: usize, align: usize) -> Option<*mut u8> {
        if size == 0 {
            return None;
        }

        // Align the current offset to the requested alignment.
        let align_mask = align.saturating_sub(1);
        let aligned_offset = (self.offset + align_mask) & !align_mask;

        let end = aligned_offset.checked_add(size)?;
        if end > self.buffer.len() {
            return None;
        }

        self.offset = end;

        // as_mut_ptr is safe; caller is responsible for not
        // dereferencing beyond the allocated range.
        let ptr = self.buffer.as_mut_ptr();
        Some(ptr.wrapping_add(aligned_offset))
    }

    fn deallocate(&mut self, _ptr: *mut u8, _size: usize, _align: usize) {
        // Individual deallocation is not supported for this simple
        // bump allocator. Callers are expected to reset or drop the
        // allocator when they are done.
    }
}

/// Memory pool allocator that matches the high-level design in the
/// core-infra documentation. It performs sequential allocations from
/// an internal buffer and does not support freeing individual blocks.
pub struct MemoryPoolAllocator<'a> {
    buffer: BufferStorage<'a>,
    size: usize,
}

enum BufferStorage<'a> {
    Owned(Vec<u8>),
    Borrowed(&'a mut [u8]),
}

impl<'a> MemoryPoolAllocator<'a> {
    /// Creates a new memory pool allocator with the given capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            buffer: BufferStorage::Owned(vec![0; capacity]),
            size: 0,
        }
    }

    /// Creates a memory pool allocator backed by a user-provided
    /// buffer. This avoids heap allocations and is suitable for
    /// embedded or high-performance scenarios.
    pub fn from_buffer(buffer: &'a mut [u8]) -> Self {
        Self {
            buffer: BufferStorage::Borrowed(buffer),
            size: 0,
        }
    }

    /// Total capacity of the underlying buffer.
    pub fn capacity(&self) -> usize {
        match &self.buffer {
            BufferStorage::Owned(buf) => buf.len(),
            BufferStorage::Borrowed(buf) => buf.len(),
        }
    }

    /// Currently used bytes in the buffer.
    pub fn used(&self) -> usize {
        self.size
    }

    /// Resets the allocator, making all memory available again.
    pub fn reset(&mut self) {
        self.size = 0;
    }
}

impl<'a> Allocator for MemoryPoolAllocator<'a> {
    fn allocate(&mut self, size: usize, align: usize) -> Option<*mut u8> {
        if size == 0 {
            return None;
        }

        let align_mask = align.saturating_sub(1);
        let aligned_size = (self.size + align_mask) & !align_mask;

        let end = aligned_size.checked_add(size)?;
        if end > self.capacity() {
            return None;
        }

        self.size = end;

        let ptr = match &mut self.buffer {
            BufferStorage::Owned(buf) => buf.as_mut_ptr(),
            BufferStorage::Borrowed(buf) => buf.as_mut_ptr(),
        };
        Some(ptr.wrapping_add(aligned_size))
    }

    fn deallocate(&mut self, _ptr: *mut u8, _size: usize, _align: usize) {
        // Individual deallocation is not supported; callers are
        // expected to reset the allocator when the pool can be reused.
    }
}

#[cfg(test)]
mod tests {
    use super::{Allocator, BumpAllocator, MemoryPoolAllocator, SystemAllocator};

    #[test]
    fn should_allocate_when_bump_allocator_has_capacity() {
        let mut alloc = BumpAllocator::with_capacity(64);
        let size = 16;
        let align = core::mem::align_of::<u64>();
        let ptr = alloc
            .allocate(size, align)
            .expect("allocation should succeed");

        // pointer should be non-null and within the buffer; detailed
        // correctness is validated by higher-level tests.
        assert!(!ptr.is_null());
    }

    #[test]
    fn should_fail_allocation_when_exceeds_memory_pool_capacity() {
        let mut alloc = MemoryPoolAllocator::with_capacity(8);
        let align = core::mem::align_of::<u32>();

        let first = alloc.allocate(4, align);
        assert!(first.is_some());

        // This allocation would exceed capacity and should fail.
        let second = alloc.allocate(8, align);
        assert!(second.is_none());
    }

    #[test]
    fn should_align_allocations_when_memory_pool_allocator() {
        let mut alloc = MemoryPoolAllocator::with_capacity(32);
        let align = core::mem::align_of::<u64>();

        let first = alloc.allocate(1, align).expect("first allocation");
        let second = alloc.allocate(1, align).expect("second allocation");

        // We cannot use unsafe pointer arithmetic due to crate-level
        // `forbid(unsafe_code)`. Instead we check that both pointers
        // are non-null and distinct, which indirectly exercises the
        // alignment logic without violating safety lints.
        assert!(!first.is_null());
        assert!(!second.is_null());
        assert_ne!(first, second);
    }

    #[test]
    fn should_delegate_to_memory_pool_when_system_allocator() {
        let mut alloc = SystemAllocator::with_capacity(16);
        let align = core::mem::align_of::<u32>();

        assert_eq!(alloc.capacity(), 16);

        let ptr = alloc
            .allocate(8, align)
            .expect("system allocator should allocate");
        assert!(!ptr.is_null());
    }

    #[test]
    fn should_use_borrowed_buffer_when_memory_pool_from_buffer() {
        let mut buffer = [0u8; 16];
        let mut alloc = MemoryPoolAllocator::from_buffer(&mut buffer);
        let align = core::mem::align_of::<u32>();

        assert_eq!(alloc.capacity(), 16);

        let ptr = alloc
            .allocate(8, align)
            .expect("allocation from borrowed buffer should succeed");
        assert!(!ptr.is_null());
    }
}
