#pragma once

#ifdef __cplusplus

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
