#include "string_basic.h"
#include <iostream>
#include <string>
#include <cstring>
#include <algorithm>
#include <cctype>

// StringImpl class implementation
StringImpl::StringImpl() : data() {
}

StringImpl::StringImpl(const char* str) : data(str ? str : "") {
}

StringImpl::StringImpl(const char* str, size_t len) : data(str ? std::string(str, len) : "") {
}

StringImpl::~StringImpl() {
    data.clear();
}

// String struct implementation
String::String() : impl(new StringImpl()) {
}

String::String(const char* str) : impl(new StringImpl(str)) {
}

String::String(const char* str, size_t len) : impl(new StringImpl(str, len)) {
}

String::~String() {
    delete impl;
    impl = nullptr;
}
