// 019_operator_overload 工具默认产物支架（hicc 直出，去 shim）。
//
// 用于 L1 黄金比对：校验 `init` 对「含运算符重载的命名空间类」默认生成的 hicc 骨架。
// hicc 直出会跳过 operator 重载（由手写 `lib.rs` 经 hicc::cpp! 命名包装函数补全），
// 故默认支架仅含普通方法 value()/compare(&Number) 与构造工厂；完整运算符见手写 `lib.rs`。

hicc::cpp! {
    #include "operator_overload.h"
}

hicc::import_class! {
    #[cpp(class = "operator_overload_ns::Number")]
    pub class Number {
        #[cpp(method = "int value() const")]
        pub fn value(&self) -> i32;

        #[cpp(method = "int compare(const operator_overload_ns::Number & other) const")]
        pub fn compare(&self, other: &Number) -> i32;

        pub fn new(v: i32) -> Self { number_new(v) }
    }
}

hicc::import_lib! {
    #![link_name = "operator_overload"]

    #[cpp(func = "std::unique_ptr<operator_overload_ns::Number> hicc::make_unique<operator_overload_ns::Number, int>(int&&)")]
    pub fn number_new(v: i32) -> Number;
}
