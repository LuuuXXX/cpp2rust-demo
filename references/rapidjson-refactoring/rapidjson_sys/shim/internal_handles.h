// Internal shared handle definitions for rapidjson FFI shims.
//
// This header is NOT part of the public C ABI; it is only included by the
// .cpp implementation files to ensure identical struct layouts across
// translation units that share the same handle type.
//
// Rules:
//   - Do NOT include this from any public .h header.
//   - Do NOT add any #ifdef guard that would allow double-inclusion from a
//     public header; the pragma once guard is only for .cpp files.
#pragma once

#include "rapidjson/document.h"
#include "rapidjson/stringbuffer.h"

using namespace rapidjson;

// Shared document handle (used by document_ffi, pointer_ffi, schema_ffi).
typedef GenericDocument<UTF8<>, CrtAllocator> CrtDocument;

struct RapidJsonDocumentHandle {
    CrtDocument doc;
};

// Shared string-buffer handle (used by writer_ffi, stringbuffer_ffi).
struct RapidJsonStringBufferHandle {
    StringBuffer sb;
};

// Shared value handle (used by document_ffi, value_ffi).
typedef GenericValue<UTF8<>, CrtAllocator> CrtValue;

struct RapidJsonValueHandle {
    CrtValue val;
    CrtAllocator alloc;
    RapidJsonValueHandle() : val(), alloc() {}
};
