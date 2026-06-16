// 011_class_const 工具默认产物支架（hicc 直出，去 shim）。
//
// 用于 L1 黄金比对：校验 `init` 对「含 const/非 const 成员函数的命名空间类」默认
// 生成的 hicc 骨架。const 方法映射为 `&self`，非 const 方法映射为 `&mut self`，
// 默认构造派生 make_unique 工厂；本示例无需手写补全（`lib.rs` 与支架一致）。

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
