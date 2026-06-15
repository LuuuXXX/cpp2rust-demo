#include "class_move.h"
#include <iostream>
#include <cstring>

UniqueVector::UniqueVector() : data(nullptr), size(0) {}

UniqueVector::UniqueVector(int* data, int size) : size(size) {
    this->data = new int[size];
    std::memcpy(this->data, data, size * sizeof(int));
}

UniqueVector::~UniqueVector() {
    delete[] data;
}

UniqueVector::UniqueVector(UniqueVector&& other) noexcept : data(other.data), size(other.size) {
    other.data = nullptr;
    other.size = 0;
}

UniqueVector& UniqueVector::operator=(UniqueVector&& other) noexcept {
    if (this != &other) {
        delete[] data;
        data = other.data;
        size = other.size;
        other.data = nullptr;
        other.size = 0;
    }
    return *this;
}

int UniqueVector::get(int index) const {
    if (index >= 0 && index < size) {
        return data[index];
    }
    return 0;
}

void UniqueVector::set(int index, int value) {
    if (index >= 0 && index < size) {
        data[index] = value;
    }
}

int UniqueVector::getSize() const {
    return size;
}

void UniqueVector::moveFrom(UniqueVector& src) {
    delete[] data;
    data = src.data;
    size = src.size;
    src.data = nullptr;
    src.size = 0;
}
