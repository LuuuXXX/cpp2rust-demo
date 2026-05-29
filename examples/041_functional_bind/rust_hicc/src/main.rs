hicc::cpp! {
    #include <stddef.h>
    #include <iostream>
    #include <functional>
    #include <string>

    int add_five_impl(int a, int b) {
        std::cout << "add_five called: " << a << " + 5 = " << (a + 5) << std::endl;
        return a + 5;
    }

    int add_ten_impl(int a, int b) {
        std::cout << "add_ten called: " << a << " + 10 = " << (a + 10) << std::endl;
        return a + 10;
    }

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

    StringProcessor* string_processor_new(void) {
        return new StringProcessor();
    }

    void string_processor_delete(StringProcessor* self) {
        delete self;
    }
}

hicc::import_class! {
    #[cpp(class = "Adder", destroy = "adder_delete")]
    class Adder {
        #[cpp(method = "int add(int value)")]
        fn add(&mut self, value: i32) -> i32;
    }
}

hicc::import_class! {
    #[cpp(class = "Multiplier", destroy = "multiplier_delete")]
    class Multiplier {
        #[cpp(method = "int multiply(int value)")]
        fn multiply(&mut self, value: i32) -> i32;
    }
}

hicc::import_class! {
    #[cpp(class = "StringProcessor", destroy = "string_processor_delete")]
    class StringProcessor {
        #[cpp(method = "void set_target(const char* t)")]
        fn set_target(&mut self, t: *const i8);

        #[cpp(method = "int count_char(char ch)")]
        fn count_char(&mut self, ch: i8) -> i32;
    }
}

hicc::import_lib! {
    #![link_name = "functional_bind"]

    class Adder;
    class Multiplier;
    class StringProcessor;

    #[cpp(func = "Adder* adder_new(int)")]
    fn adder_new(base_value: i32) -> Adder;

    #[cpp(func = "Multiplier* multiplier_new(int)")]
    fn multiplier_new(factor: i32) -> Multiplier;

    #[cpp(func = "StringProcessor* string_processor_new()")]
    fn string_processor_new() -> StringProcessor;

    #[cpp(func = "int add_five_impl(int, int)")]
    fn add_five_impl(a: i32, b: i32) -> i32;

    #[cpp(func = "int add_ten_impl(int, int)")]
    fn add_ten_impl(a: i32, b: i32) -> i32;

    #[cpp(func = "int add_five(int)")]
    fn add_five(a: i32) -> i32;

    #[cpp(func = "int add_ten(int)")]
    fn add_ten(a: i32) -> i32;
}

fn main() {
    use std::ffi::CString;

    println!("=== 041_functional_bind - std::bind 绑定 ===\n");

    // Adder example - bound base value
    println!("--- Adder Demo (绑定基础值) ---");
    unsafe {
        let mut adder = adder_new(100);
        println!("Result of adder.add(50): {}", adder.add(50));
        println!("Result of adder.add(30): {}", adder.add(30));
        adder_delete(&adder);
    }

    // Multiplier example - bound multiplier
    println!("\n--- Multiplier Demo (绑定乘数) ---");
    unsafe {
        let mut multiplier = multiplier_new(7);
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

