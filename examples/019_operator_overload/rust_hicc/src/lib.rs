hicc::cpp! {
    #include <iostream>

    #include "operator_overload.h"

    int number_get_value(const Number* self) {
        return self->getValue();
    }

    Number* number_add(const Number* a, const Number* b) {
        return new Number(*a + *b);
    }

    Number* number_sub(const Number* a, const Number* b) {
        return new Number(*a - *b);
    }

    Number* number_mul(const Number* a, const Number* b) {
        return new Number(*a * *b);
    }

    Number* number_div(const Number* a, const Number* b) {
        return new Number(*a / *b);
    }

    Number* number_negate(const Number* a) {
        return new Number(-*a);
    }

    int number_compare(const Number* a, const Number* b) {
        return a->compare(*b);
    }
}

hicc::import_class! {
    #[cpp(class = "Number", destroy = "number_delete")]
    pub class Number {
        #[cpp(method = "int getValue() const")]
        pub fn get_value(&self) -> i32;
    }
}

hicc::import_lib! {
    #![link_name = "operator_overload"]

    class Number;

    #[cpp(func = "Number* number_new(int)")]
    pub fn number_new(value: i32) -> Number;

    #[cpp(func = "int number_get_value(const Number*)")]
    pub fn number_get_value(self_: *const Number) -> i32;

    #[cpp(func = "Number* number_add(const Number*, const Number*)")]
    pub fn number_add(a: *const Number, b: *const Number) -> *mut Number;

    #[cpp(func = "Number* number_sub(const Number*, const Number*)")]
    pub fn number_sub(a: *const Number, b: *const Number) -> *mut Number;

    #[cpp(func = "Number* number_mul(const Number*, const Number*)")]
    pub fn number_mul(a: *const Number, b: *const Number) -> *mut Number;

    #[cpp(func = "Number* number_div(const Number*, const Number*)")]
    pub fn number_div(a: *const Number, b: *const Number) -> *mut Number;

    #[cpp(func = "Number* number_negate(const Number*)")]
    pub fn number_negate(a: *const Number) -> *mut Number;

    #[cpp(func = "int number_compare(const Number*, const Number*)")]
    pub fn number_compare(a: *const Number, b: *const Number) -> i32;
}
