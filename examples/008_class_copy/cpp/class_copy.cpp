#include "class_copy.h"
#include <iostream>
#include <cstring>

// Buffer class implementations
Buffer::Buffer() : data(nullptr), size(0) {}

Buffer::Buffer(int sz) : size(sz) {
    data = new int[sz];
    std::memset(data, 0, sz * sizeof(int));
}

Buffer::Buffer(const Buffer& other) : size(other.size) {
    data = new int[other.size];
    std::memcpy(data, other.data, other.size * sizeof(int));
}

Buffer::~Buffer() {
    delete[] data;
}

void Buffer::set(int index, int value) {
    if (index >= 0 && index < size) {
        data[index] = value;
    }
}

int Buffer::get(int index) const {
    if (index >= 0 && index < size) {
        return data[index];
    }
    return 0;
}

int Buffer::getSize() const {
    return size;
}

// FFI wrapper functions
struct Buffer* buffer_new(void) {
    return new Buffer();
}

struct Buffer* buffer_newWithSize(int size) {
    return new Buffer(size);
}

struct Buffer* buffer_newCopy(const struct Buffer* other) {
    return new Buffer(*other);
}

void buffer_delete(struct Buffer* self) {
    delete self;
}

void buffer_set(struct Buffer* self, int index, int value) {
    self->set(index, value);
}

int buffer_get(const struct Buffer* self, int index) {
    return self->get(index);
}

int buffer_size(const struct Buffer* self) {
    return self->getSize();
}
