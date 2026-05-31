// FFI shim implementation for rapidjson encoding utilities.

#include "encoding_ffi.h"

#include <new>
#include <cstring>
#include <stdint.h>

#include "rapidjson/encodings.h"

using namespace rapidjson;

// ── Minimal stream wrappers ───────────────────────────────────────────────────

template<typename CharT>
struct BufOutputStream {
    typedef CharT Ch;
    CharT* buf;
    size_t cap;
    size_t written;
    BufOutputStream(CharT* b, size_t c) : buf(b), cap(c), written(0) {}
    void Put(CharT c) { if (written < cap) buf[written++] = c; }
    void PutUnsafe(CharT c) { Put(c); }
    void Flush() {}
};

template<typename CharT>
struct BufInputStream {
    typedef CharT Ch;
    const CharT* src;
    const CharT* end_;
    BufInputStream(const CharT* s, size_t n) : src(s), end_(s + n) {}
    Ch Peek() const { return src < end_ ? *src : Ch(0); }
    Ch Take()       { return src < end_ ? *src++ : Ch(0); }
    size_t Tell() const { return 0; }
};

// ── UTF-8 ─────────────────────────────────────────────────────────────────────

size_t rapidjson_utf8_encode(uint32_t cp, char* buf) {
    BufOutputStream<char> os(buf, 6);
    UTF8<>::Encode(os, cp);       // returns void
    return os.written;
}

int rapidjson_utf8_decode(const char** src, const char* end, uint32_t* out) {
    BufInputStream<char> is(*src, (size_t)(end - *src));
    unsigned cp = 0;
    if (!UTF8<>::Decode(is, &cp)) return 0;
    *out = (uint32_t)cp;
    *src = is.src;
    return 1;
}

int rapidjson_utf8_validate(const char* data, size_t len) {
    BufInputStream<char> is(data, len);
    unsigned cp = 0;
    while (is.src < is.end_) {
        if (!UTF8<>::Decode(is, &cp)) return 0;
    }
    return 1;
}

// ── UTF-16 ────────────────────────────────────────────────────────────────────

size_t rapidjson_utf16_encode(uint32_t cp, uint16_t* buf) {
    BufOutputStream<uint16_t> os(buf, 2);
    UTF16LE<>::Encode(os, cp);     // returns void
    return os.written;
}

int rapidjson_utf16_decode(const uint16_t** src, const uint16_t* end, uint32_t* out) {
    BufInputStream<uint16_t> is(*src, (size_t)(end - *src));
    unsigned cp = 0;
    if (!UTF16LE<>::Decode(is, &cp)) return 0;
    *out = (uint32_t)cp;
    *src = is.src;
    return 1;
}

int rapidjson_utf16_validate(const uint16_t* data, size_t len) {
    BufInputStream<uint16_t> is(data, len);
    unsigned cp = 0;
    while (is.src < is.end_) {
        if (!UTF16LE<>::Decode(is, &cp)) return 0;
    }
    return 1;
}

// ── UTF-32 ────────────────────────────────────────────────────────────────────

int rapidjson_utf32_encode(uint32_t cp, uint32_t* out) {
    BufOutputStream<uint32_t> os(out, 1);
    UTF32LE<>::Encode(os, cp);     // returns void
    return (os.written > 0) ? 1 : 0;
}

int rapidjson_utf32_decode(const uint32_t** src, const uint32_t* end, uint32_t* out) {
    BufInputStream<uint32_t> is(*src, (size_t)(end - *src));
    unsigned cp = 0;
    if (!UTF32LE<>::Decode(is, &cp)) return 0;
    *out = (uint32_t)cp;
    *src = is.src;
    return 1;
}

int rapidjson_utf32_validate(const uint32_t* data, size_t len) {
    BufInputStream<uint32_t> is(data, len);
    unsigned cp = 0;
    while (is.src < is.end_) {
        if (!UTF32LE<>::Decode(is, &cp)) return 0;
    }
    return 1;
}

// ── Transcode helpers ─────────────────────────────────────────────────────────

size_t rapidjson_transcode_utf8_to_utf16(const char* src, size_t src_len,
                                          uint16_t* dst, size_t dst_cap) {
    BufInputStream<char>      is(src, src_len);
    BufOutputStream<uint16_t> os(dst, dst_cap);
    unsigned cp = 0;
    while (is.src < is.end_) {
        if (!UTF8<>::Decode(is, &cp)) return (size_t)-1;
        UTF16LE<>::Encode(os, cp);
    }
    return os.written;
}

size_t rapidjson_transcode_utf16_to_utf8(const uint16_t* src, size_t src_units,
                                          char* dst, size_t dst_cap) {
    BufInputStream<uint16_t> is(src, src_units);
    BufOutputStream<char>    os(dst, dst_cap);
    unsigned cp = 0;
    while (is.src < is.end_) {
        if (!UTF16LE<>::Decode(is, &cp)) return (size_t)-1;
        UTF8<>::Encode(os, cp);
    }
    return os.written;
}

