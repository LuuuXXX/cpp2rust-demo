// FFI shim implementation for rapidjson Document / Value (DOM) API.
//
// Provides extern-C wrappers around rapidjson::Document and
// rapidjson::Value for use by the Rust rapidjson_sys crate.

#include "document_ffi.h"
#include "internal_handles.h"

#include <new>
#include <cstring>
#include <stdint.h>

#include "rapidjson/document.h"

RapidJsonDocumentHandle* rapidjson_document_new(void) {
    return new (std::nothrow) RapidJsonDocumentHandle();
}

void rapidjson_document_free(RapidJsonDocumentHandle* d) {
    delete d;
}

int rapidjson_document_parse(RapidJsonDocumentHandle* d, const char* json) {
    d->doc.Parse(json);
    return !d->doc.HasParseError();
}

int rapidjson_document_parse_insitu(RapidJsonDocumentHandle* d, char* json) {
    d->doc.ParseInsitu(json);
    return !d->doc.HasParseError();
}

int rapidjson_document_has_parse_error(const RapidJsonDocumentHandle* d) {
    return d->doc.HasParseError() ? 1 : 0;
}

int rapidjson_document_get_parse_error_code(const RapidJsonDocumentHandle* d) {
    return (int)d->doc.GetParseError();
}

size_t rapidjson_document_get_error_offset(const RapidJsonDocumentHandle* d) {
    return (size_t)d->doc.GetErrorOffset();
}

int rapidjson_document_is_object(const RapidJsonDocumentHandle* d) { return d->doc.IsObject(); }
int rapidjson_document_is_array(const RapidJsonDocumentHandle* d)  { return d->doc.IsArray(); }
int rapidjson_document_is_string(const RapidJsonDocumentHandle* d) { return d->doc.IsString(); }
int rapidjson_document_is_int(const RapidJsonDocumentHandle* d)    { return d->doc.IsInt(); }
int rapidjson_document_is_uint(const RapidJsonDocumentHandle* d)   { return d->doc.IsUint(); }
int rapidjson_document_is_int64(const RapidJsonDocumentHandle* d)  { return d->doc.IsInt64(); }
int rapidjson_document_is_uint64(const RapidJsonDocumentHandle* d) { return d->doc.IsUint64(); }
int rapidjson_document_is_double(const RapidJsonDocumentHandle* d) { return d->doc.IsDouble(); }
int rapidjson_document_is_bool(const RapidJsonDocumentHandle* d)   { return d->doc.IsBool(); }
int rapidjson_document_is_null(const RapidJsonDocumentHandle* d)   { return d->doc.IsNull(); }

const char* rapidjson_document_get_string(const RapidJsonDocumentHandle* d) {
    return d->doc.GetString();
}
size_t rapidjson_document_get_string_length(const RapidJsonDocumentHandle* d) {
    return d->doc.GetStringLength();
}
int rapidjson_document_get_int(const RapidJsonDocumentHandle* d)       { return d->doc.GetInt(); }
unsigned rapidjson_document_get_uint(const RapidJsonDocumentHandle* d)  { return d->doc.GetUint(); }
int64_t rapidjson_document_get_int64(const RapidJsonDocumentHandle* d)  { return d->doc.GetInt64(); }
uint64_t rapidjson_document_get_uint64(const RapidJsonDocumentHandle* d){ return d->doc.GetUint64(); }
double rapidjson_document_get_double(const RapidJsonDocumentHandle* d)  { return d->doc.GetDouble(); }
int rapidjson_document_get_bool(const RapidJsonDocumentHandle* d)       { return d->doc.GetBool(); }

size_t rapidjson_document_member_count(const RapidJsonDocumentHandle* d) {
    return d->doc.IsObject() ? d->doc.MemberCount() : 0;
}
int rapidjson_document_has_member(const RapidJsonDocumentHandle* d, const char* key) {
    return d->doc.IsObject() && d->doc.HasMember(key);
}
size_t rapidjson_document_size(const RapidJsonDocumentHandle* d) {
    return d->doc.IsArray() ? d->doc.Size() : 0;
}

// ── Value handle ─────────────────────────────────────────────────────────────
// RapidJsonValueHandle is defined in internal_handles.h.

