#pragma once

#include <array>
#include <algorithm>

namespace array_basic_ns {

// IntArray：直接持有固定大小 std::array<int, 8>，演示基本数组操作。
// hicc 直出无需 extern-C 不透明指针 + *_delete，析构由 Rust Drop 自动完成。
class IntArray {
    std::array<int, 8> data_;
public:
    IntArray() : data_{} {}

    int size() const { return 8; }
    void set(int i, int v) {
        if (i >= 0 && static_cast<std::size_t>(i) < data_.size()) data_[i] = v;
    }
    int get(int i) const {
        return (i >= 0 && static_cast<std::size_t>(i) < data_.size()) ? data_[i] : 0;
    }
    void fill(int v) { data_.fill(v); }

    int sum() const {
        int s = 0;
        for (int x : data_) s += x;
        return s;
    }
    int max() const { return *std::max_element(data_.begin(), data_.end()); }
    int min() const { return *std::min_element(data_.begin(), data_.end()); }
};

// 锚点：本单元可链接的非模板符号。
int array_basic_anchor();

} // namespace array_basic_ns
