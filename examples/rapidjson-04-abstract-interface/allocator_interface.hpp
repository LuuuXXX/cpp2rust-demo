// allocator_interface.hpp — rapidjson-04-abstract-interface
//
// Minimal allocator interface compatible with RapidJSON's allocator concept.
// RapidJSON's GenericDocument / GenericValue accept a custom allocator whose
// type must provide Malloc / Realloc / Free.
//
// This header defines a pure-virtual abstract base class representing that
// contract, demonstrating cpp2rust-demo's #[interface] + @make_proxy pipeline.

#pragma once
#include <cstddef>   // size_t

/// Minimal allocator interface compatible with RapidJSON's allocator concept.
///
/// When mapped by cpp2rust-demo this becomes:
///   - #[interface] class IAllocator { ... }   (in method/mtd_entry.rs)
///   - @make_proxy binding                      (in free/fn_entry.rs)
///
/// Rust code can then implement this interface:
///   struct MyAlloc;
///   impl IAllocator for MyAlloc { ... }
///   let proxy = new_i_allocator_proxy(MyAlloc);
class IAllocator {
public:
    virtual ~IAllocator() {}

    /// Allocate `size` bytes.  Returns nullptr on failure.
    virtual void* Malloc(size_t size) = 0;

    /// Reallocate memory.  `original_ptr` may be nullptr (acts as Malloc then).
    virtual void* Realloc(void* original_ptr,
                          size_t original_size,
                          size_t new_size) = 0;

    /// Free memory previously allocated by Malloc/Realloc.
    virtual void Free(void* ptr) = 0;

    /// Whether the allocator tracks allocated blocks (optional metadata).
    virtual bool CanFree() const = 0;
};
