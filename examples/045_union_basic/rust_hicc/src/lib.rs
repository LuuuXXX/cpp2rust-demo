hicc::cpp! {
    #include <cstddef>
    #include <cstdint>
    #include <iostream>
    #include <cstring>
    #include <memory>

    #include "union_basic.h"

    class IntFloatUnion {
    public:
        union {
            int int_value;
            float float_value;
        } data;
        IntFloatUnion() { data.int_value = 0; }
        int get_int() const { return data.int_value; }
        float get_float() const { return data.float_value; }
        void set_int(int v) { data.int_value = v; }
        void set_float(float v) { data.float_value = v; }
    };

    std::unique_ptr<IntFloatUnion> union_new() {
        return std::make_unique<IntFloatUnion>();
    }

    std::unique_ptr<Variant> variant_new_int(int value) {
        auto v = std::make_unique<Variant>();
        v->set_int(value);
        return v;
    }

    std::unique_ptr<Variant> variant_new_float(float value) {
        auto v = std::make_unique<Variant>();
        v->set_float(value);
        return v;
    }

    std::unique_ptr<Variant> variant_new_string(const char* value) {
        auto v = std::make_unique<Variant>();
        v->set_string(value);
        return v;
    }
}

hicc::import_class! {
    #[cpp(class = "IntFloatUnion")]
    pub class IntFloatUnion {
        #[cpp(method = "int get_int() const")]
        pub fn get_int(&self) -> i32;

        #[cpp(method = "float get_float() const")]
        pub fn get_float(&self) -> f32;

        #[cpp(method = "void set_int(int)")]
        pub fn set_int(&mut self, v: i32);

        #[cpp(method = "void set_float(float)")]
        pub fn set_float(&mut self, v: f32);
    }
}

hicc::import_class! {
    #[cpp(class = "Variant")]
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

    #[cpp(func = "std::unique_ptr<IntFloatUnion> union_new()")]
    pub fn union_new() -> IntFloatUnion;

    #[cpp(func = "std::unique_ptr<Variant> variant_new_int(int)")]
    pub fn variant_new_int(value: i32) -> Variant;

    #[cpp(func = "std::unique_ptr<Variant> variant_new_float(float)")]
    pub fn variant_new_float(value: f32) -> Variant;

    #[cpp(func = "std::unique_ptr<Variant> variant_new_string(const char*)")]
    pub unsafe fn variant_new_string(value: *const i8) -> Variant;
}

pub fn variant_type_name(type_code: i32) -> &'static str {
    match type_code {
        0 => "INT",
        1 => "FLOAT",
        2 => "STRING",
        _ => "Unknown",
    }
}
