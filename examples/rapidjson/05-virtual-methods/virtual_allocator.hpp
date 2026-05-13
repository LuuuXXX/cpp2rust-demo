// virtual_allocator.hpp — rapidjson-05-virtual-methods
//
// A concrete allocator base class with non-pure virtual methods, modelling
// the pattern used by RapidJSON's CrtAllocator with extensible virtual dispatch.
// Demonstrates that cpp2rust-demo extracts non-pure virtual methods normally:
// no special handling is needed — hicc calls them through the vtable transparently.

#pragma once
#include <cstddef>  // size_t

/// Concrete allocator with virtual methods (all have default implementations).
///
/// cpp2rust-demo extracts virtual methods the same as regular methods.
/// hicc invokes them via the C++ vtable, so a Rust subclass (via @make_proxy
/// from a companion interface) can override them.
class BaseAllocator {
public:
    BaseAllocator() {}
    virtual ~BaseAllocator() {}

    /// Allocate size bytes.  Default implementation uses malloc.
    virtual void* Malloc(size_t size);

    /// Reallocate.  Default implementation uses realloc.
    virtual void* Realloc(void* original_ptr, size_t original_size, size_t new_size);

    /// Free memory.  Default implementation uses free.
    virtual void Free(void* ptr);

    /// Whether this allocator supports individual frees (always true here).
    virtual bool CanFree() const;

    // Non-virtual utility method.
    size_t align(size_t size, size_t alignment) const;

    // Static constant (goes to import_lib!)
    static const bool kNeedFree;
};
