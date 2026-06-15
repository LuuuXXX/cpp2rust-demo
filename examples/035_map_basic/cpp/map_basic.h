#pragma once

#include <map>
#include <string>

class StringIntMapImpl {
public:
    std::map<std::string, int> data;
    StringIntMapImpl();
    ~StringIntMapImpl();
};

class IntStringMapImpl {
public:
    std::map<int, std::string> data;
    IntStringMapImpl();
    ~IntStringMapImpl();
};

struct StringIntMap {
    StringIntMapImpl* impl;
    explicit StringIntMap();
    ~StringIntMap();
    bool insert(const char* key, int val) { return impl->data.insert({key, val}).second; }
    int get(const char* key) const { return impl->data.count(key) ? impl->data.at(key) : 0; }
    void set(const char* key, int val) { impl->data[key] = val; }
    bool erase(const char* key) { return impl->data.erase(key) > 0; }
    size_t size() const { return impl->data.size(); }
    bool empty() const { return impl->data.empty(); }
    void clear() { impl->data.clear(); }
};

struct IntStringMap {
    IntStringMapImpl* impl;
    explicit IntStringMap();
    ~IntStringMap();
    size_t size() const { return impl->data.size(); }
};
