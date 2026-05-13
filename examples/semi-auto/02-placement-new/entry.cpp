// entry.cpp — semi-auto/02-placement-new
//
// Compiled with clang to produce the capture file consumed by
// `cpp2rust-demo init`.

#include "fixed_buffer.hpp"
#include <cstring>
#include <algorithm>

FixedBuffer::FixedBuffer(int capacity)
    : buf_(new char[capacity]), capacity_(capacity), used_(0) {}

FixedBuffer::~FixedBuffer() { delete[] buf_; }

int FixedBuffer::write(const char* data, int size) {
    int n = std::min(size, capacity_ - used_);
    std::memcpy(buf_ + used_, data, n);
    used_ += n;
    return n;
}

const char* FixedBuffer::data()     const { return buf_; }
int         FixedBuffer::size()     const { return used_; }
int         FixedBuffer::capacity() const { return capacity_; }
void        FixedBuffer::reset()          { used_ = 0; }
