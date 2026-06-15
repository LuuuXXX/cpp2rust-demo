#pragma once

class Number {
    int value;
public:
    Number(int v);
    ~Number();
    int getValue() const;
    Number operator+(const Number& other) const;
    Number operator-(const Number& other) const;
    Number operator*(const Number& other) const;
    Number operator/(const Number& other) const;
    int compare(const Number& other) const;
    Number operator-() const;
    Number& operator++();
    Number& operator--();
    Number& operator+=(const Number& other);
    Number& operator-=(const Number& other);
};
