#pragma once

#include <cstring>
#include <string>

namespace union_basic_ns {

// Variant：tagged union，按 type_ 记录当前活跃成员。
class Variant {
    int type_;                  // 0=int, 1=float, 2=string
    union { int i_; float f_; char s_[64]; };
    std::string sbuf_;          // get_string() 返回 c_str() 的稳定 backing
public:
    Variant();

    void set_int(int v);
    void set_float(float v);
    void set_string(const char* v);

    int get_type() const;
    int get_int() const;
    float get_float() const;
    const char* get_string() const;
};

// IntFloatUnion：同一块内存用 int / float 两种视图读取。
class IntFloatUnion {
    union { int i_; float f_; };
public:
    IntFloatUnion();

    void set_int(int v);
    void set_float(float v);

    int get_int() const;
    float get_float() const;
};

// 锚点：本单元可链接的非模板符号。
int union_basic_anchor();

} // namespace union_basic_ns
