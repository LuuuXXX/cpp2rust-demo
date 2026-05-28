#pragma once

#ifdef __cplusplus
extern "C" {
#endif

// std::bind 绑定示例
// 展示如何通过 FFI 传递绑定了参数的函数

#include <stddef.h>

// Adder structure
struct Adder;

struct Adder* adder_new(int base_value);
void adder_delete(struct Adder* self);

int adder_add(const struct Adder* self, int value);

// Direct bound functions
int add_five(int a);
int add_ten(int a);

// Multiplier structure
struct Multiplier;
struct Multiplier* multiplier_new(int factor);
void multiplier_delete(struct Multiplier* self);
int multiply(const struct Multiplier* self, int value);

// StringProcessor (bound member functions)
struct StringProcessor;
struct StringProcessor* string_processor_new(void);
void string_processor_delete(struct StringProcessor* self);

void string_processor_set_target(struct StringProcessor* self, const char* target);
int string_processor_count_char(const struct StringProcessor* self, char ch);

#ifdef __cplusplus
}

// Full class definition - for hicc code generation
#include <string>

class AdderImpl {
public:
    int base_value;
    AdderImpl(int base);
    ~AdderImpl();
    int add(int value);
};

class MultiplierImpl {
public:
    int factor;
    MultiplierImpl(int f);
    ~MultiplierImpl();
    int multiply(int value);
};

class StringProcessorImpl {
public:
    std::string target;
    StringProcessorImpl();
    ~StringProcessorImpl();
    void set_target(const char* t);
    int count_char(char ch);
};

struct Adder {
    AdderImpl* impl;
    explicit Adder(int base_value);
    ~Adder();
    int add(int value) { return impl->add(value); }
};

struct Multiplier {
    MultiplierImpl* impl;
    explicit Multiplier(int factor);
    ~Multiplier();
    int multiply(int value) { return impl->multiply(value); }
};

struct StringProcessor {
    StringProcessorImpl* impl;
    StringProcessor();
    ~StringProcessor();
    void set_target(const char* t) { impl->set_target(t); }
    int count_char(char ch) { return impl->count_char(ch); }
};

#endif
