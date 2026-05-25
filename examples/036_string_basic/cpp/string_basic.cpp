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

// FFI wrapper functions
struct String* string_new(void) {
    return new String();
}

struct String* string_new_from(const char* str) {
    return new String(str);
}

struct String* string_new_from_len(const char* str, size_t len) {
    return new String(str, len);
}

void string_delete(struct String* self) {
    delete self;
}

size_t string_size(const struct String* self) {
    return self->impl->data.size();
}

size_t string_length(const struct String* self) {
    return self->impl->data.length();
}

int string_empty(const struct String* self) {
    return self->impl->data.empty() ? 1 : 0;
}

const char* string_c_str(const struct String* self) {
    return self->impl->data.c_str();
}

int string_compare(const struct String* self, const char* other) {
    if (!other) return 1;
    return self->impl->data.compare(other);
}

int string_equals(const struct String* self, const char* other) {
    if (!other) return self->impl->data.empty() ? 1 : 0;
    return self->impl->data == other ? 1 : 0;
}

int string_starts_with(const struct String* self, const char* prefix) {
    if (!prefix) return 0;
    return self->impl->data.find(prefix) == 0 ? 1 : 0;
}

int string_ends_with(const struct String* self, const char* suffix) {
    if (!suffix) return 0;
    size_t suffix_len = std::strlen(suffix);
    if (self->impl->data.length() < suffix_len) return 0;
    return self->impl->data.compare(self->impl->data.length() - suffix_len, suffix_len, suffix) == 0 ? 1 : 0;
}

struct String* string_concat(const struct String* self, const char* other) {
    std::string result = self->impl->data + (other ? other : "");
    return new String(result.c_str());
}

void string_append(struct String* self, const char* other) {
    if (other) {
        self->impl->data += other;
    }
}

void string_append_len(struct String* self, const char* other, size_t len) {
    if (other) {
        self->impl->data.append(other, len);
    }
}

void string_clear(struct String* self) {
    self->impl->data.clear();
}

void string_trim(struct String* self) {
    size_t start = self->impl->data.find_first_not_of(" \t\n\r");
    size_t end = self->impl->data.find_last_not_of(" \t\n\r");
    if (start == std::string::npos) {
        self->impl->data.clear();
    } else {
        self->impl->data = self->impl->data.substr(start, end - start + 1);
    }
}

struct String* string_substr(const struct String* self, size_t pos, size_t len) {
    if (pos >= self->impl->data.length()) {
        return new String();
    }
    return new String(self->impl->data.substr(pos, len).c_str());
}

int string_find(const struct String* self, const char* substr) {
    if (!substr) return -1;
    size_t pos = self->impl->data.find(substr);
    return pos == std::string::npos ? -1 : static_cast<int>(pos);
}

void string_to_upper(struct String* self) {
    std::transform(self->impl->data.begin(), self->impl->data.end(), self->impl->data.begin(),
                   ::toupper);
}

void string_to_lower(struct String* self) {
    std::transform(self->impl->data.begin(), self->impl->data.end(), self->impl->data.begin(),
                   ::tolower);
}
