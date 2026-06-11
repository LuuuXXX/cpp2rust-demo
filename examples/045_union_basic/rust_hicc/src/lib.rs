hicc::cpp! {
    #include <cstddef>
    #include <cstdint>
    #include <iostream>
    #include <cstring>

    #include "union_basic.h"
}

hicc::import_class! {
    #[cpp(class = "IntFloatUnion", destroy = "union_delete")]
    pub class IntFloatUnion {}
}

hicc::import_class! {
    #[cpp(class = "Variant", destroy = "variant_delete")]
    pub class Variant {
        #[cpp(method = "int get_type() const")]
        pub fn get_type(&self) -> i32;

        #[cpp(method = "void set_int(int value)")]
        pub fn set_int(&mut self, value: i32);

        #[cpp(method = "void set_float(float value)")]
        pub fn set_float(&mut self, value: f32);

        #[cpp(method = "void set_string(const char* value)")]
        pub fn set_string(&mut self, value: *const i8);

        #[cpp(method = "int get_int() const")]
        pub fn get_int(&self) -> i32;

        #[cpp(method = "float get_float() const")]
        pub fn get_float(&self) -> f32;

        #[cpp(method = "const char* get_string() const")]
        pub fn get_string(&self) -> *const i8;
    }
}

hicc::import_lib! {
    #![link_name = "union_basic"]

    class IntFloatUnion;
    class Variant;

    #[cpp(func = "IntFloatUnion* union_new()")]
    pub fn union_new() -> IntFloatUnion;

    #[cpp(func = "Variant* variant_new_int(int)")]
    pub fn variant_new_int(value: i32) -> Variant;

    #[cpp(func = "Variant* variant_new_float(float)")]
    pub fn variant_new_float(value: f32) -> Variant;

    #[cpp(func = "Variant* variant_new_string(const char*)")]
    pub unsafe fn variant_new_string(value: *const i8) -> Variant;

    #[cpp(func = "int union_get_int(IntFloatUnion* u)")]
    pub fn union_get_int(u: *mut IntFloatUnion) -> i32;

    #[cpp(func = "float union_get_float(IntFloatUnion* u)")]
    pub fn union_get_float(u: *mut IntFloatUnion) -> f32;

    #[cpp(func = "void union_set_int(IntFloatUnion* u, int)")]
    pub unsafe fn union_set_int(u: *mut IntFloatUnion, value: i32);

    #[cpp(func = "void union_set_float(IntFloatUnion* u, float)")]
    pub unsafe fn union_set_float(u: *mut IntFloatUnion, value: f32);
}
