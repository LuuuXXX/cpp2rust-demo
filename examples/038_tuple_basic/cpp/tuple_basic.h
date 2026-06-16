#pragma once

#include <tuple>
#include <string>

namespace tuple_basic_ns {

// Record：直接持有 std::tuple<int, double, std::string>，演示基本元素访问。
// hicc 直出无需 extern-C 不透明指针 + *_delete，析构由 Rust Drop 自动完成。
class Record {
    std::tuple<int, double, std::string> data_;
public:
    Record(int id, double score, const char* name) : data_(id, score, name ? name : "") {}

    int id() const { return std::get<0>(data_); }
    double score() const { return std::get<1>(data_); }
    const char* name() const { return std::get<2>(data_).c_str(); }

    void set_id(int id) { std::get<0>(data_) = id; }
    void set_score(double score) { std::get<1>(data_) = score; }
};

// 锚点：本单元可链接的非模板符号。
int tuple_basic_anchor();

} // namespace tuple_basic_ns
