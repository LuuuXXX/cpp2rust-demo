hicc::cpp! {
    #include <stddef.h>
    #include <iostream>
    #include <functional>
    #include <string>

    class AdderImpl {
    public:
        int base_value;
    public:
        AdderImpl(int base) : base_value(base) {}
        ~AdderImpl() {}
        int add(int value) {
    return base_value + value;
}
    };

    class MultiplierImpl {
    public:
        int factor;
    public:
        MultiplierImpl(int f) : factor(f) {}
        ~MultiplierImpl() {}
        int multiply(int value) {
    return factor * value;
}
    };

    class StringProcessorImpl {
    public:
        std::string target;
    public:
        StringProcessorImpl() {}
        ~StringProcessorImpl() {}
        void set_target(const char* t) {
    target = t;
}
        int count_char(char ch) {
    int count = 0;
    for (char c : target) {
        if (c == ch) count++;
    }
    return count;
}
    };

    struct Adder {
    public:
        AdderImpl* impl;
        Adder(int base_value) : impl(new AdderImpl(base_value)) {}
        ~Adder() { delete impl; }
    };

    struct Multiplier {
    public:
        MultiplierImpl* impl;
        Multiplier(int factor) : impl(new MultiplierImpl(factor)) {}
        ~Multiplier() { delete impl; }
    };

    struct StringProcessor {
    public:
        StringProcessorImpl* impl;
        StringProcessor() : impl(new StringProcessorImpl()) {}
        ~StringProcessor() { delete impl; }
    };

    Adder* adder_new(int base_value) {
        return new Adder(base_value);
    }

    void adder_delete(Adder* self) {
        delete self;
    }

    int add_five(int a) {
        return add_five_impl(a, 5);
    }

    int add_ten(int a) {
        return add_ten_impl(a, 10);
    }

    Multiplier* multiplier_new(int factor) {
        return new Multiplier(factor);
    }

    void multiplier_delete(Multiplier* self) {
        delete self;
    }

    StringProcessor* string_processor_new() {
        return new StringProcessor();
    }

    void string_processor_delete(StringProcessor* self) {
        delete self;
    }

    int add_five_impl(int a, int b) {
        std::cout << "add_five called: " << a << " + 5 = " << (a + 5) << std::endl;
        return a + 5;
    }

    int add_ten_impl(int a, int b) {
        std::cout << "add_ten called: " << a << " + 10 = " << (a + 10) << std::endl;
        return a + 10;
    }
}

hicc::import_lib! {
    #![link_name = "functional_bind"]

    class Adder;
    class Multiplier;
    class StringProcessor;

    #[cpp(func = "Adder* adder_new(int)")]
    fn adder_new(base_value: i32) -> *mut Adder;

    #[cpp(func = "void adder_delete(Adder* self)")]
    unsafe fn adder_delete(self_: *mut Adder);

    #[cpp(func = "int add_five(int)")]
    fn add_five(a: i32) -> i32;

    #[cpp(func = "int add_ten(int)")]
    fn add_ten(a: i32) -> i32;

    #[cpp(func = "Multiplier* multiplier_new(int)")]
    fn multiplier_new(factor: i32) -> *mut Multiplier;

    #[cpp(func = "void multiplier_delete(Multiplier* self)")]
    unsafe fn multiplier_delete(self_: *mut Multiplier);

    #[cpp(func = "StringProcessor* string_processor_new()")]
    fn string_processor_new() -> *mut StringProcessor;

    #[cpp(func = "void string_processor_delete(StringProcessor* self)")]
    unsafe fn string_processor_delete(self_: *mut StringProcessor);

    #[cpp(func = "int add_five_impl(int, int)")]
    fn add_five_impl(a: i32, b: i32) -> i32;

    #[cpp(func = "int add_ten_impl(int, int)")]
    fn add_ten_impl(a: i32, b: i32) -> i32;
}

fn main() {
    use std::ffi::CString;

    println!("=== 041_functional_bind - std::bind 绑定 ===\n");

    // Adder example - bound base value
    println!("--- Adder Demo (绑定基础值) ---");
    unsafe {
        let adder = adder_new(100);
        println!("Result of adder.add(50): {}", adder.add(50));
        println!("Result of adder.add(30): {}", adder.add(30));
        adder_delete(&adder);
    }

    // Multiplier example - bound multiplier
    println!("\n--- Multiplier Demo (绑定乘数) ---");
    unsafe {
        let multiplier = multiplier_new(7);
        println!("multiply(6) = {}", multiplier.multiply(6));
        println!("multiply(11) = {}", multiplier.multiply(11));
        multiplier_delete(&multiplier);
    }

    // StringProcessor example - bound member function and argument
    println!("\n--- StringProcessor Demo (成员函数绑定) ---");
    unsafe {
        let mut processor = string_processor_new();
        processor.set_target(CString::new("hello world!").unwrap().as_ptr());

        println!("Count of 'l': {}", processor.count_char('l' as i8));
        println!("Count of 'o': {}", processor.count_char('o' as i8));
        println!("Count of 'h': {}", processor.count_char('h' as i8));

        string_processor_delete(&processor);
    }

    println!("\n--- 总结 ---");
    println!("1. std::bind 创建部分应用的函数对象");
    println!("2. 可以绑定函数、成员函数、参数值");
    println!("3. 通过 opaque pointer 在 FFI 间传递绑定后的函数");
    println!("4. _1, _2 等占位符表示未绑定的参数位置");
}


