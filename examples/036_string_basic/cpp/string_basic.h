#pragma once

#include <algorithm>
#include <cctype>
#include <cstddef>
#include <string>

namespace string_basic_ns {

// MyString：直接持有 std::string，演示基本字符串操作。
// hicc 直出无需 extern-C 不透明指针 + *_delete，析构由 Rust Drop 自动完成。
class MyString {
    std::string data_;
public:
    explicit MyString(const char* s) : data_(s ? s : "") {}

    int length() const { return static_cast<int>(data_.length()); }
    int empty() const { return data_.empty() ? 1 : 0; }
    void append(const char* s) { if (s) data_ += s; }
    char at(int i) const {
        return (i >= 0 && static_cast<std::size_t>(i) < data_.size()) ? data_[i] : 0;
    }
    const char* c_str() const { return data_.c_str(); }
    int compare(const char* other) const { return data_.compare(other ? other : ""); }
    void to_upper() {
        std::transform(data_.begin(), data_.end(), data_.begin(),
                       [](unsigned char c) { return static_cast<char>(std::toupper(c)); });
    }
    int find(const char* sub) const {
        if (!sub) return -1;
        std::size_t pos = data_.find(sub);
        return pos == std::string::npos ? -1 : static_cast<int>(pos);
    }
};

// 锚点：本单元可链接的非模板符号。
int string_basic_anchor();

} // namespace string_basic_ns
