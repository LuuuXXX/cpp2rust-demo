#pragma once

#ifdef __cplusplus
extern "C" {
#endif

// std::vector 基本操作示例

#include <stddef.h>

// Forward declarations (opaque pointers)
struct IntVector;
struct StringVector;

// IntVector operations
struct IntVector* int_vector_new(void);
void int_vector_delete(struct IntVector* self);

size_t int_vector_size(const struct IntVector* self);
size_t int_vector_capacity(const struct IntVector* self);
int int_vector_empty(const struct IntVector* self);

void int_vector_push_back(struct IntVector* self, int value);
void int_vector_pop_back(struct IntVector* self);
int int_vector_get(const struct IntVector* self, size_t index);
void int_vector_set(struct IntVector* self, size_t index, int value);

void int_vector_clear(struct IntVector* self);
int* int_vector_data(struct IntVector* self);

// StringVector operations
struct StringVector* string_vector_new(void);
void string_vector_delete(struct StringVector* self);

size_t string_vector_size(const struct StringVector* self);
size_t string_vector_capacity(const struct StringVector* self);

void string_vector_push_back(struct StringVector* self, const char* value);
const char* string_vector_get(const struct StringVector* self, size_t index);
void string_vector_clear(struct StringVector* self);

#ifdef __cplusplus
}

// Full class definition - for hicc code generation
#include <vector>
#include <string>

class IntVectorImpl {
public:
    std::vector<int> data;
    IntVectorImpl();
    ~IntVectorImpl();
};

class StringVectorImpl {
public:
    std::vector<std::string> data;
    StringVectorImpl();
    ~StringVectorImpl();
};

struct IntVector {
    IntVectorImpl* impl;
    explicit IntVector();
    ~IntVector();
    void push_back(int val) { impl->data.push_back(val); }
    int get(size_t i) const { return impl->data[i]; }
    void set(size_t i, int val) { impl->data[i] = val; }
    size_t size() const { return impl->data.size(); }
    bool empty() const { return impl->data.empty(); }
    size_t capacity() const { return impl->data.capacity(); }
    int* data() { return impl->data.data(); }
    void clear() { impl->data.clear(); }
};

struct StringVector {
    StringVectorImpl* impl;
    explicit StringVector();
    ~StringVector();
    size_t size() const { return impl->data.size(); }
};

#endif
