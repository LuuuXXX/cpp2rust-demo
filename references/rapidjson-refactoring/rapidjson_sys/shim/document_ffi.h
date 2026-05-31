// FFI shim header for rapidjson Document / Value (DOM) API.
//
// Covers the API surface used by:
//   example/tutorial, simpledom, filterkeydom, sortkeys, serialize
//   test/unittest/documenttest, valuetest
//
// All functions use an opaque handle pattern with pure C ABI.

#pragma once

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

// ── Opaque handle types ──────────────────────────────────────────────────────

typedef struct RapidJsonDocumentHandle   RapidJsonDocumentHandle;
typedef struct RapidJsonValueHandle      RapidJsonValueHandle;
typedef struct RapidJsonAllocatorHandle  RapidJsonAllocatorHandle;

// ── RapidJsonDocumentHandle lifecycle ───────────────────────────────────────

RapidJsonDocumentHandle* rapidjson_document_new(void);
void                     rapidjson_document_free(RapidJsonDocumentHandle* doc);

// Parsing
// Returns 0 on error, non-zero on success.
int  rapidjson_document_parse(RapidJsonDocumentHandle* doc, const char* json);
int  rapidjson_document_parse_insitu(RapidJsonDocumentHandle* doc, char* json);
int  rapidjson_document_has_parse_error(const RapidJsonDocumentHandle* doc);
int  rapidjson_document_get_parse_error_code(const RapidJsonDocumentHandle* doc);
size_t rapidjson_document_get_error_offset(const RapidJsonDocumentHandle* doc);

// Type predicates on the document root value
int rapidjson_document_is_object(const RapidJsonDocumentHandle* doc);
int rapidjson_document_is_array(const RapidJsonDocumentHandle* doc);
int rapidjson_document_is_string(const RapidJsonDocumentHandle* doc);
int rapidjson_document_is_int(const RapidJsonDocumentHandle* doc);
int rapidjson_document_is_uint(const RapidJsonDocumentHandle* doc);
int rapidjson_document_is_int64(const RapidJsonDocumentHandle* doc);
int rapidjson_document_is_uint64(const RapidJsonDocumentHandle* doc);
int rapidjson_document_is_double(const RapidJsonDocumentHandle* doc);
int rapidjson_document_is_bool(const RapidJsonDocumentHandle* doc);
int rapidjson_document_is_null(const RapidJsonDocumentHandle* doc);

// Scalar getters on the document root value
const char* rapidjson_document_get_string(const RapidJsonDocumentHandle* doc);
size_t      rapidjson_document_get_string_length(const RapidJsonDocumentHandle* doc);
int         rapidjson_document_get_int(const RapidJsonDocumentHandle* doc);
unsigned    rapidjson_document_get_uint(const RapidJsonDocumentHandle* doc);
int64_t     rapidjson_document_get_int64(const RapidJsonDocumentHandle* doc);
uint64_t    rapidjson_document_get_uint64(const RapidJsonDocumentHandle* doc);
double      rapidjson_document_get_double(const RapidJsonDocumentHandle* doc);
int         rapidjson_document_get_bool(const RapidJsonDocumentHandle* doc);

// Object / array queries on the document root value
size_t rapidjson_document_member_count(const RapidJsonDocumentHandle* doc);
int    rapidjson_document_has_member(const RapidJsonDocumentHandle* doc, const char* key);
size_t rapidjson_document_size(const RapidJsonDocumentHandle* doc);

// ── RapidJsonValueHandle: independent Value operations ───────────────────────

// Create a standalone Value (backed by the document's allocator).
RapidJsonValueHandle* rapidjson_value_new_null(void);
RapidJsonValueHandle* rapidjson_value_new_bool(int b);
RapidJsonValueHandle* rapidjson_value_new_int(int v);
RapidJsonValueHandle* rapidjson_value_new_uint(unsigned v);
RapidJsonValueHandle* rapidjson_value_new_int64(int64_t v);
RapidJsonValueHandle* rapidjson_value_new_uint64(uint64_t v);
RapidJsonValueHandle* rapidjson_value_new_double(double v);
RapidJsonValueHandle* rapidjson_value_new_string(const char* s);
RapidJsonValueHandle* rapidjson_value_new_object(void);
RapidJsonValueHandle* rapidjson_value_new_array(void);
void                  rapidjson_value_free(RapidJsonValueHandle* val);

