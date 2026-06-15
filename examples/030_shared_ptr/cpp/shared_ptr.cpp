#include "shared_ptr.h"
#include <iostream>
#include <memory>
#include <cstring>
#include <unordered_map>

SharedData::SharedData(const char* n) : name_(n ? n : ""), value(0) {
}

SharedData::~SharedData() {
}

int SharedData::useCount() const {
    return 1;
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
    SharedData* new_data = new SharedData(name);
    data_[key] = reinterpret_cast<void*>(new_data);
    return new_data;
}
