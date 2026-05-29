#include "shared_ptr.h"
#include <iostream>
#include <memory>
#include <cstring>
#include <unordered_map>

// SharedData class implementation
SharedData::SharedData(const char* n) : name_(n ? n : ""), value(0) {
}

SharedData::~SharedData() {
}

int SharedData::useCount() const {
    return 1;  // Simplified - actual shared_ptr would have ref count
}

const char* SharedData::getName() const {
    return name_.c_str();
}

SharedData* SharedData::clone() const {
    return new SharedData(name_.c_str());
}

void SharedData::reset() {
    name_.clear();
}

// Cache class implementation
Cache::Cache() : data_() {
}

Cache::~Cache() {
}

SharedData* Cache::get(const char* name) {
    if (!name) return nullptr;
    std::string key(name);
    auto it = data_.find(key);
    if (it != data_.end()) {
        return reinterpret_cast<SharedData*>(it->second);
    }
    // If not found, create new and store
    SharedData* new_data = new SharedData(name);
    data_[key] = reinterpret_cast<void*>(new_data);
    return new_data;
}

// FFI wrapper functions
SharedData* shareddata_new(const char* name) {
    return new SharedData(name);
}

void shareddata_delete(SharedData* self) {
    delete self;
}

int shareddata_use_count(SharedData* self) {
    return self ? self->useCount() : 0;
}

const char* shareddata_getName(SharedData* self) {
    return self ? self->getName() : "";
}

SharedData* shareddata_clone(SharedData* self) {
    return self ? self->clone() : nullptr;
}

void shareddata_reset(SharedData* self) {
    if (self) self->reset();
}

int shareddata_expired(SharedData* self) {
    return self == nullptr || self->useCount() == 0;
}

Cache* cache_new(void) {
    return new Cache();
}

void cache_delete(Cache* self) {
    delete self;
}

SharedData* cache_get(Cache* c, const char* name) {
    return c ? c->get(name) : nullptr;
}
