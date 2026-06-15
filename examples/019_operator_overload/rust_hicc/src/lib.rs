hicc::cpp! {
    #include "operator_overload.h"
    std::unique_ptr<Number> number_new(int v) {
        return std::make_unique<Number>(v);
    }
}

hicc::import_class! {
    #[cpp(class = "Number")]
    pub class Number {
        #[cpp(method = "int getValue() const")]
        pub fn get_value(&self) -> i32;

        #[cpp(method = "int compare(const Number & other) const")]
        pub fn compare(&self, other: &Number) -> i32;
    }
}

hicc::import_lib! {
    #![link_name = "operator_overload"]

    class Number;

    #[cpp(func = "std::unique_ptr<Number> number_new(int)")]
    pub fn number_new_with_v(v: i32) -> Number;
}
