#include <hicc/hicc.hpp>
#line 0 R"(src/main.rs)"
#line 2
#include <iostream>

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

    Number* number_new(int value) {
        return new Number(value);
    }

    void number_delete(Number* self) {
        delete self;
    }

    int number_getValue(Number* self) {
        return self->getValue();
    }

    Number* number_add(Number* self, Number* other) {
        return new Number(self->operator+(*other));
    }

    Number* number_sub(Number* self, Number* other) {
        return new Number(self->operator-(*other));
    }

    Number* number_mul(Number* self, Number* other) {
        return new Number(self->operator*(*other));
    }

    Number* number_div(Number* self, Number* other) {
        return new Number(self->operator/(*other));
    }

    int number_compare(Number* self, Number* other) {
        return self->compare(*other);
    }

    Number* number_negate(Number* self) {
        return new Number(self->operator-());
    }

    Number* number_increment(Number* self) {
        return &self->operator++();
    }

    Number* number_decrement(Number* self) {
        return &self->operator--();
    }

    void number_add_assign(Number* self, Number* other) {
        self->operator+=(*other);
    }

    void number_sub_assign(Number* self, Number* other) {
        self->operator-=(*other);
    }

    Number::Number(int v) : value(v) {}
    Number::~Number() {}
    int Number::getValue() const { return value; }
    Number Number::operator+(const Number& other) const { return Number(value + other.value); }
    Number Number::operator-(const Number& other) const { return Number(value - other.value); }
    Number Number::operator*(const Number& other) const { return Number(value * other.value); }
    Number Number::operator/(const Number& other) const { return Number(value / other.value); }
    int Number::compare(const Number& other) const { return value - other.value; }
    Number Number::operator-() const { return Number(-value); }
    Number& Number::operator++() { ++value; return *this; }
    Number& Number::operator--() { --value; return *this; }
    Number& Number::operator+=(const Number& other) { value += other.value; return *this; }
    Number& Number::operator-=(const Number& other) { value -= other.value; return *this; }
#line 90
 struct Number_90;
#line 90
namespace hicc { template<> struct MethodsType<Number, void> { typedef Number_90 methods_type; }; }
#line 90
 struct Number_90 {
#line 90
typedef Number Self; typedef void SelfContainer; typedef Number_90 SelfMethods;
#line 92
static void _hicc_test_92() { int (Self::* _92)() const = &Self::getValue; (void)_92; }
#line 92
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((int (Self::*)() const)&Self::getValue));
#line 90
};
#line 98
EXPORT_METHODS_BEG(operator_overload) {
#line 102
static void _hicc_test_102() { Number* (* _102)(int value) = &number_new; (void)_102; }
#line 102
EXPORT_METHOD_IN(void, ExportMethods, ((Number* (*)(int value))&number_new));
#line 105
static void _hicc_test_105() { void (* _105)(Number* self) = &number_delete; (void)_105; }
#line 105
EXPORT_METHOD_IN(void, ExportMethods, ((void (*)(Number* self))&number_delete));
#line 108
static void _hicc_test_108() { int (* _108)(Number* self) = &number_getValue; (void)_108; }
#line 108
EXPORT_METHOD_IN(void, ExportMethods, ((int (*)(Number* self))&number_getValue));
#line 111
static void _hicc_test_111() { Number* (* _111)(Number* self, Number* other) = &number_add; (void)_111; }
#line 111
EXPORT_METHOD_IN(void, ExportMethods, ((Number* (*)(Number* self, Number* other))&number_add));
#line 114
static void _hicc_test_114() { Number* (* _114)(Number* self, Number* other) = &number_sub; (void)_114; }
#line 114
EXPORT_METHOD_IN(void, ExportMethods, ((Number* (*)(Number* self, Number* other))&number_sub));
#line 117
static void _hicc_test_117() { Number* (* _117)(Number* self, Number* other) = &number_mul; (void)_117; }
#line 117
EXPORT_METHOD_IN(void, ExportMethods, ((Number* (*)(Number* self, Number* other))&number_mul));
#line 120
static void _hicc_test_120() { Number* (* _120)(Number* self, Number* other) = &number_div; (void)_120; }
#line 120
EXPORT_METHOD_IN(void, ExportMethods, ((Number* (*)(Number* self, Number* other))&number_div));
#line 123
static void _hicc_test_123() { int (* _123)(Number* self, Number* other) = &number_compare; (void)_123; }
#line 123
EXPORT_METHOD_IN(void, ExportMethods, ((int (*)(Number* self, Number* other))&number_compare));
#line 126
static void _hicc_test_126() { Number* (* _126)(Number* self) = &number_negate; (void)_126; }
#line 126
EXPORT_METHOD_IN(void, ExportMethods, ((Number* (*)(Number* self))&number_negate));
#line 129
static void _hicc_test_129() { Number* (* _129)(Number* self) = &number_increment; (void)_129; }
#line 129
EXPORT_METHOD_IN(void, ExportMethods, ((Number* (*)(Number* self))&number_increment));
#line 132
static void _hicc_test_132() { Number* (* _132)(Number* self) = &number_decrement; (void)_132; }
#line 132
EXPORT_METHOD_IN(void, ExportMethods, ((Number* (*)(Number* self))&number_decrement));
#line 135
static void _hicc_test_135() { void (* _135)(Number* self, Number* other) = &number_add_assign; (void)_135; }
#line 135
EXPORT_METHOD_IN(void, ExportMethods, ((void (*)(Number* self, Number* other))&number_add_assign));
#line 138
static void _hicc_test_138() { void (* _138)(Number* self, Number* other) = &number_sub_assign; (void)_138; }
#line 138
EXPORT_METHOD_IN(void, ExportMethods, ((void (*)(Number* self, Number* other))&number_sub_assign));
#line 98
} EXPORT_METHODS_END();

