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

// FFI factory implementations
struct Variant* variant_new_int(int value) {
    auto* v = new Variant();
    v->set_int(value);
    return v;
}

struct Variant* variant_new_float(float value) {
    auto* v = new Variant();
    v->set_float(value);
    return v;
}

struct Variant* variant_new_string(const char* value) {
    auto* v = new Variant();
    v->set_string(value);
    return v;
}

void variant_delete(struct Variant* self) {
    delete self;
}

int variant_get_type(const struct Variant* self) {
    if (self) return self->get_type();
    return VALUE_TYPE_INT;
}

int variant_get_int(const struct Variant* self) {
    if (self) return self->get_int();
    return 0;
}

float variant_get_float(const struct Variant* self) {
    if (self) return self->get_float();
    return 0.0f;
}

const char* variant_get_string(const struct Variant* self) {
    if (self) return self->get_string();
    return "";
}

void variant_set_int(struct Variant* self, int value) {
    if (self) self->set_int(value);
}

void variant_set_float(struct Variant* self, float value) {
    if (self) self->set_float(value);
}

void variant_set_string(struct Variant* self, const char* value) {
    if (self) self->set_string(value);
}

// IntFloatUnion implementations - demonstrating memory overlay
int union_get_int(const struct IntFloatUnion* u) {
    if (u) return u->data.int_value;
    return 0;
}

float union_get_float(const struct IntFloatUnion* u) {
    if (u) return u->data.float_value;
    return 0.0f;
}

void union_set_int(struct IntFloatUnion* u, int value) {
    if (u) u->data.int_value = value;
}

void union_set_float(struct IntFloatUnion* u, float value) {
    if (u) u->data.float_value = value;
}
