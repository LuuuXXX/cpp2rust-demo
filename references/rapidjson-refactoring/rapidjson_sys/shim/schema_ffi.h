// FFI shim header for rapidjson SchemaDocument / SchemaValidator.
//
// Covers the API surface used by:
//   example/schemavalidator
//   test/unittest/schematest

#pragma once

#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef struct RapidJsonDocumentHandle      RapidJsonDocumentHandle;
typedef struct RapidJsonSchemaDocHandle     RapidJsonSchemaDocHandle;
typedef struct RapidJsonSchemaValidHandle   RapidJsonSchemaValidHandle;

// ── SchemaDocument ────────────────────────────────────────────────────────────

// Build a SchemaDocument from a parsed Document handle.
// Returns NULL if doc does not contain a valid JSON Schema object.
RapidJsonSchemaDocHandle* rapidjson_schemadoc_new(const RapidJsonDocumentHandle* doc);
void                      rapidjson_schemadoc_free(RapidJsonSchemaDocHandle* sd);

// ── SchemaValidator ───────────────────────────────────────────────────────────

RapidJsonSchemaValidHandle* rapidjson_schemavalidator_new(const RapidJsonSchemaDocHandle* sd);
void                        rapidjson_schemavalidator_free(RapidJsonSchemaValidHandle* sv);

// Validate a document; returns 1 if valid, 0 otherwise.
int rapidjson_schemavalidator_validate(RapidJsonSchemaValidHandle* sv,
                                        const RapidJsonDocumentHandle* doc);

// Inspect failures
int    rapidjson_schemavalidator_is_valid(const RapidJsonSchemaValidHandle* sv);
// Writes pointer string to buf (up to buf_len bytes, NUL-terminated); returns bytes written.
size_t rapidjson_schemavalidator_get_invalid_schema_pointer(
            const RapidJsonSchemaValidHandle* sv, char* buf, size_t buf_len);
const char* rapidjson_schemavalidator_get_invalid_schema_keyword(
                const RapidJsonSchemaValidHandle* sv);
size_t rapidjson_schemavalidator_get_invalid_document_pointer(
            const RapidJsonSchemaValidHandle* sv, char* buf, size_t buf_len);

// Reset so the validator can be reused.
void rapidjson_schemavalidator_reset(RapidJsonSchemaValidHandle* sv);

#ifdef __cplusplus
}
#endif
