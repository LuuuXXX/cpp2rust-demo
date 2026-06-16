//! 011_class_const: const 成员函数（命名空间类 + make_unique 工厂）。
//!
//! hicc 模式：const 方法（`value`/`history_count`）映射为 `&self`，非 const 方法
//! （`add`/`subtract`/`clear`）映射为 `&mut self`；默认构造派生 make_unique 工厂，
//! 析构由 hicc `Drop` 负责。本示例 `lib.rs` 与工具默认支架一致（无需手写补全）。

hicc::cpp! {
    #include "class_const.h"
}

hicc::import_class! {
    #[cpp(class = "class_const_ns::Calculator")]
    pub class Calculator {
        #[cpp(method = "int value() const")]
        pub fn value(&self) -> i32;

        #[cpp(method = "int history_count() const")]
        pub fn history_count(&self) -> i32;

        #[cpp(method = "void add(int v)")]
        pub fn add(&mut self, v: i32);

        #[cpp(method = "void subtract(int v)")]
        pub fn subtract(&mut self, v: i32);

        #[cpp(method = "void clear()")]
        pub fn clear(&mut self);

        pub fn new() -> Self { calculator_new() }
    }
}

hicc::import_lib! {
    #![link_name = "class_const"]

    #[cpp(func = "std::unique_ptr<class_const_ns::Calculator> hicc::make_unique<class_const_ns::Calculator>()")]
    pub fn calculator_new() -> Calculator;
}
