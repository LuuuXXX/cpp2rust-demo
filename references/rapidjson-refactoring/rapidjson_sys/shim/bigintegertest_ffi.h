// FFI shim header for test/unittest/bigintegertest.cpp
//
// This header exposes a minimal C ABI around rapidjson::internal::BigInteger
// sufficient to support the L1/L2 mirror tests. The goal is to mirror the
// behaviour tested in bigintegertest.cpp while keeping the surface area
// small and stable.

#pragma once

#ifdef __cplusplus
extern "C" {
#endif

// Opaque handle for a BigInteger used in tests
typedef struct RapidJsonBigIntegerHandle RapidJsonBigIntegerHandle;

// Lifecycle
RapidJsonBigIntegerHandle* rapidjson_biginteger_new(void);
void rapidjson_biginteger_free(RapidJsonBigIntegerHandle* handle);

// Assigns this handle from a decimal literal, e.g. "0", "1", "123".
// Returns 0 on failure (invalid literal), non-zero on success.
int rapidjson_biginteger_from_decimal_literal(
    RapidJsonBigIntegerHandle* handle,
    const char* literal
);

// Adds a 64-bit unsigned integer to the BigInteger in-place (+= rhs).
void rapidjson_biginteger_add_u64(
    RapidJsonBigIntegerHandle* handle,
    unsigned long long value
);

// Multiplies the BigInteger by a 64-bit unsigned integer in-place (*= rhs).
void rapidjson_biginteger_mul_u64(
    RapidJsonBigIntegerHandle* handle,
    unsigned long long value
);

// Multiplies the BigInteger by a 32-bit unsigned integer in-place (*= rhs).
void rapidjson_biginteger_mul_u32(
    RapidJsonBigIntegerHandle* handle,
    unsigned int value
);

// Left-shifts the BigInteger by the given number of bits in-place (<<= shift).
void rapidjson_biginteger_shl(
    RapidJsonBigIntegerHandle* handle,
    unsigned int shift
);

// Compares two BigInteger handles.
// Returns <0 if a < b, 0 if a == b, >0 if a > b.
int rapidjson_biginteger_compare(
    const RapidJsonBigIntegerHandle* a,
    const RapidJsonBigIntegerHandle* b
);

// Serializes the BigInteger to a decimal string into the provided buffer.
// Returns 0 on failure, non-zero on success.
int rapidjson_biginteger_to_string(
    const RapidJsonBigIntegerHandle* handle,
    char* out,
    unsigned long out_capacity
);

#ifdef __cplusplus
} // extern "C"
#endif
