// FFI shim header for standalone rapidjson Value operations.
//
// Covers the API surface tested by:
//   test/unittest/valuetest
//
// NOTE: Most Value functionality is already exposed through document_ffi.h.
// This shim adds the remaining Value operations not covered there.

#pragma once

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef struct RapidJsonValueHandle RapidJsonValueHandle;

// ── Value iteration (object members) ─────────────────────────────────────────

// Iterate over object members.  Returns 1 while there are more members.
// index is zero-based; writes key/value for the member at that index.
int rapidjson_value_get_member_at(const RapidJsonValueHandle* val,
                                   size_t index,
                                   const char** out_key,
                                   size_t* out_key_len);

// ── Value remove member ───────────────────────────────────────────────────────

// Remove an object member by key; returns 1 if found and removed.
int rapidjson_value_remove_member(RapidJsonValueHandle* val, const char* key);

// ── Value array operations ────────────────────────────────────────────────────

// Pop the last element; undefined if array is empty.
void rapidjson_value_pop_back(RapidJsonValueHandle* arr);

// Erase element at index; returns 1 on success.
int rapidjson_value_erase_index(RapidJsonValueHandle* arr, size_t index);

// Reserve capacity for array.
void rapidjson_value_reserve(RapidJsonValueHandle* arr, size_t new_capacity);

// ── Numeric conversion helpers ────────────────────────────────────────────────

// Returns 1 if the value can be losslessly represented as the target type.
int rapidjson_value_is_lossless_double(const RapidJsonValueHandle* val);
int rapidjson_value_is_lossless_float(const RapidJsonValueHandle* val);

// Get as float (single precision).
float rapidjson_value_get_float(const RapidJsonValueHandle* val);

// ── Object / array clear ──────────────────────────────────────────────────────

void rapidjson_value_clear_members(RapidJsonValueHandle* obj);
void rapidjson_value_clear_elements(RapidJsonValueHandle* arr);

#ifdef __cplusplus
}
#endif
