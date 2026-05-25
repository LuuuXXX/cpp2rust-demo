#pragma once

#ifdef __cplusplus
extern "C" {
#endif

struct Number;

struct Number* number_new(int value);
void number_delete(struct Number* self);

int number_getValue(struct Number* self);

struct Number* number_add(struct Number* self, struct Number* other);
struct Number* number_sub(struct Number* self, struct Number* other);
struct Number* number_mul(struct Number* self, struct Number* other);
struct Number* number_div(struct Number* self, struct Number* other);

int number_compare(struct Number* self, struct Number* other);

struct Number* number_negate(struct Number* self);
struct Number* number_increment(struct Number* self);
struct Number* number_decrement(struct Number* self);

void number_add_assign(struct Number* self, struct Number* other);
void number_sub_assign(struct Number* self, struct Number* other);

#ifdef __cplusplus
}

// Full class definition - for hicc code generation
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

#endif
