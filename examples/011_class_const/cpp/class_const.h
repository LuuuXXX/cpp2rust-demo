#pragma once

#ifdef __cplusplus
extern "C" {
#endif

struct Calculator;

struct Calculator* calculator_new(void);
void calculator_delete(struct Calculator* self);

// const 成员函数
int calculator_getValue(const struct Calculator* self);
int calculator_getHistoryCount(const struct Calculator* self);

// 非 const 成员函数
void calculator_add(struct Calculator* self, int value);
void calculator_subtract(struct Calculator* self, int value);
void calculator_clear(struct Calculator* self);

#ifdef __cplusplus
}

// Full class definition - for hicc code generation
#include <vector>
class Calculator {
    int value;
    std::vector<int> history;
public:
    Calculator();
    ~Calculator();
    int getValue() const;
    int getHistoryCount() const;
    void add(int v);
    void subtract(int v);
    void clear();
};

#endif
