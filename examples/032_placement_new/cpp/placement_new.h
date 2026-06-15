#pragma once

#include <stddef.h>

struct SimpleValue {
    int value;
};

class Buffer {
    char* data_;
    size_t capacity_;
    size_t constructed_size_;
public:
    explicit Buffer(size_t capacity);
    ~Buffer();
    Buffer(const Buffer&) = delete;
    Buffer& operator=(const Buffer&) = delete;
    void* data();
    size_t capacity() const;
    size_t size() const;
    void* construct(size_t offset);
};

class VectorBuffer {
    char* data_;
    size_t capacity_;
    size_t size_;
    size_t element_size_;
public:
    explicit VectorBuffer(size_t capacity, size_t elem_size);
    ~VectorBuffer();
    VectorBuffer(const VectorBuffer&) = delete;
    VectorBuffer& operator=(const VectorBuffer&) = delete;
    void* data();
    size_t element_size() const;
    void destroy_all();
};
