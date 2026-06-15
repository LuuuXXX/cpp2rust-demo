#pragma once

class Counter {
    int value;
    static int instance_count;
public:
    Counter();
    ~Counter();
    int getValue() const;
    void increment();
    static int getInstanceCount();
    static void resetInstanceCount();
};
