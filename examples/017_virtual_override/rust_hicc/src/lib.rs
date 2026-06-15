hicc::cpp! {
    #include <string>

    #include "virtual_override.h"
    std::unique_ptr<Base> _cpp2rust_make_unique_base_with_n(const char* n) { return std::make_unique<Base>(n); }
    std::unique_ptr<Derived> _cpp2rust_make_unique_derived_with_v(double v) { return std::make_unique<Derived>(v); }
}

hicc::import_class! {
    #[cpp(class = "Base")]
    pub class Base {
        #[cpp(method = "double area() const")]
        pub fn area(&self) -> f64;

        #[cpp(method = "const char* getName() const")]
        pub fn get_name(&self) -> *const i8;
    }
}

hicc::import_class! {
    #[cpp(class = "Derived")]
    pub class Derived {
        #[cpp(method = "const char* getName() const")]
        pub fn get_name(&self) -> *const i8;

        #[cpp(method = "double area() const")]
        pub fn area(&self) -> f64;

        #[cpp(method = "double getValue() const")]
        pub fn get_value(&self) -> f64;
    }
}

hicc::import_lib! {
    #![link_name = "virtual_override"]

    class Base;
    class Derived;

    #[cpp(func = "std::unique_ptr<Base> _cpp2rust_make_unique_base_with_n(const char*)")]
    pub unsafe fn base_new_with_n(n: *const i8) -> Base;

    #[cpp(func = "std::unique_ptr<Derived> _cpp2rust_make_unique_derived_with_v(double)")]
    pub fn derived_new_with_v(v: f64) -> Derived;
}
