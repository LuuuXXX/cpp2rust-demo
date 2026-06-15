// 此文件为 cpp2rust-demo 工具对 019_operator_overload 自动生成的支架黄金文件，
// 仅供 L1 golden 测试进行生成准确性验证。
//
// Direct 模式下，工具为 Number 生成 import_class! + make_unique 工厂，
// 使用 std::make_unique<T>(args) 模板函数绑定（无法直接通过 hicc 导出模板，
// 实际 lib.rs 用手动 C++ 包装函数替代）。
hicc::cpp! {
    #include "operator_overload.h"
}

hicc::import_class! {
    #[cpp(class = "Number")]
    pub class Number {
        #[cpp(method = "int getValue() const")]
        pub fn get_value(&self) -> i32;

        #[cpp(method = "int compare(const Number & other) const")]
        pub fn compare(&self, other: &Number) -> i32;
    }
}

hicc::import_lib! {
    #![link_name = "operator_overload"]

    class Number;

    #[cpp(func = "std::unique_ptr<Number> std::make_unique<Number>(int)")]
    pub fn number_new_with_v(v: i32) -> Number;
}
