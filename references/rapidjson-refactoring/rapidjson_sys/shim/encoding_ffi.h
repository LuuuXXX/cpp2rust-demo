// FFI shim header for rapidjson encoding transcoding utilities.
//
// Covers the API surface used by:
//   test/unittest/encodingstest

#pragma once

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

// ── UTF-8 ─────────────────────────────────────────────────────────────────────

// Encode a Unicode code point to UTF-8 into a 6-byte buffer.
// Returns number of bytes written, 0 on error.
size_t rapidjson_utf8_encode(uint32_t codepoint, char* out_buf);

// Decode one UTF-8 sequence from *src; advances *src past the consumed bytes.
// Stores decoded codepoint in *out; returns 1 on success, 0 on error.
int rapidjson_utf8_decode(const char** src, const char* end, uint32_t* out);

// Validate a UTF-8 string; returns 1 if valid.
int rapidjson_utf8_validate(const char* data, size_t len);

// ── UTF-16 (little-endian, stored as uint16_t units) ─────────────────────────

// Encode a Unicode code point to UTF-16LE.
// Returns number of uint16_t units written (1 or 2), 0 on error.
size_t rapidjson_utf16_encode(uint32_t codepoint, uint16_t* out_buf);

// Decode one UTF-16 code unit sequence from *src.
// Returns 1 on success, 0 on error; advances *src.
int rapidjson_utf16_decode(const uint16_t** src, const uint16_t* end, uint32_t* out);

// Validate a UTF-16 buffer (len = number of uint16_t units).
int rapidjson_utf16_validate(const uint16_t* data, size_t len);

// ── UTF-32 (little-endian) ────────────────────────────────────────────────────

// Encode a Unicode code point to UTF-32LE (always 1 uint32_t).
// Returns 1 on success, 0 on error.
int rapidjson_utf32_encode(uint32_t codepoint, uint32_t* out);

// Decode one UTF-32 code unit.
int rapidjson_utf32_decode(const uint32_t** src, const uint32_t* end, uint32_t* out);

// Validate a UTF-32 buffer (len = number of uint32_t units).
int rapidjson_utf32_validate(const uint32_t* data, size_t len);

// ── Transcode helpers ─────────────────────────────────────────────────────────

// Transcode UTF-8 → UTF-16LE (writes into a caller-provided uint16_t buffer).
// Returns number of uint16_t units written, or (size_t)-1 on error.
size_t rapidjson_transcode_utf8_to_utf16(const char* src, size_t src_len,
                                          uint16_t* dst, size_t dst_cap);

// Transcode UTF-16LE → UTF-8.
// Returns number of bytes written, or (size_t)-1 on error.
size_t rapidjson_transcode_utf16_to_utf8(const uint16_t* src, size_t src_units,
                                          char* dst, size_t dst_cap);

#ifdef __cplusplus
}
#endif
