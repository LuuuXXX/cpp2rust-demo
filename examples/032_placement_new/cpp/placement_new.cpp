#include "placement_new.h"
#include <iostream>
#include <cstring>
#include <new>

Buffer::Buffer(size_t capacity) : data_(nullptr), capacity_(capacity), constructed_size_(0) {
    if (capacity_ > 0) {
        data_ = new char[capacity_];
        std::memset(data_, 0, capacity_);
    }
}

Buffer::~Buffer() {
    if (data_) {
        delete[] data_;
        data_ = nullptr;
    }
}

void* Buffer::data() {
    return static_cast<void*>(data_);
}

size_t Buffer::capacity() const {
    return capacity_;
}

size_t Buffer::size() const {
    return constructed_size_;
}

void* Buffer::construct(size_t offset) {
    if (offset < capacity_) {
        constructed_size_ = offset + sizeof(SimpleValue);
        return static_cast<void*>(data_ + offset);
    }
    return nullptr;
}

VectorBuffer::VectorBuffer(size_t capacity, size_t elem_size)
    : data_(nullptr), capacity_(capacity), size_(0), element_size_(elem_size) {
    if (capacity_ > 0) {
        data_ = new char[capacity_ * element_size_];
        std::memset(data_, 0, capacity_ * element_size_);
    }
}

VectorBuffer::~VectorBuffer() {
    destroy_all();
    if (data_) {
        delete[] data_;
        data_ = nullptr;
    }
}

void* VectorBuffer::data() {
    return static_cast<void*>(data_);
}

size_t VectorBuffer::element_size() const {
    return element_size_;
}

void VectorBuffer::destroy_all() {
    size_ = 0;
    if (data_) {
        std::memset(data_, 0, capacity_ * element_size_);
    }
}
