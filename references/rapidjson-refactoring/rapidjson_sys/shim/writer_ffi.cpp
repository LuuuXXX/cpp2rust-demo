// FFI shim implementation for rapidjson StringBuffer, Writer, PrettyWriter.

#include "writer_ffi.h"
#include "internal_handles.h"

#include <new>
#include <cstring>

#include "rapidjson/stringbuffer.h"
#include "rapidjson/writer.h"
#include "rapidjson/prettywriter.h"

using namespace rapidjson;

// ── StringBuffer ─────────────────────────────────────────────────────────────
// RapidJsonStringBufferHandle is defined in internal_handles.h.

RapidJsonStringBufferHandle* rapidjson_stringbuffer_new(void) {
    return new (std::nothrow) RapidJsonStringBufferHandle();
}
void rapidjson_stringbuffer_free(RapidJsonStringBufferHandle* h) { delete h; }
const char* rapidjson_stringbuffer_get_string(const RapidJsonStringBufferHandle* h) {
    return h->sb.GetString();
}
size_t rapidjson_stringbuffer_get_size(const RapidJsonStringBufferHandle* h) {
    return h->sb.GetSize();
}
void rapidjson_stringbuffer_clear(RapidJsonStringBufferHandle* h) {
    h->sb.Clear();
}
void rapidjson_stringbuffer_put(RapidJsonStringBufferHandle* h, char c) {
    *(h->sb.Push(1)) = c;
}
void rapidjson_stringbuffer_pop(RapidJsonStringBufferHandle* h, size_t n) {
    h->sb.Pop(n);
}
void rapidjson_stringbuffer_shrink_to_fit(RapidJsonStringBufferHandle* h) {
    h->sb.ShrinkToFit();
}

// ── Writer ────────────────────────────────────────────────────────────────────

struct RapidJsonWriterHandle {
    Writer<StringBuffer> w;
    RapidJsonWriterHandle(StringBuffer& sb) : w(sb) {}
};

RapidJsonWriterHandle* rapidjson_writer_new(RapidJsonStringBufferHandle* sb) {
    return new (std::nothrow) RapidJsonWriterHandle(sb->sb);
}
void rapidjson_writer_free(RapidJsonWriterHandle* h) { delete h; }
int  rapidjson_writer_is_complete(const RapidJsonWriterHandle* h) {
    return h->w.IsComplete() ? 1 : 0;
}

int rapidjson_writer_null(RapidJsonWriterHandle* h)          { return h->w.Null(); }
int rapidjson_writer_bool(RapidJsonWriterHandle* h, int b)   { return h->w.Bool(b != 0); }
int rapidjson_writer_int(RapidJsonWriterHandle* h, int i)    { return h->w.Int(i); }
int rapidjson_writer_uint(RapidJsonWriterHandle* h, unsigned u){ return h->w.Uint(u); }
int rapidjson_writer_int64(RapidJsonWriterHandle* h, long long i)          { return h->w.Int64(i); }
int rapidjson_writer_uint64(RapidJsonWriterHandle* h, unsigned long long u){ return h->w.Uint64(u); }
int rapidjson_writer_double(RapidJsonWriterHandle* h, double d){ return h->w.Double(d); }
int rapidjson_writer_string(RapidJsonWriterHandle* h, const char* s, size_t len) {
    return h->w.String(s, (SizeType)len);
}
int rapidjson_writer_string_cstr(RapidJsonWriterHandle* h, const char* s) {
    return h->w.String(s, (SizeType)strlen(s));
}
int rapidjson_writer_raw_value(RapidJsonWriterHandle* h, const char* json, size_t len) {
    return h->w.RawValue(json, len, kNullType);
}
int rapidjson_writer_start_object(RapidJsonWriterHandle* h) { return h->w.StartObject(); }
int rapidjson_writer_end_object(RapidJsonWriterHandle* h)   { return h->w.EndObject(); }
int rapidjson_writer_start_array(RapidJsonWriterHandle* h)  { return h->w.StartArray(); }
int rapidjson_writer_end_array(RapidJsonWriterHandle* h)    { return h->w.EndArray(); }
int rapidjson_writer_key(RapidJsonWriterHandle* h, const char* k, size_t l) {
    return h->w.Key(k, (SizeType)l);
}
int rapidjson_writer_key_cstr(RapidJsonWriterHandle* h, const char* k) {
    return h->w.Key(k, (SizeType)strlen(k));
}

