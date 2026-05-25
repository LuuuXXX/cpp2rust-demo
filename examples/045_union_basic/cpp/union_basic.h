#pragma once

#include <cstddef>
#include <cstdint>

#ifdef __cplusplus
extern "C" {
#endif

// Value type constants
static const int VALUE_TYPE_INT = 0;
static const int VALUE_TYPE_FLOAT = 1;
static const int VALUE_TYPE_STRING = 2;

// Forward declaration
struct Variant;

// Factory functions
struct Variant* variant_new_int(int value);
struct Variant* variant_new_float(float value);
struct Variant* variant_new_string(const char* value);
void variant_delete(struct Variant* self);

// Accessors
int variant_get_type(const struct Variant* self);
int variant_get_int(const struct Variant* self);
float variant_get_float(const struct Variant* self);
const char* variant_get_string(const struct Variant* self);

// Mutators
void variant_set_int(struct Variant* self, int value);
void variant_set_float(struct Variant* self, float value);
void variant_set_string(struct Variant* self, const char* value);

// IntFloatUnion for demonstrating memory overlay
struct IntFloatUnion {
    union {
        int int_value;
        float float_value;
    } data;
};

int union_get_int(const struct IntFloatUnion* u);
float union_get_float(const struct IntFloatUnion* u);
void union_set_int(struct IntFloatUnion* u, int value);
void union_set_float(struct IntFloatUnion* u, float value);

#ifdef __cplusplus
}

// Full class definition - for hicc code generation
class Variant {
public:
    using StringBuffer = char[64];
private:
    int type_;
    union {
        int int_value_;
        float float_value_;
        char string_buffer_[64];
    } data_;
public:
    Variant();
    ~Variant();
    [[nodiscard]] int get_type() const;
    void set_int(int value);
    void set_float(float value);
    void set_string(const char* value);
    [[nodiscard]] int get_int() const;
    [[nodiscard]] float get_float() const;
    [[nodiscard]] const char* get_string() const;
};

#endif
