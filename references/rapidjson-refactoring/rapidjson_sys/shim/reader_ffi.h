// FFI shim header for rapidjson Reader (SAX pull-parser).
//
// Covers the API surface used by:
//   example/simplereader, simplepullreader, messagereader, parsebyparts
//   test/unittest/readertest

#pragma once

#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

// ── Opaque handle types ──────────────────────────────────────────────────────

typedef struct RapidJsonReaderHandle          RapidJsonReaderHandle;
typedef struct RapidJsonStringStreamHandle    RapidJsonStringStreamHandle;
typedef struct RapidJsonInsituStreamHandle    RapidJsonInsituStreamHandle;

// ── SAX event handler callback struct ───────────────────────────────────────
//
// Callers supply a C callback table; the shim wraps it in a rapidjson Handler.

typedef struct RapidJsonHandlerCallbacks {
    void* user_data;
    int (*on_null)(void*);
    int (*on_bool)(void*, int);
    int (*on_int)(void*, int);
    int (*on_uint)(void*, unsigned);
    int (*on_int64)(void*, long long);
    int (*on_uint64)(void*, unsigned long long);
    int (*on_double)(void*, double);
    int (*on_string)(void*, const char* s, size_t len, int copy);
    int (*on_start_object)(void*);
    int (*on_key)(void*, const char* s, size_t len, int copy);
    int (*on_end_object)(void*, size_t member_count);
    int (*on_start_array)(void*);
    int (*on_end_array)(void*, size_t element_count);
} RapidJsonHandlerCallbacks;

// ── StringStream ─────────────────────────────────────────────────────────────

RapidJsonStringStreamHandle* rapidjson_stringstream_new(const char* json);
void rapidjson_stringstream_free(RapidJsonStringStreamHandle* ss);

// ── InsituStringStream ───────────────────────────────────────────────────────

RapidJsonInsituStreamHandle* rapidjson_insitustream_new(char* json);
void rapidjson_insitustream_free(RapidJsonInsituStreamHandle* ss);

// ── Reader ───────────────────────────────────────────────────────────────────

RapidJsonReaderHandle* rapidjson_reader_new(void);
void rapidjson_reader_free(RapidJsonReaderHandle* r);

// Parse from a StringStream using the provided SAX callback table.
// Returns non-zero on success.
int rapidjson_reader_parse(RapidJsonReaderHandle* r,
                           RapidJsonStringStreamHandle* ss,
                           const RapidJsonHandlerCallbacks* cb);

// Parse from an InsituStringStream.
int rapidjson_reader_parse_insitu(RapidJsonReaderHandle* r,
                                   RapidJsonInsituStreamHandle* ss,
                                   const RapidJsonHandlerCallbacks* cb);

// Iterative pull-parser: advance one token, fire callback.
// Returns 0 when done or on error.
int rapidjson_reader_iterate_next(RapidJsonReaderHandle* r,
                                   RapidJsonStringStreamHandle* ss,
                                   const RapidJsonHandlerCallbacks* cb);

// Error inspection
int    rapidjson_reader_has_parse_error(const RapidJsonReaderHandle* r);
int    rapidjson_reader_get_parse_error_code(const RapidJsonReaderHandle* r);
size_t rapidjson_reader_get_error_offset(const RapidJsonReaderHandle* r);

#ifdef __cplusplus
}
#endif
