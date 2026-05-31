// FFI shim implementation for rapidjson allocators.

#include "allocator_ffi.h"

#include <new>
#include <cstdlib>

#include "rapidjson/allocators.h"

using namespace rapidjson;

// ── CrtAllocator ─────────────────────────────────────────────────────────────

struct RapidJsonCrtAllocHandle {
    CrtAllocator alloc;
};

RapidJsonCrtAllocHandle* rapidjson_crtallocator_new(void) {
    return new (std::nothrow) RapidJsonCrtAllocHandle();
}
void rapidjson_crtallocator_free(RapidJsonCrtAllocHandle* h) { delete h; }

void* rapidjson_crtallocator_malloc(RapidJsonCrtAllocHandle* h, size_t size) {
    return h->alloc.Malloc(size);
}
void* rapidjson_crtallocator_realloc(RapidJsonCrtAllocHandle* h, void* ptr,
                                      size_t orig_size, size_t new_size) {
    return h->alloc.Realloc(ptr, orig_size, new_size);
}
void rapidjson_crtallocator_free_ptr(void* ptr) {
    CrtAllocator::Free(ptr);
}
int rapidjson_crtallocator_needs_free(void) {
    return CrtAllocator::kNeedFree ? 1 : 0;
}

// ── MemoryPoolAllocator ───────────────────────────────────────────────────────

struct RapidJsonMpAllocHandle {
    MemoryPoolAllocator<> alloc;
    RapidJsonMpAllocHandle(size_t cap) : alloc(cap) {}
};

RapidJsonMpAllocHandle* rapidjson_mpallocator_new(size_t cap) {
    return new (std::nothrow) RapidJsonMpAllocHandle(cap == 0 ? 65536 : cap);
}
void rapidjson_mpallocator_free(RapidJsonMpAllocHandle* h) { delete h; }

void* rapidjson_mpallocator_malloc(RapidJsonMpAllocHandle* h, size_t size) {
    return h->alloc.Malloc(size);
}
void* rapidjson_mpallocator_realloc(RapidJsonMpAllocHandle* h, void* ptr,
                                     size_t orig_size, size_t new_size) {
    return h->alloc.Realloc(ptr, orig_size, new_size);
}
void rapidjson_mpallocator_free_ptr(RapidJsonMpAllocHandle* h, void* ptr) {
    h->alloc.Free(ptr);
}
void rapidjson_mpallocator_clear(RapidJsonMpAllocHandle* h) {
    h->alloc.Clear();
}
size_t rapidjson_mpallocator_capacity(const RapidJsonMpAllocHandle* h) {
    return h->alloc.Capacity();
}
size_t rapidjson_mpallocator_size(const RapidJsonMpAllocHandle* h) {
    return h->alloc.Size();
}
int rapidjson_mpallocator_needs_free(void) {
    return MemoryPoolAllocator<>::kNeedFree ? 1 : 0;
}
