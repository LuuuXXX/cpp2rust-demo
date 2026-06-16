//! 045_union_basic: union 基本操作（命名空间类直接持有 union）。
//!
//! `Variant` / `IntFloatUnion` 直接持有 C++ union，演示 tagged union 与内存 overlay。
//! hicc 直出无需 extern-C 不透明指针 + `*_delete`，析构由 Rust `Drop` 自动完成。

hicc::cpp! {
    #include "union_basic.h"
}

hicc::import_class! {
    #[cpp(class = "union_basic_ns::Variant")]
    pub class Variant {
        #[cpp(method = "void set_int(int)")]
        pub fn set_int(&mut self, v: i32);

        #[cpp(method = "void set_float(float)")]
        pub fn set_float(&mut self, v: f32);

        #[cpp(method = "void set_string(const char*)")]
        pub fn set_string(&mut self, v: *const i8);

        #[cpp(method = "int get_type() const")]
        pub fn get_type(&self) -> i32;

        #[cpp(method = "int get_int() const")]
        pub fn get_int(&self) -> i32;

        #[cpp(method = "float get_float() const")]
        pub fn get_float(&self) -> f32;

        #[cpp(method = "const char* get_string() const")]
        pub fn get_string(&self) -> *const i8;

        pub fn new() -> Self { variant_new() }
    }
}

hicc::import_class! {
    #[cpp(class = "union_basic_ns::IntFloatUnion")]
    pub class IntFloatUnion {
        #[cpp(method = "void set_int(int)")]
        pub fn set_int(&mut self, v: i32);

        #[cpp(method = "void set_float(float)")]
        pub fn set_float(&mut self, v: f32);

        #[cpp(method = "int get_int() const")]
        pub fn get_int(&self) -> i32;

        #[cpp(method = "float get_float() const")]
        pub fn get_float(&self) -> f32;

        pub fn new() -> Self { int_float_union_new() }
    }
}

hicc::import_lib! {
    #![link_name = "union_basic"]

    #[cpp(func = "std::unique_ptr<union_basic_ns::Variant> hicc::make_unique<union_basic_ns::Variant>()")]
    pub fn variant_new() -> Variant;

    #[cpp(func = "std::unique_ptr<union_basic_ns::IntFloatUnion> hicc::make_unique<union_basic_ns::IntFloatUnion>()")]
    pub fn int_float_union_new() -> IntFloatUnion;
}
