#include "vector_basic.h"
#include <iostream>
#include <vector>
#include <string>
#include <cstring>

// IntVectorImpl class implementation
IntVectorImpl::IntVectorImpl() : data() {
}

IntVectorImpl::~IntVectorImpl() {
    data.clear();
}

// StringVectorImpl class implementation
StringVectorImpl::StringVectorImpl() : data() {
}

StringVectorImpl::~StringVectorImpl() {
    data.clear();
}

// IntVector struct implementation
IntVector::IntVector() : impl(new IntVectorImpl()) {
}

IntVector::~IntVector() {
    delete impl;
    impl = nullptr;
}

// StringVector struct implementation
StringVector::StringVector() : impl(new StringVectorImpl()) {
}

StringVector::~StringVector() {
    delete impl;
    impl = nullptr;
}
