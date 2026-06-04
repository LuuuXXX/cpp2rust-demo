#include "unique_ptr.h"
#include <iostream>
#include <memory>
#include <cstring>

// UniqueBuffer class implementation
UniqueBuffer::UniqueBuffer(int sz) : data(sz, '\0') {
}

UniqueBuffer::~UniqueBuffer() {
}

int UniqueBuffer::getSize() const {
    return static_cast<int>(data.size());
}

char* UniqueBuffer::getData() {
    return const_cast<char*>(data.data());
}

UniqueBuffer UniqueBuffer::move() {
    return UniqueBuffer(*this);
}

int UniqueBuffer::useCount() const {
    return 1;  // unique_ptr always has use count of 1
}

// Processor class implementation
Processor::Processor() : buffer() {
}

Processor::~Processor() {
}

char* Processor::process(const char* input) {
    if (input) {
        buffer = std::string(input) + " [processed]";
    }
    return const_cast<char*>(buffer.c_str());
}

// FFI wrapper functions
UniqueBuffer* uniquebuffer_new(int size) {
    return new UniqueBuffer(size);
}

void uniquebuffer_delete(UniqueBuffer* self) {
    delete self;
}

int uniquebuffer_size(UniqueBuffer* self) {
    return self->getSize();
}

char* uniquebuffer_data(UniqueBuffer* self) {
    return self->getData();
}

UniqueBuffer* uniquebuffer_move(UniqueBuffer* self) {
    return new UniqueBuffer(self->move());
}

int uniquebuffer_use_count(UniqueBuffer* self) {
    return self->useCount();
}

Processor* processor_new(void) {
    return new Processor();
}

void processor_delete(Processor* self) {
    delete self;
}

char* processor_process(Processor* self, const char* input) {
    return self->process(input);
}
