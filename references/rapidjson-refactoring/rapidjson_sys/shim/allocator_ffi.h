// FFI shim header for rapidjson allocators.
//
// Covers the API surface used by:
//   test/unittest/allocatorstest

#pragma once

#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

// ── CrtAllocator ─────────────────────────────────────────────────────────────

typedef struct RapidJsonCrtAllocHandle   RapidJsonCrtAllocHandle;

RapidJsonCrtAllocHandle* rapidjson_crtallocator_new(void);
void   rapidjson_crtallocator_free(RapidJsonCrtAllocHandle* a);
void*  rapidjson_crtallocator_malloc(RapidJsonCrtAllocHandle* a, size_t size);
void*  rapidjson_crtallocator_realloc(RapidJsonCrtAllocHandle* a, void* ptr,
                                       size_t orig_size, size_t new_size);
// Static free (CrtAllocator::Free is static)
void   rapidjson_crtallocator_free_ptr(void* ptr);
int    rapidjson_crtallocator_needs_free(void);   // returns 1 (CrtAllocator always needs free)

// ── MemoryPoolAllocator ───────────────────────────────────────────────────────

typedef struct RapidJsonMpAllocHandle    RapidJsonMpAllocHandle;

// Create with a fixed-size internal buffer (allocates buffer internally).
RapidJsonMpAllocHandle* rapidjson_mpallocator_new(size_t chunk_capacity);
void    rapidjson_mpallocator_free(RapidJsonMpAllocHandle* a);
void*   rapidjson_mpallocator_malloc(RapidJsonMpAllocHandle* a, size_t size);
void*   rapidjson_mpallocator_realloc(RapidJsonMpAllocHandle* a, void* ptr,
                                       size_t orig_size, size_t new_size);
void    rapidjson_mpallocator_free_ptr(RapidJsonMpAllocHandle* a, void* ptr);
void    rapidjson_mpallocator_clear(RapidJsonMpAllocHandle* a);
size_t  rapidjson_mpallocator_capacity(const RapidJsonMpAllocHandle* a);
size_t  rapidjson_mpallocator_size(const RapidJsonMpAllocHandle* a);
int     rapidjson_mpallocator_needs_free(void);   // returns 0 (pool allocator)

#ifdef __cplusplus
}
#endif
