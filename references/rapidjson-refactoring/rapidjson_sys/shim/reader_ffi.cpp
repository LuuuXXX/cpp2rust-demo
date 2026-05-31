// FFI shim implementation for rapidjson Reader (SAX pull-parser).

#include "reader_ffi.h"

#include <new>
#include <cstring>

#include "rapidjson/reader.h"
#include "rapidjson/stringbuffer.h"

using namespace rapidjson;

// ── StringStream ─────────────────────────────────────────────────────────────

struct RapidJsonStringStreamHandle {
    StringStream ss;
    RapidJsonStringStreamHandle(const char* json) : ss(json) {}
};

RapidJsonStringStreamHandle* rapidjson_stringstream_new(const char* json) {
    return new (std::nothrow) RapidJsonStringStreamHandle(json);
}
void rapidjson_stringstream_free(RapidJsonStringStreamHandle* h) { delete h; }

// ── InsituStringStream ───────────────────────────────────────────────────────

struct RapidJsonInsituStreamHandle {
    InsituStringStream ss;
    RapidJsonInsituStreamHandle(char* json) : ss(json) {}
};

RapidJsonInsituStreamHandle* rapidjson_insitustream_new(char* json) {
    return new (std::nothrow) RapidJsonInsituStreamHandle(json);
}
void rapidjson_insitustream_free(RapidJsonInsituStreamHandle* h) { delete h; }

// ── SAX handler bridge ───────────────────────────────────────────────────────

struct CCallbackHandler {
    const RapidJsonHandlerCallbacks* cb;

    typedef char Ch;

    bool Null()        { return cb->on_null   ? cb->on_null(cb->user_data)       != 0 : true; }
    bool Bool(bool b)  { return cb->on_bool   ? cb->on_bool(cb->user_data, b?1:0) != 0 : true; }
    bool Int(int i)    { return cb->on_int    ? cb->on_int(cb->user_data, i)      != 0 : true; }
    bool Uint(unsigned u){ return cb->on_uint ? cb->on_uint(cb->user_data, u)     != 0 : true; }
    bool Int64(int64_t i){ return cb->on_int64  ? cb->on_int64(cb->user_data, (long long)i)         != 0 : true; }
    bool Uint64(uint64_t u){ return cb->on_uint64 ? cb->on_uint64(cb->user_data, (unsigned long long)u) != 0 : true; }
    bool Double(double d){ return cb->on_double ? cb->on_double(cb->user_data, d) != 0 : true; }
    bool String(const char* s, SizeType len, bool copy) {
        return cb->on_string ? cb->on_string(cb->user_data, s, (size_t)len, copy?1:0) != 0 : true;
    }
    bool StartObject() { return cb->on_start_object ? cb->on_start_object(cb->user_data) != 0 : true; }
    bool Key(const char* s, SizeType len, bool copy) {
        return cb->on_key ? cb->on_key(cb->user_data, s, (size_t)len, copy?1:0) != 0 : true;
    }
    bool EndObject(SizeType mc) {
        return cb->on_end_object ? cb->on_end_object(cb->user_data, (size_t)mc) != 0 : true;
    }
    bool StartArray() { return cb->on_start_array ? cb->on_start_array(cb->user_data) != 0 : true; }
    bool EndArray(SizeType ec) {
        return cb->on_end_array ? cb->on_end_array(cb->user_data, (size_t)ec) != 0 : true;
    }
    bool RawNumber(const char* s, SizeType len, bool copy) {
        return String(s, len, copy);
    }
};

// ── Reader ───────────────────────────────────────────────────────────────────

struct RapidJsonReaderHandle {
    Reader reader;
};

RapidJsonReaderHandle* rapidjson_reader_new(void) {
    return new (std::nothrow) RapidJsonReaderHandle();
}
void rapidjson_reader_free(RapidJsonReaderHandle* h) { delete h; }

int rapidjson_reader_parse(RapidJsonReaderHandle* r,
                           RapidJsonStringStreamHandle* ss,
                           const RapidJsonHandlerCallbacks* cb) {
    CCallbackHandler handler{cb};
    return r->reader.Parse(ss->ss, handler) ? 1 : 0;
}

int rapidjson_reader_parse_insitu(RapidJsonReaderHandle* r,
                                   RapidJsonInsituStreamHandle* ss,
                                   const RapidJsonHandlerCallbacks* cb) {
    CCallbackHandler handler{cb};
    return r->reader.Parse(ss->ss, handler) ? 1 : 0;
}

int rapidjson_reader_iterate_next(RapidJsonReaderHandle* r,
                                   RapidJsonStringStreamHandle* ss,
                                   const RapidJsonHandlerCallbacks* cb) {
    CCallbackHandler handler{cb};
    // IterativeParseNext advances one token; returns false when done.
    if (!r->reader.IterativeParseNext<kParseDefaultFlags>(ss->ss, handler))
        return 0;
    return r->reader.IterativeParseComplete() ? 0 : 1;
}

int rapidjson_reader_has_parse_error(const RapidJsonReaderHandle* r) {
    return r->reader.HasParseError() ? 1 : 0;
}
int rapidjson_reader_get_parse_error_code(const RapidJsonReaderHandle* r) {
    return (int)r->reader.GetParseErrorCode();
}
size_t rapidjson_reader_get_error_offset(const RapidJsonReaderHandle* r) {
    return (size_t)r->reader.GetErrorOffset();
}
