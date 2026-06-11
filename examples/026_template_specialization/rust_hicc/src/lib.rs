hicc::cpp! {
    #include <iostream>
    #include <cstring>
    #include <cstdlib>
    #include <cstdio>

    #include "template_specialization.h"
}

hicc::import_class! {
    #[cpp(class = "IntHolder", destroy = "intholder_delete")]
    pub class IntHolder {
        #[cpp(method = "int get() const")]
        pub fn get(&self) -> i32;

        #[cpp(method = "const char* describe() const")]
        pub fn describe(&self) -> *const i8;
    }
}

hicc::import_class! {
    #[cpp(class = "DoubleHolder", destroy = "doubleholder_delete")]
    pub class DoubleHolder {
        #[cpp(method = "double get() const")]
        pub fn get(&self) -> f64;

        #[cpp(method = "const char* describe() const")]
        pub fn describe(&self) -> *const i8;
    }
}

hicc::import_class! {
    #[cpp(class = "StringHolder", destroy = "stringholder_delete")]
    pub class StringHolder {
        #[cpp(method = "const char* get() const")]
        pub fn get(&self) -> *const i8;

        #[cpp(method = "const char* describe() const")]
        pub fn describe(&self) -> *const i8;
    }
}

hicc::import_lib! {
    #![link_name = "template_specialization"]

    class IntHolder;
    class DoubleHolder;
    class StringHolder;

    #[cpp(func = "IntHolder* intholder_new(int)")]
    pub fn intholder_new(value: i32) -> IntHolder;

    #[cpp(func = "DoubleHolder* doubleholder_new(double)")]
    pub fn doubleholder_new(value: f64) -> DoubleHolder;

    #[cpp(func = "StringHolder* stringholder_new(const char*)")]
    pub unsafe fn stringholder_new(value: *const i8) -> StringHolder;
}
