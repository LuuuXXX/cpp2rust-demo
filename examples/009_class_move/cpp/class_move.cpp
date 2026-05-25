#include "class_move.h"
#include <iostream>
#include <cstring>

// UniqueVector class implementations
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

// FFI wrapper functions
struct UniqueVector* unique_vector_new(void) {
    return new UniqueVector();
}

struct UniqueVector* unique_vector_newWithData(int* data, int size) {
    return new UniqueVector(data, size);
}

void unique_vector_delete(struct UniqueVector* self) {
    delete self;
}

int unique_vector_get(const struct UniqueVector* self, int index) {
    return self->get(index);
}

void unique_vector_set(struct UniqueVector* self, int index, int value) {
    self->set(index, value);
}

int unique_vector_size(const struct UniqueVector* self) {
    return self->getSize();
}

void unique_vector_move(struct UniqueVector* dest, struct UniqueVector* src) {
    std::cout << "Moving UniqueVector: " << src->getSize() << " -> " << dest->getSize() << std::endl;
    dest->moveFrom(*src);
}
