#include "unique_ptr.h"
#include <iostream>
#include <memory>
#include <cstring>

UniqueBuffer::UniqueBuffer(int sz) : data(sz, '\0') {
}

UniqueBuffer::~UniqueBuffer() {
}

int UniqueBuffer::getSize() const {
    return static_cast<int>(data.size());
}

char* UniqueBuffer::getData() {
    return &data[0];
}

UniqueBuffer UniqueBuffer::move() {
    return UniqueBuffer(*this);
}

int UniqueBuffer::useCount() const {
    return 1;
}

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