// Type predicates
int rapidjson_value_is_null(const RapidJsonValueHandle* val);
int rapidjson_value_is_bool(const RapidJsonValueHandle* val);
int rapidjson_value_is_int(const RapidJsonValueHandle* val);
int rapidjson_value_is_uint(const RapidJsonValueHandle* val);
int rapidjson_value_is_int64(const RapidJsonValueHandle* val);
int rapidjson_value_is_uint64(const RapidJsonValueHandle* val);
int rapidjson_value_is_double(const RapidJsonValueHandle* val);
int rapidjson_value_is_string(const RapidJsonValueHandle* val);
int rapidjson_value_is_object(const RapidJsonValueHandle* val);
int rapidjson_value_is_array(const RapidJsonValueHandle* val);
int rapidjson_value_is_number(const RapidJsonValueHandle* val);

// Scalar getters
int         rapidjson_value_get_bool(const RapidJsonValueHandle* val);
int         rapidjson_value_get_int(const RapidJsonValueHandle* val);
unsigned    rapidjson_value_get_uint(const RapidJsonValueHandle* val);
int64_t     rapidjson_value_get_int64(const RapidJsonValueHandle* val);
uint64_t    rapidjson_value_get_uint64(const RapidJsonValueHandle* val);
double      rapidjson_value_get_double(const RapidJsonValueHandle* val);
const char* rapidjson_value_get_string(const RapidJsonValueHandle* val);
size_t      rapidjson_value_get_string_length(const RapidJsonValueHandle* val);

// Scalar setters
void rapidjson_value_set_null(RapidJsonValueHandle* val);
void rapidjson_value_set_bool(RapidJsonValueHandle* val, int b);
void rapidjson_value_set_int(RapidJsonValueHandle* val, int v);
void rapidjson_value_set_uint(RapidJsonValueHandle* val, unsigned v);
void rapidjson_value_set_int64(RapidJsonValueHandle* val, int64_t v);
void rapidjson_value_set_uint64(RapidJsonValueHandle* val, uint64_t v);
void rapidjson_value_set_double(RapidJsonValueHandle* val, double v);
void rapidjson_value_set_string_copy(RapidJsonValueHandle* val, const char* s, size_t len);

// Object / array size
size_t rapidjson_value_member_count(const RapidJsonValueHandle* val);
size_t rapidjson_value_size(const RapidJsonValueHandle* val);
int    rapidjson_value_has_member(const RapidJsonValueHandle* val, const char* key);
int    rapidjson_value_empty(const RapidJsonValueHandle* val);

// CopyFrom / Swap / Move
void rapidjson_value_copy_from(RapidJsonValueHandle* dst, const RapidJsonValueHandle* src);
void rapidjson_value_swap(RapidJsonValueHandle* a, RapidJsonValueHandle* b);

// Object member get/set by key (string copy variant)
// Returns 0 if key not found.
int rapidjson_value_get_member_int(const RapidJsonValueHandle* val, const char* key,
                                    int* out);
int rapidjson_value_get_member_string(const RapidJsonValueHandle* val, const char* key,
                                       const char** out);
void rapidjson_value_add_member_int(RapidJsonValueHandle* obj, const char* key, int v);
void rapidjson_value_add_member_string(RapidJsonValueHandle* obj, const char* key,
                                        const char* v);

// Array element get by index
int    rapidjson_value_get_element_int(const RapidJsonValueHandle* val, size_t index);
double rapidjson_value_get_element_double(const RapidJsonValueHandle* val, size_t index);

// Push back scalars to array
void rapidjson_value_push_back_int(RapidJsonValueHandle* arr, int v);
void rapidjson_value_push_back_double(RapidJsonValueHandle* arr, double v);
void rapidjson_value_push_back_string(RapidJsonValueHandle* arr, const char* s);
void rapidjson_value_push_back_bool(RapidJsonValueHandle* arr, int b);

// Get type as integer (maps to rapidjson::Type enum)
int rapidjson_value_get_type(const RapidJsonValueHandle* val);

#ifdef __cplusplus
}
#endif
