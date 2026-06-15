#include "union_basic.h"
#include <iostream>
#include <cstring>

// Variant implementation
Variant::Variant() : type_(VALUE_TYPE_INT) {
    data_.int_value_ = 0;
}
Variant::~Variant() {}
int Variant::get_type() const {
    return type_;
}
void Variant::set_int(int value) {
    type_ = VALUE_TYPE_INT;
    data_.int_value_ = value;
}
void Variant::set_float(float value) {
    type_ = VALUE_TYPE_FLOAT;
    data_.float_value_ = value;
}
void Variant::set_string(const char* value) {
    type_ = VALUE_TYPE_STRING;
    if (value) {
        strncpy(data_.string_buffer_, value, 63);
        data_.string_buffer_[63] = '\0';
    } else {
        data_.string_buffer_[0] = '\0';
    }
}
int Variant::get_int() const {
    if (type_ == VALUE_TYPE_INT) {
        return data_.int_value_;
    }
    return 0;
}
float Variant::get_float() const {
    if (type_ == VALUE_TYPE_FLOAT) {
        return data_.float_value_;
    }
    return 0.0f;
}
const char* Variant::get_string() const {
    if (type_ == VALUE_TYPE_STRING) {
        return data_.string_buffer_;
    }
    return "";
}
