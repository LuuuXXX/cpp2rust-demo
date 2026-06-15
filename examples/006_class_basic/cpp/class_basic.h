#pragma once

class Counter {
    int value = 0;
public:
    Counter();
    ~Counter();
    int get() const;
    void increment();
    void decrement();
};
