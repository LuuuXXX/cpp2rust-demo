#pragma once

#include <cstddef>
#include <cstdint>

enum VariantType { VALUE_TYPE_INT = 0, VALUE_TYPE_FLOAT = 1, VALUE_TYPE_STRING = 2 };

#ifdef __cplusplus

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
