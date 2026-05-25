// FFI shim implementation for test/unittest/bigintegertest.cpp
//
// This bridges rapidjson::internal::BigInteger to a small C ABI used
// by the Rust L1/L2 mirror tests.

#include "bigintegertest_ffi.h"

#include <new>
#include <cstring>

#include "rapidjson/internal/biginteger.h"

using namespace rapidjson::internal;

struct RapidJsonBigIntegerHandle {
    BigInteger value;

    RapidJsonBigIntegerHandle()
        : value(0) {
    }
};

RapidJsonBigIntegerHandle* rapidjson_biginteger_new(void) {
    return new (std::nothrow) RapidJsonBigIntegerHandle();
}

void rapidjson_biginteger_free(RapidJsonBigIntegerHandle* handle) {
    delete handle;
}

int rapidjson_biginteger_from_decimal_literal(
    RapidJsonBigIntegerHandle* handle,
    const char* literal
) {
    if (!handle || !literal)
        return 0;

    // BigInteger(const char* buffer, size_t length)
    handle->value = BigInteger(literal, std::strlen(literal));
    return 1;
}

void rapidjson_biginteger_add_u64(
    RapidJsonBigIntegerHandle* handle,
    unsigned long long value
) {
    if (!handle)
        return;
    handle->value += static_cast<uint64_t>(value);
}

void rapidjson_biginteger_mul_u64(
    RapidJsonBigIntegerHandle* handle,
    unsigned long long value
) {
    if (!handle)
        return;
    handle->value *= static_cast<uint64_t>(value);
}

void rapidjson_biginteger_mul_u32(
    RapidJsonBigIntegerHandle* handle,
    unsigned int value
) {
    if (!handle)
        return;
    handle->value *= static_cast<uint32_t>(value);
}

void rapidjson_biginteger_shl(
    RapidJsonBigIntegerHandle* handle,
    unsigned int shift
) {
    if (!handle)
        return;
    handle->value <<= shift;
}

int rapidjson_biginteger_compare(
    const RapidJsonBigIntegerHandle* a,
    const RapidJsonBigIntegerHandle* b
) {
    if (!a || !b)
        return 0;
    return a->value.Compare(b->value);
}

int rapidjson_biginteger_to_string(
    const RapidJsonBigIntegerHandle* handle,
    char* out,
    unsigned long out_capacity
) {
    if (!handle || !out || out_capacity == 0)
        return 0;

    // Placeholder: decimal formatting not yet implemented. The
    // current L1/L2 mirror tests do not rely on this function.
    (void)out;
    (void)out_capacity;
    (void)handle;
    return 0;
}
