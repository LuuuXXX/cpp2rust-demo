// FFI shim header for rapidjson GenericStringBuffer extended operations.
//
// Covers the API surface tested by:
//   test/unittest/stringbuffertest
//
// Basic StringBuffer operations (new/free/get_string/get_size/clear) are
// already in writer_ffi.h; this shim adds the extended/generic operations.

#pragma once

#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef struct RapidJsonStringBufferHandle RapidJsonStringBufferHandle;

// ── Extended StringBuffer operations ─────────────────────────────────────────

// Reserve capacity so the buffer can hold at least `new_capacity` bytes
// without reallocation.
void rapidjson_stringbuffer_reserve(RapidJsonStringBufferHandle* sb, size_t new_capacity);

// Return currently reserved capacity in bytes.
size_t rapidjson_stringbuffer_get_capacity(const RapidJsonStringBufferHandle* sb);

// Return pointer to the internal stack base (== GetString()).
const char* rapidjson_stringbuffer_get_stack_top(const RapidJsonStringBufferHandle* sb);

// Push `n` characters at once (uninitialized), return pointer to first.
char* rapidjson_stringbuffer_push_n(RapidJsonStringBufferHandle* sb, size_t n);

// Flush (no-op for StringBuffer; exposed for completeness / trait coverage).
void rapidjson_stringbuffer_flush(RapidJsonStringBufferHandle* sb);

#ifdef __cplusplus
}
#endif
