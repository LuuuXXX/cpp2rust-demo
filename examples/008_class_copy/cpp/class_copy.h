#pragma once
#include <cstring>
#include <iostream>

namespace class_copy_ns {

// 拥有动态数组的缓冲区，演示「深拷贝」：拷贝构造分配新内存并复制内容，
// 因此拷贝体与原对象相互独立。析构释放内存（交由 hicc 的 Drop 调用）。
class Buffer {
public:
    Buffer() : data_(nullptr), size_(0) {
        std::cout << "Buffer() ctor" << std::endl;
    }
    explicit Buffer(int sz) : data_(new int[sz]()), size_(sz) {
        std::cout << "Buffer(int) ctor, size=" << sz << std::endl;
    }
    // 深拷贝构造：分配新内存并逐字节复制 other 的内容。
    Buffer(const Buffer& other) : data_(new int[other.size_]), size_(other.size_) {
        std::memcpy(data_, other.data_, other.size_ * sizeof(int));
        std::cout << "Buffer(const Buffer&) copy ctor, size=" << size_ << std::endl;
    }
    // 深拷贝赋值（rule of three），保持与拷贝构造一致的独立内存语义。
    Buffer& operator=(const Buffer& other) {
        if (this != &other) {
            int* fresh = new int[other.size_];
            std::memcpy(fresh, other.data_, other.size_ * sizeof(int));
            delete[] data_;
            data_ = fresh;
            size_ = other.size_;
        }
        return *this;
    }
    ~Buffer() {
        delete[] data_;
        std::cout << "~Buffer() dtor, size=" << size_ << std::endl;
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

private:
    int* data_;
    int size_;
};

} // namespace class_copy_ns
