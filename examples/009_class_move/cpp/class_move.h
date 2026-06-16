#pragma once
#include <cstring>
#include <iostream>
#include <utility>

namespace class_move_ns {

// 拥有动态数组的「只移动」向量，演示移动语义：移动构造/移动赋值「窃取」源对象的
// 内存指针并把源置空，因此移动是 O(1) 资源转移而非深拷贝。析构释放内存
// （交由 hicc 的 Drop 调用）。
class UniqueVector {
public:
    UniqueVector() : data_(nullptr), size_(0) {
        std::cout << "UniqueVector() ctor" << std::endl;
    }
    explicit UniqueVector(int size) : data_(new int[size]()), size_(size) {
        std::cout << "UniqueVector(int) ctor, size=" << size << std::endl;
    }
    // 移动构造：窃取 other 的指针并把 other 置空。
    UniqueVector(UniqueVector&& other) noexcept
        : data_(other.data_), size_(other.size_) {
        other.data_ = nullptr;
        other.size_ = 0;
        std::cout << "UniqueVector(UniqueVector&&) move ctor" << std::endl;
    }
    // 移动赋值：释放自身内存，窃取 other 的指针并把 other 置空。
    UniqueVector& operator=(UniqueVector&& other) noexcept {
        if (this != &other) {
            delete[] data_;
            data_ = other.data_;
            size_ = other.size_;
            other.data_ = nullptr;
            other.size_ = 0;
        }
        return *this;
    }
    ~UniqueVector() {
        delete[] data_;
        std::cout << "~UniqueVector() dtor, size=" << size_ << std::endl;
    }

    void set(int index, int value) {
        if (index >= 0 && index < size_) {
            data_[index] = value;
        }
    }
    int get(int index) const {
        if (index >= 0 && index < size_) {
            return data_[index];
        }
        return 0;
    }
    int size() const { return size_; }

    // 从 src 移动资源到自身（src 被置空），演示成员级移动操作。
    void move_from(UniqueVector& src) {
        *this = std::move(src);
    }

private:
    int* data_;
    int size_;
};

} // namespace class_move_ns
