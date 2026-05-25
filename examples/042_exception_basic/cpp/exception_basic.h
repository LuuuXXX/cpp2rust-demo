#pragma once

#ifdef __cplusplus
extern "C" {
#endif

// 异常处理示例
// 展示如何在 FFI 边界处理 C++ 异常

#include <stddef.h>

// Exception status codes
#define EXCEPTION_NONE 0
#define EXCEPTION_INVALID_ARGUMENT 1
#define EXCEPTION_OUT_OF_RANGE 2
#define EXCEPTION_RUNTIME_ERROR 3

// Calculator structure
struct Calculator;

struct Calculator* calculator_new(void);
void calculator_delete(struct Calculator* self);

// Get last exception code
int calculator_get_exception(const struct Calculator* self);

// Clear exception state
void calculator_clear_exception(struct Calculator* self);

// Division (may throw exception)
int calculator_divide(struct Calculator* self, int a, int b);

// Safe array access (may throw exception)
int calculator_safe_get(struct Calculator* self, int* arr, int size, int index);

// String to int conversion (may throw exception)
int string_to_int(struct Calculator* self, const char* str);

// Check if there's an exception
int has_exception(const struct Calculator* self);

#ifdef __cplusplus
}

// Full class definition - for hicc code generation
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
};

#endif
