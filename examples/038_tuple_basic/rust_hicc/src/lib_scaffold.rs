hicc::cpp! {
    #include <iostream>
    #include <tuple>
    #include <string>
    #include <cstring>

    #include "tuple_basic.h"
}

hicc::import_class! {
    #[cpp(class = "Tuple2")]
    pub class Tuple2 {
        #[cpp(method = "int get_first() const")]
        pub fn get_first(&self) -> i32;

        #[cpp(method = "const char* get_second() const")]
        pub fn get_second(&self) -> *const i8;
    }
}

hicc::import_class! {
    #[cpp(class = "Tuple3")]
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
    #[cpp(class = "Tuple4")]
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

    #[cpp(func = "std::unique_ptr<Tuple2> std::make_unique<Tuple2>(int, const char*)")]
    pub unsafe fn tuple2_new_2(first: i32, second: *const i8) -> Tuple2;

    #[cpp(func = "std::unique_ptr<Tuple3> std::make_unique<Tuple3>(int, double, const char*)")]
    pub unsafe fn tuple3_new_3(first: i32, second: f64, third: *const i8) -> Tuple3;

    #[cpp(func = "std::unique_ptr<Tuple4> std::make_unique<Tuple4>(int, double, const char*, int)")]
    pub unsafe fn tuple4_new_4(first: i32, second: f64, third: *const i8, fourth: i32) -> Tuple4;
}
