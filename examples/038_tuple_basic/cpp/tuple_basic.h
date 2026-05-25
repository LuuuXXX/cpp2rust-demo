#pragma once

#ifdef __cplusplus
extern "C" {
#endif

// std::tuple 基本操作示例

#include <stddef.h>

// Tuple wrappers
struct Tuple2;  // (int, const char*)
struct Tuple3;  // (int, double, const char*)
struct Tuple4;  // (int, double, const char*, int)

// Tuple2 operations
struct Tuple2* tuple2_new(int first, const char* second);
void tuple2_delete(struct Tuple2* self);

int tuple2_get_first(const struct Tuple2* self);
const char* tuple2_get_second(const struct Tuple2* self);

// Tuple3 operations
struct Tuple3* tuple3_new(int first, double second, const char* third);
void tuple3_delete(struct Tuple3* self);

int tuple3_get_first(const struct Tuple3* self);
double tuple3_get_second(const struct Tuple3* self);
const char* tuple3_get_third(const struct Tuple3* self);

// Tuple4 operations
struct Tuple4* tuple4_new(int first, double second, const char* third, int fourth);
void tuple4_delete(struct Tuple4* self);

int tuple4_get_first(const struct Tuple4* self);
double tuple4_get_second(const struct Tuple4* self);
const char* tuple4_get_third(const struct Tuple4* self);
int tuple4_get_fourth(const struct Tuple4* self);

// Helper functions to create various tuple types
struct Tuple2* make_int_string_pair(int i, const char* s);
struct Tuple3* make_int_double_string(int i, double d, const char* s);

// Tuple comparison
int tuple2_equals(const struct Tuple2* self, int first, const char* second);
int tuple3_equals(const struct Tuple3* self, int first, double second, const char* third);

#ifdef __cplusplus
}

// Full class definition - for hicc code generation
#include <tuple>
#include <string>

class Tuple2Impl {
public:
    std::tuple<int, std::string> data;
    Tuple2Impl(int first, const char* second);
    ~Tuple2Impl();
};

class Tuple3Impl {
public:
    std::tuple<int, double, std::string> data;
    Tuple3Impl(int first, double second, const char* third);
    ~Tuple3Impl();
};

class Tuple4Impl {
public:
    std::tuple<int, double, std::string, int> data;
    Tuple4Impl(int first, double second, const char* third, int fourth);
    ~Tuple4Impl();
};

struct Tuple2 {
    Tuple2Impl* impl;
    explicit Tuple2(int first, const char* second);
    ~Tuple2();
};

struct Tuple3 {
    Tuple3Impl* impl;
    explicit Tuple3(int first, double second, const char* third);
    ~Tuple3();
};

struct Tuple4 {
    Tuple4Impl* impl;
    explicit Tuple4(int first, double second, const char* third, int fourth);
    ~Tuple4();
};

#endif
