#pragma once

#define EXCEPTION_NONE 0
#define EXCEPTION_INVALID_ARGUMENT 1
#define EXCEPTION_OUT_OF_RANGE 2
#define EXCEPTION_RUNTIME_ERROR 3

#ifdef __cplusplus

class ExceptionInfo {
public:
    int code;
    char message[256];
    ExceptionInfo();
    void clear();
    void set(int c, const char* msg);
};

class CalculatorImpl {
public:
    ExceptionInfo last_exception;
    CalculatorImpl();
    ~CalculatorImpl();
    void clear_exception();
    int get_exception();
    int divide(int a, int b);
    int safe_get(int* arr, int size, int index);
    int string_to_int(const char* str);
};

struct Calculator {
    CalculatorImpl* impl;
    Calculator();
    ~Calculator();
    void clear_exception() { impl->clear_exception(); }
    int get_exception() { return impl->get_exception(); }
    int divide(int a, int b) {
        try { return impl->divide(a, b); } catch (...) { return 0; }
    }
    int string_to_int(const char* str) {
        try { return impl->string_to_int(str); } catch (...) { return 0; }
    }
};

#endif
