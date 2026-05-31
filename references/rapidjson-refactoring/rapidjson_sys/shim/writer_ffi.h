// FFI shim header for rapidjson StringBuffer, Writer and PrettyWriter.
//
// Covers the API surface used by:
//   example/simplewriter, pretty, prettyauto, condense, capitalize,
//          filterkey, filterkeydom
//   test/unittest/writertest, prettywritertest, stringbuffertest

#pragma once

#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

// ── Opaque handle types ──────────────────────────────────────────────────────

typedef struct RapidJsonStringBufferHandle  RapidJsonStringBufferHandle;
typedef struct RapidJsonWriterHandle        RapidJsonWriterHandle;
typedef struct RapidJsonPrettyWriterHandle  RapidJsonPrettyWriterHandle;

// ── StringBuffer ─────────────────────────────────────────────────────────────

RapidJsonStringBufferHandle* rapidjson_stringbuffer_new(void);
void   rapidjson_stringbuffer_free(RapidJsonStringBufferHandle* sb);
const char* rapidjson_stringbuffer_get_string(const RapidJsonStringBufferHandle* sb);
size_t      rapidjson_stringbuffer_get_size(const RapidJsonStringBufferHandle* sb);
void        rapidjson_stringbuffer_clear(RapidJsonStringBufferHandle* sb);
// Push a single character (for generic buffer tests)
void        rapidjson_stringbuffer_put(RapidJsonStringBufferHandle* sb, char c);
// Pop n characters
void        rapidjson_stringbuffer_pop(RapidJsonStringBufferHandle* sb, size_t n);
// Shrink reserved capacity to current length
void        rapidjson_stringbuffer_shrink_to_fit(RapidJsonStringBufferHandle* sb);

// ── Writer<StringBuffer> ─────────────────────────────────────────────────────

RapidJsonWriterHandle* rapidjson_writer_new(RapidJsonStringBufferHandle* sb);
void rapidjson_writer_free(RapidJsonWriterHandle* w);
int  rapidjson_writer_is_complete(const RapidJsonWriterHandle* w);

int rapidjson_writer_null(RapidJsonWriterHandle* w);
int rapidjson_writer_bool(RapidJsonWriterHandle* w, int b);
int rapidjson_writer_int(RapidJsonWriterHandle* w, int i);
int rapidjson_writer_uint(RapidJsonWriterHandle* w, unsigned u);
int rapidjson_writer_int64(RapidJsonWriterHandle* w, long long i);
int rapidjson_writer_uint64(RapidJsonWriterHandle* w, unsigned long long u);
int rapidjson_writer_double(RapidJsonWriterHandle* w, double d);
int rapidjson_writer_string(RapidJsonWriterHandle* w, const char* s, size_t len);
int rapidjson_writer_string_cstr(RapidJsonWriterHandle* w, const char* s);
int rapidjson_writer_raw_value(RapidJsonWriterHandle* w, const char* json, size_t len);
int rapidjson_writer_start_object(RapidJsonWriterHandle* w);
int rapidjson_writer_end_object(RapidJsonWriterHandle* w);
int rapidjson_writer_start_array(RapidJsonWriterHandle* w);
int rapidjson_writer_end_array(RapidJsonWriterHandle* w);
int rapidjson_writer_key(RapidJsonWriterHandle* w, const char* key, size_t len);
int rapidjson_writer_key_cstr(RapidJsonWriterHandle* w, const char* key);

// ── PrettyWriter<StringBuffer> ───────────────────────────────────────────────

RapidJsonPrettyWriterHandle* rapidjson_prettywriter_new(RapidJsonStringBufferHandle* sb);
void rapidjson_prettywriter_free(RapidJsonPrettyWriterHandle* w);
void rapidjson_prettywriter_set_indent(RapidJsonPrettyWriterHandle* w, char ch, unsigned count);
void rapidjson_prettywriter_set_format_options(RapidJsonPrettyWriterHandle* w, int options);

int rapidjson_prettywriter_null(RapidJsonPrettyWriterHandle* w);
int rapidjson_prettywriter_bool(RapidJsonPrettyWriterHandle* w, int b);
int rapidjson_prettywriter_int(RapidJsonPrettyWriterHandle* w, int i);
int rapidjson_prettywriter_uint(RapidJsonPrettyWriterHandle* w, unsigned u);
int rapidjson_prettywriter_int64(RapidJsonPrettyWriterHandle* w, long long i);
int rapidjson_prettywriter_uint64(RapidJsonPrettyWriterHandle* w, unsigned long long u);
int rapidjson_prettywriter_double(RapidJsonPrettyWriterHandle* w, double d);
int rapidjson_prettywriter_string(RapidJsonPrettyWriterHandle* w, const char* s, size_t len);
int rapidjson_prettywriter_string_cstr(RapidJsonPrettyWriterHandle* w, const char* s);
int rapidjson_prettywriter_raw_value(RapidJsonPrettyWriterHandle* w, const char* json, size_t len);
int rapidjson_prettywriter_start_object(RapidJsonPrettyWriterHandle* w);
int rapidjson_prettywriter_end_object(RapidJsonPrettyWriterHandle* w);
int rapidjson_prettywriter_start_array(RapidJsonPrettyWriterHandle* w);
int rapidjson_prettywriter_end_array(RapidJsonPrettyWriterHandle* w);
int rapidjson_prettywriter_key(RapidJsonPrettyWriterHandle* w, const char* key, size_t len);
int rapidjson_prettywriter_key_cstr(RapidJsonPrettyWriterHandle* w, const char* key);
int rapidjson_prettywriter_is_complete(const RapidJsonPrettyWriterHandle* w);

#ifdef __cplusplus
}
#endif
