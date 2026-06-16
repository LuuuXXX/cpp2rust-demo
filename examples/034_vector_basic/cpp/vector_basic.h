#pragma once

#include <vector>
#include <string>

namespace vector_basic_ns {

// IntVector：直接持有 std::vector<int>，演示基本容器操作。
// hicc 直出无需 extern-C 不透明指针 + *_delete，析构由 Rust Drop 自动完成。
class IntVector {
    std::vector<int> data_;
public:
    IntVector() = default;

    int size() const { return static_cast<int>(data_.size()); }
    int capacity() const { return static_cast<int>(data_.capacity()); }
    int empty() const { return data_.empty() ? 1 : 0; }

    void reserve(int n) { if (n > 0) data_.reserve(static_cast<std::size_t>(n)); }
    void push_back(int v) { data_.push_back(v); }
    void pop_back() { if (!data_.empty()) data_.pop_back(); }

    int get(int i) const {
        return (i >= 0 && static_cast<std::size_t>(i) < data_.size()) ? data_[i] : 0;
    }
    void set(int i, int v) {
        if (i >= 0 && static_cast<std::size_t>(i) < data_.size()) data_[i] = v;
    }

    int sum() const {
        int s = 0;
        for (int x : data_) s += x;
        return s;
    }
    void clear() { data_.clear(); }
};

// StringVector：直接持有 std::vector<std::string>，get 返回元素的 c_str()。
class StringVector {
    std::vector<std::string> data_;
public:
    StringVector() = default;

    int size() const { return static_cast<int>(data_.size()); }
    void push_back(const char* s) { data_.push_back(s ? s : ""); }
    const char* get(int i) const {
        return (i >= 0 && static_cast<std::size_t>(i) < data_.size()) ? data_[i].c_str() : "";
    }
    void clear() { data_.clear(); }
};

// 锚点：本单元可链接的非模板符号。
int vector_basic_anchor();

} // namespace vector_basic_ns
