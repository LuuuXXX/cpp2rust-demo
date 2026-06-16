#pragma once

#include <vector>
#include <new>
#include <cstddef>

namespace placement_new_ns {

// 用于 placement new 演示的简单 POD（平凡可析构）。
struct SimpleValue {
    int value;
};

// Buffer：持有一段原始存储，演示在预分配内存的指定偏移处用 placement new 构造对象。
// hicc 直出无需手写 *_delete，存储由 Rust Drop 自动回收。
class Buffer {
    std::vector<char> storage_;
    std::size_t constructed_size_;
public:
    explicit Buffer(int capacity)
        : storage_(capacity > 0 ? static_cast<std::size_t>(capacity) : 0, 0),
          constructed_size_(0) {}

    int capacity() const { return static_cast<int>(storage_.size()); }
    int size() const { return static_cast<int>(constructed_size_); }

    // 在 offset 处用 placement new 构造 SimpleValue(v)，返回读回的值（越界返回 -1）。
    int construct_at(int offset, int v) {
        if (offset < 0 ||
            static_cast<std::size_t>(offset) + sizeof(SimpleValue) > storage_.size()) {
            return -1;
        }
        SimpleValue* p = new (storage_.data() + offset) SimpleValue{v};
        constructed_size_ = static_cast<std::size_t>(offset) + sizeof(SimpleValue);
        return p->value;
    }

    // 读回 offset 处已构造对象的值（越界返回 -1）。
    int value_at(int offset) const {
        if (offset < 0 ||
            static_cast<std::size_t>(offset) + sizeof(SimpleValue) > storage_.size()) {
            return -1;
        }
        const SimpleValue* p =
            reinterpret_cast<const SimpleValue*>(storage_.data() + offset);
        return p->value;
    }
};

// ObjectArray：以「元素槽位」的方式在连续存储中逐个 placement new 构造对象，
// 模拟 std::vector 的底层内存管理风格。
class ObjectArray {
    std::vector<char> storage_;
    int count_;
public:
    explicit ObjectArray(int count)
        : storage_((count > 0 ? count : 0) * sizeof(SimpleValue), 0),
          count_(count > 0 ? count : 0) {}

    int count() const { return count_; }
    int element_size() const { return static_cast<int>(sizeof(SimpleValue)); }

    // 在第 i 个槽位用 placement new 构造 SimpleValue(v)，返回读回的值（越界返回 -1）。
    int emplace(int i, int v) {
        if (i < 0 || i >= count_) return -1;
        SimpleValue* p =
            new (storage_.data() + i * sizeof(SimpleValue)) SimpleValue{v};
        return p->value;
    }

    int at(int i) const {
        if (i < 0 || i >= count_) return -1;
        const SimpleValue* p = reinterpret_cast<const SimpleValue*>(
            storage_.data() + i * sizeof(SimpleValue));
        return p->value;
    }
};

// 锚点：本单元可链接的非模板符号。
int placement_new_anchor();

} // namespace placement_new_ns
