#include "map_basic.h"
#include <iostream>
#include <map>
#include <string>
#include <cstring>

// StringIntMapImpl class implementation
StringIntMapImpl::StringIntMapImpl() : data() {
}

StringIntMapImpl::~StringIntMapImpl() {
    data.clear();
}

// IntStringMapImpl class implementation
IntStringMapImpl::IntStringMapImpl() : data() {
}

IntStringMapImpl::~IntStringMapImpl() {
    data.clear();
}

// StringIntMap struct implementation
StringIntMap::StringIntMap() : impl(new StringIntMapImpl()) {
}

StringIntMap::~StringIntMap() {
    delete impl;
    impl = nullptr;
}

// IntStringMap struct implementation
IntStringMap::IntStringMap() : impl(new IntStringMapImpl()) {
}

IntStringMap::~IntStringMap() {
    delete impl;
    impl = nullptr;
}
