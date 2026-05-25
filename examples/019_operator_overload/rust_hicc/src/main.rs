hicc::cpp! {
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
}

hicc::import_class! {
    #[cpp(class = "Number")]
    class Number {
        #[cpp(method = "int getValue() const")]
        fn getValue(&self) -> i32;
    }
}

hicc::import_lib! {
    #![link_name = "operator_overload"]

    class Number;

    #[cpp(func = "Number* number_new(int value)")]
    fn number_new(value: i32) -> *mut Number;

    #[cpp(func = "void number_delete(Number* self)")]
    unsafe fn number_delete(self_: *mut Number);

    #[cpp(func = "int number_getValue(Number* self)")]
    fn number_getValue(self_: *mut Number) -> i32;

    #[cpp(func = "Number* number_add(Number* self, Number* other)")]
    fn number_add(self_: *mut Number, other: *mut Number) -> *mut Number;

    #[cpp(func = "Number* number_sub(Number* self, Number* other)")]
    fn number_sub(self_: *mut Number, other: *mut Number) -> *mut Number;

    #[cpp(func = "Number* number_mul(Number* self, Number* other)")]
    fn number_mul(self_: *mut Number, other: *mut Number) -> *mut Number;

    #[cpp(func = "Number* number_div(Number* self, Number* other)")]
    fn number_div(self_: *mut Number, other: *mut Number) -> *mut Number;

    #[cpp(func = "int number_compare(Number* self, Number* other)")]
    fn number_compare(self_: *mut Number, other: *mut Number) -> i32;

    #[cpp(func = "Number* number_negate(Number* self)")]
    fn number_negate(self_: *mut Number) -> *mut Number;

    #[cpp(func = "Number* number_increment(Number* self)")]
    fn number_increment(self_: *mut Number) -> *mut Number;

    #[cpp(func = "Number* number_decrement(Number* self)")]
    fn number_decrement(self_: *mut Number) -> *mut Number;

    #[cpp(func = "void number_add_assign(Number* self, Number* other)")]
    fn number_add_assign(self_: *mut Number, other: *mut Number);

    #[cpp(func = "void number_sub_assign(Number* self, Number* other)")]
    fn number_sub_assign(self_: *mut Number, other: *mut Number);
}

fn main() {
    println!("=== Operator Overload FFI ===\n");
    println!("C++ operator overloading becomes named method calls in FFI\n");

    let a = number_new(10);
    let b = number_new(3);

    println!("Created numbers: a = {}, b = {}", number_getValue(&a), number_getValue(&b));
    println!();

    // Addition: a + b
    let sum = number_add(&a, &b);
    println!("Result of a + b = {}", number_getValue(&sum));
    unsafe { number_delete(&sum) };

    // Subtraction: a - b
    let diff = number_sub(&a, &b);
    println!("Result of a - b = {}", number_getValue(&diff));
    unsafe { number_delete(&diff) };

    // Multiplication: a * b
    let prod = number_mul(&a, &b);
    println!("Result of a * b = {}", number_getValue(&prod));
    unsafe { number_delete(&prod) };

    // Division: a / b
    let quot = number_div(&a, &b);
    println!("Result of a / b = {}", number_getValue(&quot));
    unsafe { number_delete(&quot) };

    println!();

    // Unary operators
    println!("Unary operators:");
    let neg = number_negate(&a);
    println!("Negation of a = {}", number_getValue(&neg));
    unsafe { number_delete(&neg) };

    // Comparison
    let cmp = number_compare(&a, &b);
    println!("a compared to b = {}", cmp);

    println!();
    println!("Rust FFI: Operators become named methods");
    println!("a + b -> number_add(a, b)");
    println!("a - b -> number_sub(a, b)");
    println!("a * b -> number_mul(a, b)");

    unsafe {
        number_delete(&a);
        number_delete(&b);
    }
}