// ── PrettyWriter ──────────────────────────────────────────────────────────────

struct RapidJsonPrettyWriterHandle {
    PrettyWriter<StringBuffer> w;
    RapidJsonPrettyWriterHandle(StringBuffer& sb) : w(sb) {}
};

RapidJsonPrettyWriterHandle* rapidjson_prettywriter_new(RapidJsonStringBufferHandle* sb) {
    return new (std::nothrow) RapidJsonPrettyWriterHandle(sb->sb);
}
void rapidjson_prettywriter_free(RapidJsonPrettyWriterHandle* h) { delete h; }
void rapidjson_prettywriter_set_indent(RapidJsonPrettyWriterHandle* h, char ch, unsigned count) {
    h->w.SetIndent(ch, count);
}
void rapidjson_prettywriter_set_format_options(RapidJsonPrettyWriterHandle* h, int opts) {
    h->w.SetFormatOptions((PrettyFormatOptions)opts);
}

int rapidjson_prettywriter_null(RapidJsonPrettyWriterHandle* h)          { return h->w.Null(); }
int rapidjson_prettywriter_bool(RapidJsonPrettyWriterHandle* h, int b)   { return h->w.Bool(b != 0); }
int rapidjson_prettywriter_int(RapidJsonPrettyWriterHandle* h, int i)    { return h->w.Int(i); }
int rapidjson_prettywriter_uint(RapidJsonPrettyWriterHandle* h, unsigned u){ return h->w.Uint(u); }
int rapidjson_prettywriter_int64(RapidJsonPrettyWriterHandle* h, long long i)          { return h->w.Int64(i); }
int rapidjson_prettywriter_uint64(RapidJsonPrettyWriterHandle* h, unsigned long long u){ return h->w.Uint64(u); }
int rapidjson_prettywriter_double(RapidJsonPrettyWriterHandle* h, double d){ return h->w.Double(d); }
int rapidjson_prettywriter_string(RapidJsonPrettyWriterHandle* h, const char* s, size_t len) {
    return h->w.String(s, (SizeType)len);
}
int rapidjson_prettywriter_string_cstr(RapidJsonPrettyWriterHandle* h, const char* s) {
    return h->w.String(s, (SizeType)strlen(s));
}
int rapidjson_prettywriter_raw_value(RapidJsonPrettyWriterHandle* h, const char* json, size_t len) {
    return h->w.RawValue(json, len, kNullType);
}
int rapidjson_prettywriter_start_object(RapidJsonPrettyWriterHandle* h) { return h->w.StartObject(); }
int rapidjson_prettywriter_end_object(RapidJsonPrettyWriterHandle* h)   { return h->w.EndObject(); }
int rapidjson_prettywriter_start_array(RapidJsonPrettyWriterHandle* h)  { return h->w.StartArray(); }
int rapidjson_prettywriter_end_array(RapidJsonPrettyWriterHandle* h)    { return h->w.EndArray(); }
int rapidjson_prettywriter_key(RapidJsonPrettyWriterHandle* h, const char* k, size_t l) {
    return h->w.Key(k, (SizeType)l);
}
int rapidjson_prettywriter_key_cstr(RapidJsonPrettyWriterHandle* h, const char* k) {
    return h->w.Key(k, (SizeType)strlen(k));
}
int rapidjson_prettywriter_is_complete(const RapidJsonPrettyWriterHandle* h) {
    return h->w.IsComplete() ? 1 : 0;
}
