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

// FFI wrapper functions
struct StringIntMap* string_int_map_new(void) {
    return new StringIntMap();
}

void string_int_map_delete(struct StringIntMap* self) {
    delete self;
}

size_t string_int_map_size(const struct StringIntMap* self) {
    return self->impl->data.size();
}

int string_int_map_empty(const struct StringIntMap* self) {
    return self->impl->data.empty() ? 1 : 0;
}

int string_int_map_insert(struct StringIntMap* self, const char* key, int value) {
    if (!key) return 0;
    auto result = self->impl->data.insert({std::string(key), value});
    return result.second ? 1 : 0;
}

int string_int_map_find(struct StringIntMap* self, const char* key, int* out_value) {
    if (!key || !out_value) return 0;
    auto it = self->impl->data.find(std::string(key));
    if (it != self->impl->data.end()) {
        *out_value = it->second;
        return 1;
    }
    return 0;
}

int string_int_map_get(const struct StringIntMap* self, const char* key) {
    if (!key) return 0;
    return self->impl->data[std::string(key)];
}

void string_int_map_set(struct StringIntMap* self, const char* key, int value) {
    if (key) {
        self->impl->data[std::string(key)] = value;
    }
}

int string_int_map_erase(struct StringIntMap* self, const char* key) {
    if (!key) return 0;
    return self->impl->data.erase(std::string(key)) > 0 ? 1 : 0;
}

void string_int_map_clear(struct StringIntMap* self) {
    self->impl->data.clear();
}

int string_int_map_contains(const struct StringIntMap* self, const char* key) {
    if (!key) return 0;
    return self->impl->data.find(std::string(key)) != self->impl->data.end() ? 1 : 0;
}

// IntStringMap C API implementation
struct IntStringMap* int_string_map_new(void) {
    return new IntStringMap();
}

void int_string_map_delete(struct IntStringMap* self) {
    delete self;
}

size_t int_string_map_size(const struct IntStringMap* self) {
    return self->impl->data.size();
}

int int_string_map_insert_int(struct IntStringMap* self, int key, const char* value) {
    if (!value) return 0;
    auto result = self->impl->data.insert({key, std::string(value)});
    return result.second ? 1 : 0;
}

int int_string_map_find_int(struct IntStringMap* self, int key, const char** out_value) {
    static thread_local std::string temp;
    if (!out_value) return 0;
    auto it = self->impl->data.find(key);
    if (it != self->impl->data.end()) {
        temp = it->second;
        *out_value = temp.c_str();
        return 1;
    }
    return 0;
}

int int_string_map_erase_int(struct IntStringMap* self, int key) {
    return self->impl->data.erase(key) > 0 ? 1 : 0;
}
