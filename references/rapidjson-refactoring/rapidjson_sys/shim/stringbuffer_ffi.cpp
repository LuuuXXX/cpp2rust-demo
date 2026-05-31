// FFI shim implementation for rapidjson GenericStringBuffer extended ops.

#include "stringbuffer_ffi.h"
#include "internal_handles.h"

#include <new>

#include "rapidjson/stringbuffer.h"

using namespace rapidjson;

// RapidJsonStringBufferHandle is defined in internal_handles.h.

void rapidjson_stringbuffer_reserve(RapidJsonStringBufferHandle* h, size_t cap) {
    h->sb.Reserve(cap);
}
size_t rapidjson_stringbuffer_get_capacity(const RapidJsonStringBufferHandle* h) {
    // rapidjson StringBuffer does not expose a GetCapacity() method in this
    // version; return the current content size as an approximation.
    // Callers needing precise capacity should use rapidjson_stringbuffer_get_size.
    return h->sb.GetSize();
}
const char* rapidjson_stringbuffer_get_stack_top(const RapidJsonStringBufferHandle* h) {
    return h->sb.GetString();
}
char* rapidjson_stringbuffer_push_n(RapidJsonStringBufferHandle* h, size_t n) {
    return h->sb.Push(n);
}
void rapidjson_stringbuffer_flush(RapidJsonStringBufferHandle* h) {
    h->sb.Flush();
}

