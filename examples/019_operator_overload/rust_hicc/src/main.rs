hicc::cpp! {
    #include <iostream>

    class Number {
        int value;
    public:
        Number(int v) : value(v) {}
        ~Number() {}
        int getValue() const { return value; }
        Number operator+(const Number& other) const { return Number(value + other.value); }
        Number operator-(const Number& other) const { return Number(value - other.value); }
        Number operator*(const Number& other) const { return Number(value * other.value); }
        Number operator/(const Number& other) const { return Number(value / other.value); }
        int compare(const Number& other) const { return value - other.value; }
        Number operator-() const { return Number(-value); }
        Number& operator++() { ++value; return *this; }
        Number& operator--() { --value; return *this; }
        Number& operator+=(const Number& other) { value += other.value; return *this; }
        Number& operator-=(const Number& other) { value -= other.value; return *this; }
    };

    Number* number_new(int value) {
        return new Number(value);
    }

    void number_delete(Number* self) {
        delete self;
    }

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
    #[cpp(class = "Number")]
    class Number {
        #[cpp(method = "int getValue() const")]
        fn get_value(&self) -> i32;

        #[cpp(method = "int compare(const Number & other) const")]
        fn compare(&self, other: *const Number) -> i32;
    }
}

hicc::import_lib! {
    #![link_name = "operator_overload"]

    class Number;

    #[cpp(func = "Number* number_new(int)")]
    fn number_new(value: i32) -> *mut Number;

    #[cpp(func = "void number_delete(Number* self)")]
    unsafe fn number_delete(self_: *mut Number);

    #[cpp(func = "int number_get_value(const Number*)")]
    fn number_getValue(self_: *const Number) -> i32;

    #[cpp(func = "Number* number_add(const Number*, const Number*)")]
    fn number_add(a: *const Number, b: *const Number) -> *mut Number;

    #[cpp(func = "Number* number_sub(const Number*, const Number*)")]
    fn number_sub(a: *const Number, b: *const Number) -> *mut Number;

    #[cpp(func = "Number* number_mul(const Number*, const Number*)")]
    fn number_mul(a: *const Number, b: *const Number) -> *mut Number;

    #[cpp(func = "Number* number_div(const Number*, const Number*)")]
    fn number_div(a: *const Number, b: *const Number) -> *mut Number;

    #[cpp(func = "Number* number_negate(const Number*)")]
    fn number_negate(a: *const Number) -> *mut Number;

    #[cpp(func = "int number_compare(const Number*, const Number*)")]
    fn number_compare(a: *const Number, b: *const Number) -> i32;
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



