hicc::cpp! {
    #include <iostream>

    #include "inheritance_multiple.h"
}

hicc::import_class! {
    #[cpp(class = "Derived", destroy = "derived_delete")]
    class Derived {
        #[cpp(method = "int getValue1() const")]
        fn get_value1(&self) -> i32;

        #[cpp(method = "int getValue2() const")]
        fn get_value2(&self) -> i32;

        #[cpp(method = "int getDerivedValue() const")]
        fn get_derived_value(&self) -> i32;

        #[cpp(method = "void compute() const")]
        fn compute(&self);
    }
}

hicc::import_lib! {
    #![link_name = "inheritance_multiple"]

    class Derived;

    #[cpp(func = "Derived* derived_new(int, int, int)")]
    fn derived_new(v1: i32, v2: i32, dv: i32) -> Derived;
}

fn main() {
    let derived = unsafe { derived_new(10, 20, 30) };

    println!("Base1 value: {}", derived.get_value1());
    println!("Base2 value: {}", derived.get_value2());
    println!("Derived value: {}", derived.get_derived_value());

    derived.compute();

    println!("\nRust FFI: Multiple inheritance with hicc pattern");
}