RapidJsonValueHandle* rapidjson_value_new_null(void) {
    auto* h = new (std::nothrow) RapidJsonValueHandle();
    if (h) h->val.SetNull();
    return h;
}
RapidJsonValueHandle* rapidjson_value_new_bool(int b) {
    auto* h = new (std::nothrow) RapidJsonValueHandle();
    if (h) h->val.SetBool(b != 0);
    return h;
}
RapidJsonValueHandle* rapidjson_value_new_int(int v) {
    auto* h = new (std::nothrow) RapidJsonValueHandle();
    if (h) h->val.SetInt(v);
    return h;
}
RapidJsonValueHandle* rapidjson_value_new_uint(unsigned v) {
    auto* h = new (std::nothrow) RapidJsonValueHandle();
    if (h) h->val.SetUint(v);
    return h;
}
RapidJsonValueHandle* rapidjson_value_new_int64(int64_t v) {
    auto* h = new (std::nothrow) RapidJsonValueHandle();
    if (h) h->val.SetInt64(v);
    return h;
}
RapidJsonValueHandle* rapidjson_value_new_uint64(uint64_t v) {
    auto* h = new (std::nothrow) RapidJsonValueHandle();
    if (h) h->val.SetUint64(v);
    return h;
}
RapidJsonValueHandle* rapidjson_value_new_double(double v) {
    auto* h = new (std::nothrow) RapidJsonValueHandle();
    if (h) h->val.SetDouble(v);
    return h;
}
RapidJsonValueHandle* rapidjson_value_new_string(const char* s) {
    auto* h = new (std::nothrow) RapidJsonValueHandle();
    if (h) h->val.SetString(s, (SizeType)strlen(s), h->alloc);
    return h;
}
RapidJsonValueHandle* rapidjson_value_new_object(void) {
    auto* h = new (std::nothrow) RapidJsonValueHandle();
    if (h) h->val.SetObject();
    return h;
}
RapidJsonValueHandle* rapidjson_value_new_array(void) {
    auto* h = new (std::nothrow) RapidJsonValueHandle();
    if (h) h->val.SetArray();
    return h;
}
void rapidjson_value_free(RapidJsonValueHandle* h) { delete h; }

int rapidjson_value_is_null(const RapidJsonValueHandle* h)   { return h->val.IsNull(); }
int rapidjson_value_is_bool(const RapidJsonValueHandle* h)   { return h->val.IsBool(); }
int rapidjson_value_is_int(const RapidJsonValueHandle* h)    { return h->val.IsInt(); }
int rapidjson_value_is_uint(const RapidJsonValueHandle* h)   { return h->val.IsUint(); }
int rapidjson_value_is_int64(const RapidJsonValueHandle* h)  { return h->val.IsInt64(); }
int rapidjson_value_is_uint64(const RapidJsonValueHandle* h) { return h->val.IsUint64(); }
int rapidjson_value_is_double(const RapidJsonValueHandle* h) { return h->val.IsDouble(); }
int rapidjson_value_is_string(const RapidJsonValueHandle* h) { return h->val.IsString(); }
int rapidjson_value_is_object(const RapidJsonValueHandle* h) { return h->val.IsObject(); }
int rapidjson_value_is_array(const RapidJsonValueHandle* h)  { return h->val.IsArray(); }
int rapidjson_value_is_number(const RapidJsonValueHandle* h) { return h->val.IsNumber(); }

int rapidjson_value_get_bool(const RapidJsonValueHandle* h)          { return h->val.GetBool(); }
int rapidjson_value_get_int(const RapidJsonValueHandle* h)           { return h->val.GetInt(); }
unsigned rapidjson_value_get_uint(const RapidJsonValueHandle* h)     { return h->val.GetUint(); }
int64_t rapidjson_value_get_int64(const RapidJsonValueHandle* h)     { return h->val.GetInt64(); }
uint64_t rapidjson_value_get_uint64(const RapidJsonValueHandle* h)   { return h->val.GetUint64(); }
double rapidjson_value_get_double(const RapidJsonValueHandle* h)     { return h->val.GetDouble(); }
const char* rapidjson_value_get_string(const RapidJsonValueHandle* h){ return h->val.GetString(); }
size_t rapidjson_value_get_string_length(const RapidJsonValueHandle* h){ return h->val.GetStringLength(); }

