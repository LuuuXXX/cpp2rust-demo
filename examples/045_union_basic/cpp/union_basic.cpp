#include "union_basic.h"

namespace union_basic_ns {

Variant::Variant() : type_(0), i_(0), sbuf_() {}

void Variant::set_int(int v) {
    type_ = 0;
    i_ = v;
}

void Variant::set_float(float v) {
    type_ = 1;
    f_ = v;
}

void Variant::set_string(const char* v) {
    type_ = 2;
    if (v) {
        std::strncpy(s_, v, 63);
        s_[63] = '\0';
    } else {
        s_[0] = '\0';
    }
    sbuf_ = s_;
}

int Variant::get_type() const { return type_; }
int Variant::get_int() const { return i_; }
float Variant::get_float() const { return f_; }
const char* Variant::get_string() const { return sbuf_.c_str(); }

IntFloatUnion::IntFloatUnion() : i_(0) {}

void IntFloatUnion::set_int(int v) { i_ = v; }
void IntFloatUnion::set_float(float v) { f_ = v; }
int IntFloatUnion::get_int() const { return i_; }
float IntFloatUnion::get_float() const { return f_; }

int union_basic_anchor() { return 0; }

} // namespace union_basic_ns
