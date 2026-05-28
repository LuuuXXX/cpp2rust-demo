#pragma once

#ifdef __cplusplus
extern "C" {
#endif

// std::map 基本操作示例

#include <stddef.h>

// Forward declarations (opaque pointers)
struct StringIntMap;
struct IntStringMap;

// StringIntMap operations
struct StringIntMap* string_int_map_new(void);
void string_int_map_delete(struct StringIntMap* self);

size_t string_int_map_size(const struct StringIntMap* self);
int string_int_map_empty(const struct StringIntMap* self);

// Insert key-value pair (returns 1 on success, 0 if key exists)
int string_int_map_insert(struct StringIntMap* self, const char* key, int value);

// Find value (returns 1 if found, 0 if not)
int string_int_map_find(struct StringIntMap* self, const char* key, int* out_value);

// Access element (inserts default if not exists)
int string_int_map_get(const struct StringIntMap* self, const char* key);
void string_int_map_set(struct StringIntMap* self, const char* key, int value);

// Erase key-value pair (returns 1 on success, 0 if key not found)
int string_int_map_erase(struct StringIntMap* self, const char* key);

// Clear all elements
void string_int_map_clear(struct StringIntMap* self);

// Contains check
int string_int_map_contains(const struct StringIntMap* self, const char* key);

// IntStringMap operations
struct IntStringMap* int_string_map_new(void);
void int_string_map_delete(struct IntStringMap* self);

size_t int_string_map_size(const struct IntStringMap* self);

int int_string_map_insert_int(struct IntStringMap* self, int key, const char* value);
int int_string_map_find_int(struct IntStringMap* self, int key, const char** out_value);
int int_string_map_erase_int(struct IntStringMap* self, int key);

#ifdef __cplusplus
}

// Full class definition - for hicc code generation
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

#endif