void rapidjson_value_set_null(RapidJsonValueHandle* h)                    { h->val.SetNull(); }
void rapidjson_value_set_bool(RapidJsonValueHandle* h, int b)             { h->val.SetBool(b != 0); }
void rapidjson_value_set_int(RapidJsonValueHandle* h, int v)              { h->val.SetInt(v); }
void rapidjson_value_set_uint(RapidJsonValueHandle* h, unsigned v)        { h->val.SetUint(v); }
void rapidjson_value_set_int64(RapidJsonValueHandle* h, int64_t v)        { h->val.SetInt64(v); }
void rapidjson_value_set_uint64(RapidJsonValueHandle* h, uint64_t v)      { h->val.SetUint64(v); }
void rapidjson_value_set_double(RapidJsonValueHandle* h, double v)        { h->val.SetDouble(v); }
void rapidjson_value_set_string_copy(RapidJsonValueHandle* h, const char* s, size_t len) {
    h->val.SetString(s, (SizeType)len, h->alloc);
}

size_t rapidjson_value_member_count(const RapidJsonValueHandle* h) {
    return h->val.IsObject() ? h->val.MemberCount() : 0;
}
size_t rapidjson_value_size(const RapidJsonValueHandle* h) {
    return h->val.IsArray() ? h->val.Size() : 0;
}
int rapidjson_value_has_member(const RapidJsonValueHandle* h, const char* key) {
    return h->val.IsObject() && h->val.HasMember(key);
}
int rapidjson_value_empty(const RapidJsonValueHandle* h) {
    return h->val.IsArray() ? (h->val.Empty() ? 1 : 0)
         : h->val.IsObject() ? (h->val.ObjectEmpty() ? 1 : 0)
         : 1;
}

void rapidjson_value_copy_from(RapidJsonValueHandle* dst, const RapidJsonValueHandle* src) {
    dst->val.CopyFrom(src->val, dst->alloc);
}
void rapidjson_value_swap(RapidJsonValueHandle* a, RapidJsonValueHandle* b) {
    a->val.Swap(b->val);
}

int rapidjson_value_get_member_int(const RapidJsonValueHandle* h, const char* key, int* out) {
    if (!h->val.IsObject() || !h->val.HasMember(key)) return 0;
    const CrtValue& v = h->val[key];
    if (!v.IsInt()) return 0;
    *out = v.GetInt();
    return 1;
}
int rapidjson_value_get_member_string(const RapidJsonValueHandle* h, const char* key,
                                       const char** out) {
    if (!h->val.IsObject() || !h->val.HasMember(key)) return 0;
    const CrtValue& v = h->val[key];
    if (!v.IsString()) return 0;
    *out = v.GetString();
    return 1;
}
void rapidjson_value_add_member_int(RapidJsonValueHandle* h, const char* key, int v) {
    if (!h->val.IsObject()) return;
    CrtValue kv(key, (SizeType)strlen(key), h->alloc);
    h->val.AddMember(kv, CrtValue(v), h->alloc);
}
void rapidjson_value_add_member_string(RapidJsonValueHandle* h, const char* key,
                                        const char* v) {
    if (!h->val.IsObject()) return;
    CrtValue kv(key, (SizeType)strlen(key), h->alloc);
    CrtValue sv(v, (SizeType)strlen(v), h->alloc);
    h->val.AddMember(kv, sv, h->alloc);
}

int rapidjson_value_get_element_int(const RapidJsonValueHandle* h, size_t index) {
    if (!h->val.IsArray() || index >= h->val.Size()) return 0;
    return h->val[(SizeType)index].GetInt();
}
double rapidjson_value_get_element_double(const RapidJsonValueHandle* h, size_t index) {
    if (!h->val.IsArray() || index >= h->val.Size()) return 0.0;
    return h->val[(SizeType)index].GetDouble();
}

void rapidjson_value_push_back_int(RapidJsonValueHandle* h, int v) {
    if (!h->val.IsArray()) return;
    h->val.PushBack(CrtValue(v), h->alloc);
}
void rapidjson_value_push_back_double(RapidJsonValueHandle* h, double v) {
    if (!h->val.IsArray()) return;
    h->val.PushBack(CrtValue(v), h->alloc);
}
void rapidjson_value_push_back_string(RapidJsonValueHandle* h, const char* s) {
    if (!h->val.IsArray()) return;
    CrtValue sv(s, (SizeType)strlen(s), h->alloc);
    h->val.PushBack(sv, h->alloc);
}
void rapidjson_value_push_back_bool(RapidJsonValueHandle* h, int b) {
    if (!h->val.IsArray()) return;
    h->val.PushBack(CrtValue(b != 0), h->alloc);
}

int rapidjson_value_get_type(const RapidJsonValueHandle* h) {
    return (int)h->val.GetType();
}
