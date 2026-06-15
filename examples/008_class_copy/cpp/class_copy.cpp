#include "class_copy.h"
#include <iostream>
#include <cstring>

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
