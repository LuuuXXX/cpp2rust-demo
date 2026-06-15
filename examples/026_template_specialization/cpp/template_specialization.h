#pragma once

class IntHolder {
    int value_;
public:
    explicit IntHolder(int value);
    ~IntHolder();
    int get() const;
    const char* describe() const;
};

class DoubleHolder {
    double value_;
public:
    explicit DoubleHolder(double value);
    ~DoubleHolder();
    double get() const;
    const char* describe() const;
};

class StringHolder {
    char* value_;
    int length_;
public:
    explicit StringHolder(const char* value);
    ~StringHolder();
    const char* get() const;
    const char* describe() const;
};
