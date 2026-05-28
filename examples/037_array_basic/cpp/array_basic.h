#pragma once

#ifdef __cplusplus
extern "C" {
#endif

// std::array 基本操作示例

#include <stddef.h>

// Fixed-size array wrappers
struct IntArray5;
struct DoubleArray3;
struct StringArray4;

// IntArray5 operations (size 5 int array)
struct IntArray5* int_array5_new(void);
struct IntArray5* int_array5_new_from(const int* values);
void int_array5_delete(struct IntArray5* self);

size_t int_array5_size(const struct IntArray5* self);
int int_array5_empty(const struct IntArray5* self);

int int_array5_get(const struct IntArray5* self, size_t index);
void int_array5_set(struct IntArray5* self, size_t index, int value);

int* int_array5_data(struct IntArray5* self);
const int* int_array5_data_const(const struct IntArray5* self);

// Iterator support
int* int_array5_begin(struct IntArray5* self);
int* int_array5_end(struct IntArray5* self);

// Bounds-checked access
int int_array5_at(const struct IntArray5* self, size_t index);

// Swap
void int_array5_swap(struct IntArray5* self, struct IntArray5* other);

// DoubleArray3 operations
struct DoubleArray3* double_array3_new(void);
struct DoubleArray3* double_array3_new_from(const double* values);
void double_array3_delete(struct DoubleArray3* self);

size_t double_array3_size(const struct DoubleArray3* self);
double double_array3_get(const struct DoubleArray3* self, size_t index);
void double_array3_set(struct DoubleArray3* self, size_t index, double value);
double* double_array3_data(struct DoubleArray3* self);

// StringArray4 operations
struct StringArray4* string_array4_new(void);
void string_array4_delete(struct StringArray4* self);

size_t string_array4_size(const struct StringArray4* self);
const char* string_array4_get(const struct StringArray4* self, size_t index);
void string_array4_set(struct StringArray4* self, size_t index, const char* value);

#ifdef __cplusplus
}

// Full class definition - for hicc code generation
#include <array>
#include <string>

class IntArray5Impl {
public:
    std::array<int, 5> data;
    IntArray5Impl();
    explicit IntArray5Impl(const int* values);
    ~IntArray5Impl();
};

class DoubleArray3Impl {
public:
    std::array<double, 3> data;
    DoubleArray3Impl();
    explicit DoubleArray3Impl(const double* values);
    ~DoubleArray3Impl();
};

class StringArray4Impl {
public:
    std::array<std::string, 4> data;
    bool initialized[4];
    StringArray4Impl();
    ~StringArray4Impl();
};

struct IntArray5 {
    IntArray5Impl* impl;
    explicit IntArray5();
    explicit IntArray5(const int* values);
    ~IntArray5();
    size_t size() const { return impl->data.size(); }
    bool empty() const { return impl->data.empty(); }
    void set(size_t i, int val) { impl->data[i] = val; }
    int get(size_t i) const { return impl->data[i]; }
    int at(size_t i) const { return impl->data.at(i); }
    int* data() { return impl->data.data(); }
};

struct DoubleArray3 {
    DoubleArray3Impl* impl;
    explicit DoubleArray3();
    explicit DoubleArray3(const double* values);
    ~DoubleArray3();
    size_t size() const { return impl->data.size(); }
};

struct StringArray4 {
    StringArray4Impl* impl;
    explicit StringArray4();
    ~StringArray4();
    size_t size() const { return impl->data.size(); }
};

#endif
