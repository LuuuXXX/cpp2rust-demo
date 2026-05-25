#include "placement_new.h"
#include <iostream>
#include <cstring>
#include <new>

// SimpleValue type needed for vector_buffer_new
struct SimpleValue {
    int value;
};

// Buffer class implementation
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

// VectorBuffer class implementation
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

// FFI wrapper functions
Buffer* buffer_new(size_t capacity) {
    return new Buffer(capacity);
}

void buffer_delete(Buffer* self) {
    if (self) {
        std::cout << "Buffer delete called" << std::endl;
        delete self;
    }
}

void* buffer_data(Buffer* self) {
    return self->data();
}

size_t buffer_capacity(Buffer* self) {
    return self->capacity();
}

void* buffer_construct(Buffer* self, size_t offset) {
    return self->construct(offset);
}

size_t buffer_size(Buffer* self) {
    return self->size();
}

VectorBuffer* vector_buffer_new(size_t capacity) {
    return new VectorBuffer(capacity, sizeof(SimpleValue));
}

void vector_buffer_delete(VectorBuffer* self) {
    if (self) {
        self->destroy_all();
        delete self;
    }
}

void* vector_buffer_data(VectorBuffer* self) {
    return self->data();
}

size_t vector_buffer_element_size(VectorBuffer* self) {
    return self->element_size();
}
