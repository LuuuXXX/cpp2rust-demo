hicc::cpp! {
    #include <iostream>
    #include <vector>

    class Calculator {
        int value;
        std::vector<int> history;
    public:
        Calculator() : value(0) {}
        ~Calculator() {}
        int getValue() const {
            return value;
        }
        int getHistoryCount() const {
            return static_cast<int>(history.size());
        }
        void add(int v) {
            history.push_back(v);
            value += v;
        }
        void subtract(int v) {
            history.push_back(-v);
            value -= v;
        }
        void clear() {
            history.clear();
            value = 0;
        }
    };

    Calculator* calculator_new() {
        return new Calculator();
    }

    void calculator_delete(Calculator* self) {
        delete self;
    }
}

hicc::import_class! {
    #[cpp(class = "Calculator")]
    class Calculator {
        #[cpp(method = "int getValue() const")]
        fn get_value(&self) -> i32;

        #[cpp(method = "int getHistoryCount() const")]
        fn get_history_count(&self) -> i32;

        #[cpp(method = "void add(int v)")]
        fn add(&mut self, v: i32);

        #[cpp(method = "void subtract(int v)")]
        fn subtract(&mut self, v: i32);

        #[cpp(method = "void clear()")]
        fn clear(&mut self);
    }
}

hicc::import_lib! {
    #![link_name = "class_const"]

    class Calculator;

    #[cpp(func = "Calculator* calculator_new()")]
    fn calculator_new() -> *mut Calculator;

    #[cpp(func = "void calculator_delete(Calculator* self)")]
    unsafe fn calculator_delete(self_: *mut Calculator);
}

fn main() {
    let mut calc = calculator_new();

    println!("Initial value: {}", calc.get_value());
    println!("History count: {}", calc.get_history_count());

    calc.add(10);
    println!("After add(10): {}", calc.get_value());

    calc.add(5);
    println!("After add(5): {}", calc.get_value());

    calc.subtract(3);
    println!("After subtract(3): {}", calc.get_value());

    println!("History count: {}", calc.get_history_count());

    calc.clear();
    println!("After clear: {}", calc.get_value());
    println!("History count: {}", calc.get_history_count());

    unsafe {
        calculator_delete(&calc);
    }

    println!("\nRust FFI: const member functions work!");
}
