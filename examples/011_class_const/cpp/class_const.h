#pragma once

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
