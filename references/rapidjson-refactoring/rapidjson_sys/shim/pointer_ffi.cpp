// FFI shim implementation for rapidjson Pointer.

#include "pointer_ffi.h"
#include "internal_handles.h"

#include <new>
#include <cstring>

#include "rapidjson/document.h"
#include "rapidjson/pointer.h"

using namespace rapidjson;

// RapidJsonDocumentHandle is defined in internal_handles.h.
typedef GenericPointer<GenericValue<UTF8<>, CrtAllocator>, CrtAllocator> CrtPointer;

struct RapidJsonPointerHandle {
    CrtPointer ptr;
    RapidJsonPointerHandle(const char* s) : ptr(s) {}
};

RapidJsonPointerHandle* rapidjson_pointer_new(const char* s) {
    return new (std::nothrow) RapidJsonPointerHandle(s);
}
void rapidjson_pointer_free(RapidJsonPointerHandle* h) { delete h; }

int rapidjson_pointer_is_valid(const RapidJsonPointerHandle* h) {
    return h->ptr.IsValid() ? 1 : 0;
}
int rapidjson_pointer_get_parse_error_code(const RapidJsonPointerHandle* h) {
    return (int)h->ptr.GetParseErrorCode();
}
size_t rapidjson_pointer_get_parse_error_offset(const RapidJsonPointerHandle* h) {
    return (size_t)h->ptr.GetParseErrorOffset();
}

// ── Get helpers ───────────────────────────────────────────────────────────────

int rapidjson_pointer_get_int(const RapidJsonPointerHandle* h,
                               const RapidJsonDocumentHandle* d,
                               int* out) {
    const CrtDocument::ValueType* v = h->ptr.Get(d->doc);
    if (!v || !v->IsInt()) return 0;
    *out = v->GetInt();
    return 1;
}
int rapidjson_pointer_get_string(const RapidJsonPointerHandle* h,
                                  const RapidJsonDocumentHandle* d,
                                  const char** out) {
    const CrtDocument::ValueType* v = h->ptr.Get(d->doc);
    if (!v || !v->IsString()) return 0;
    *out = v->GetString();
    return 1;
}
int rapidjson_pointer_get_bool(const RapidJsonPointerHandle* h,
                                const RapidJsonDocumentHandle* d,
                                int* out) {
    const CrtDocument::ValueType* v = h->ptr.Get(d->doc);
    if (!v || !v->IsBool()) return 0;
    *out = v->GetBool() ? 1 : 0;
    return 1;
}
int rapidjson_pointer_get_double(const RapidJsonPointerHandle* h,
                                  const RapidJsonDocumentHandle* d,
                                  double* out) {
    const CrtDocument::ValueType* v = h->ptr.Get(d->doc);
    if (!v || !v->IsDouble()) return 0;
    *out = v->GetDouble();
    return 1;
}

// ── GetWithDefault ────────────────────────────────────────────────────────────

int rapidjson_pointer_get_with_default_int(const RapidJsonPointerHandle* h,
                                            RapidJsonDocumentHandle* d,
                                            int def,
                                            int* out) {
    CrtDocument::AllocatorType& alloc = d->doc.GetAllocator();
    CrtDocument::ValueType& v = h->ptr.GetWithDefault(d->doc, def, alloc);
    if (!v.IsInt()) return 0;
    *out = v.GetInt();
    return 1;
}
int rapidjson_pointer_get_with_default_string(const RapidJsonPointerHandle* h,
                                               RapidJsonDocumentHandle* d,
                                               const char* def,
                                               const char** out) {
    CrtDocument::AllocatorType& alloc = d->doc.GetAllocator();
    CrtDocument::ValueType defaultVal(def, (SizeType)strlen(def), alloc);
    CrtDocument::ValueType& v = h->ptr.GetWithDefault(d->doc, defaultVal, alloc);
    if (!v.IsString()) return 0;
    *out = v.GetString();
    return 1;
}

// ── Set ───────────────────────────────────────────────────────────────────────

void rapidjson_pointer_set_int(const RapidJsonPointerHandle* h,
                                RapidJsonDocumentHandle* d,
                                int val) {
    h->ptr.Set(d->doc, val, d->doc.GetAllocator());
}
void rapidjson_pointer_set_string(const RapidJsonPointerHandle* h,
                                   RapidJsonDocumentHandle* d,
                                   const char* val) {
    h->ptr.Set(d->doc, StringRef(val), d->doc.GetAllocator());
}
void rapidjson_pointer_set_bool(const RapidJsonPointerHandle* h,
                                 RapidJsonDocumentHandle* d,
                                 int val) {
    h->ptr.Set(d->doc, val != 0, d->doc.GetAllocator());
}
void rapidjson_pointer_set_double(const RapidJsonPointerHandle* h,
                                   RapidJsonDocumentHandle* d,
                                   double val) {
    h->ptr.Set(d->doc, val, d->doc.GetAllocator());
}
void rapidjson_pointer_set_null(const RapidJsonPointerHandle* h,
                                 RapidJsonDocumentHandle* d) {
    CrtDocument::ValueType nullVal;
    h->ptr.Set(d->doc, nullVal, d->doc.GetAllocator());
}

// ── Create / Erase ────────────────────────────────────────────────────────────

int rapidjson_pointer_create(const RapidJsonPointerHandle* h,
                              RapidJsonDocumentHandle* d) {
    bool created = false;
    h->ptr.Create(d->doc, d->doc.GetAllocator(), &created);
    return created ? 1 : 0;
}
int rapidjson_pointer_erase(const RapidJsonPointerHandle* h,
                             RapidJsonDocumentHandle* d) {
    return h->ptr.Erase(d->doc) ? 1 : 0;
}
