hicc::cpp! {
    #include <stddef.h>
    #include <iostream>
    #include <tuple>
    #include <string>
    #include <cstring>

    #include "tuple_basic.h"
}

hicc::import_class! {
    #[cpp(class = "Tuple2", destroy = "tuple2_delete")]
    pub class Tuple2 {
        #[cpp(method = "int get_first() const")]
        pub fn get_first(&self) -> i32;

        #[cpp(method = "const char* get_second() const")]
        pub fn get_second(&self) -> *const i8;
    }
}

hicc::import_class! {
    #[cpp(class = "Tuple3", destroy = "tuple3_delete")]
    pub class Tuple3 {
        #[cpp(method = "int get_first() const")]
        pub fn get_first(&self) -> i32;

        #[cpp(method = "double get_second() const")]
        pub fn get_second(&self) -> f64;

        #[cpp(method = "const char* get_third() const")]
        pub fn get_third(&self) -> *const i8;
    }
}

hicc::import_class! {
    #[cpp(class = "Tuple4", destroy = "tuple4_delete")]
    pub class Tuple4 {
        #[cpp(method = "int get_first() const")]
        pub fn get_first(&self) -> i32;

        #[cpp(method = "double get_second() const")]
        pub fn get_second(&self) -> f64;

        #[cpp(method = "const char* get_third() const")]
        pub fn get_third(&self) -> *const i8;

        #[cpp(method = "int get_fourth() const")]
        pub fn get_fourth(&self) -> i32;
    }
}

hicc::import_lib! {
    #![link_name = "tuple_basic"]

    class Tuple2;
    class Tuple3;
    class Tuple4;

    #[cpp(func = "Tuple2* tuple2_new(int, const char*)")]
    pub unsafe fn tuple2_new(first: i32, second: *const i8) -> Tuple2;

    #[cpp(func = "Tuple3* tuple3_new(int, double, const char*)")]
    pub unsafe fn tuple3_new(first: i32, second: f64, third: *const i8) -> Tuple3;

    #[cpp(func = "Tuple4* tuple4_new(int, double, const char*, int)")]
    pub unsafe fn tuple4_new(first: i32, second: f64, third: *const i8, fourth: i32) -> Tuple4;

    #[cpp(func = "Tuple2* make_int_string_pair(int, const char*)")]
    pub unsafe fn make_int_string_pair(i: i32, s: *const i8) -> *mut Tuple2;

    #[cpp(func = "Tuple3* make_int_double_string(int, double, const char*)")]
    pub unsafe fn make_int_double_string(i: i32, d: f64, s: *const i8) -> *mut Tuple3;
}
