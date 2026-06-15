hicc::cpp! {
    #include <iostream>
    #include <vector>

    #include "class_const.h"
}

hicc::import_class! {
    #[cpp(class = "Calculator")]
    pub class Calculator {
        #[cpp(method = "int getValue() const")]
        pub fn get_value(&self) -> i32;

        #[cpp(method = "int getHistoryCount() const")]
        pub fn get_history_count(&self) -> i32;

        #[cpp(method = "void add(int v)")]
        pub fn add(&mut self, v: i32);

        #[cpp(method = "void subtract(int v)")]
        pub fn subtract(&mut self, v: i32);

        #[cpp(method = "void clear()")]
        pub fn clear(&mut self);
    }
}

hicc::import_lib! {
    #![link_name = "class_const"]

    class Calculator;

    #[cpp(func = "std::unique_ptr<Calculator> hicc::make_unique<Calculator>()")]
    pub fn calculator_new() -> Calculator;
}
