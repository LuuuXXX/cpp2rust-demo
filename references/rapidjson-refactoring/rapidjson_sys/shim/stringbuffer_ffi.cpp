// FFI shim implementation for rapidjson GenericStringBuffer extended ops.

#include "stringbuffer_ffi.h"

#include <new>

#include "rapidjson/stringbuffer.h"

using namespace rapidjson;

// Note: RapidJsonStringBufferHandle is already defined in writer_ffi.cpp.
// Since these are separate translation units, we redeclare the struct here
// so this TU can use it independently.
struct RapidJsonStringBufferHandle {
    StringBuffer sb;
};

void rapidjson_stringbuffer_reserve(RapidJsonStringBufferHandle* h, size_t cap) {
    h->sb.Reserve(cap);
}
size_t rapidjson_stringbuffer_get_capacity(const RapidJsonStringBufferHandle* h) {
    // StringBuffer does not expose GetCapacity(); return current size as a proxy.
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

