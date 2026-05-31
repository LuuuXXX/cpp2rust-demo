// FFI shim header for rapidjson Pointer (JSON Pointer RFC 6901).
//
// Covers the API surface used by:
//   example/traverseaspointer
//   test/unittest/pointertest

#pragma once

#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef struct RapidJsonDocumentHandle  RapidJsonDocumentHandle;
typedef struct RapidJsonPointerHandle   RapidJsonPointerHandle;

// ── Pointer lifecycle ─────────────────────────────────────────────────────────

RapidJsonPointerHandle* rapidjson_pointer_new(const char* pointer_str);
void                    rapidjson_pointer_free(RapidJsonPointerHandle* ptr);

// Validity / error
int    rapidjson_pointer_is_valid(const RapidJsonPointerHandle* ptr);
int    rapidjson_pointer_get_parse_error_code(const RapidJsonPointerHandle* ptr);
size_t rapidjson_pointer_get_parse_error_offset(const RapidJsonPointerHandle* ptr);

// ── Operations on Document ────────────────────────────────────────────────────

// Get: returns 1 and writes value fields if found, 0 if not.
int rapidjson_pointer_get_int(const RapidJsonPointerHandle* ptr,
                               const RapidJsonDocumentHandle* doc,
                               int* out_val);
int rapidjson_pointer_get_string(const RapidJsonPointerHandle* ptr,
                                  const RapidJsonDocumentHandle* doc,
                                  const char** out_val);
int rapidjson_pointer_get_bool(const RapidJsonPointerHandle* ptr,
                                const RapidJsonDocumentHandle* doc,
                                int* out_val);
int rapidjson_pointer_get_double(const RapidJsonPointerHandle* ptr,
                                  const RapidJsonDocumentHandle* doc,
                                  double* out_val);

// GetWithDefault: if node absent, inserts default and returns it.
int rapidjson_pointer_get_with_default_int(const RapidJsonPointerHandle* ptr,
                                            RapidJsonDocumentHandle* doc,
                                            int default_val,
                                            int* out_val);
int rapidjson_pointer_get_with_default_string(const RapidJsonPointerHandle* ptr,
                                               RapidJsonDocumentHandle* doc,
                                               const char* default_val,
                                               const char** out_val);

// Set: sets the value at pointer path (creating intermediaries).
void rapidjson_pointer_set_int(const RapidJsonPointerHandle* ptr,
                                RapidJsonDocumentHandle* doc,
                                int val);
void rapidjson_pointer_set_string(const RapidJsonPointerHandle* ptr,
                                   RapidJsonDocumentHandle* doc,
                                   const char* val);
void rapidjson_pointer_set_bool(const RapidJsonPointerHandle* ptr,
                                 RapidJsonDocumentHandle* doc,
                                 int val);
void rapidjson_pointer_set_double(const RapidJsonPointerHandle* ptr,
                                   RapidJsonDocumentHandle* doc,
                                   double val);
void rapidjson_pointer_set_null(const RapidJsonPointerHandle* ptr,
                                 RapidJsonDocumentHandle* doc);

// Create: returns non-zero if node was newly created.
int rapidjson_pointer_create(const RapidJsonPointerHandle* ptr,
                              RapidJsonDocumentHandle* doc);

// Erase: returns 1 if node existed and was erased.
int rapidjson_pointer_erase(const RapidJsonPointerHandle* ptr,
                             RapidJsonDocumentHandle* doc);

#ifdef __cplusplus
}
#endif
