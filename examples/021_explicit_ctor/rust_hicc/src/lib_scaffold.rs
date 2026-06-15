// 此文件为 cpp2rust-demo 工具对 021_explicit_ctor 自动生成的支架黄金文件，
// 仅供 L1 golden 测试进行生成准确性验证。
//
// Direct 模式下，工具为 Widget 生成 import_class! + make_unique 工厂，
// 使用 std::make_unique<T>(args) 模板函数绑定（无法直接通过 hicc 导出模板，
// 实际 lib.rs 用手动 C++ 包装函数替代）。
hicc::cpp! {
    #include "explicit_ctor.h"
}

hicc::import_class! {
    #[cpp(class = "Widget")]
    pub class Widget {
        #[cpp(method = "int getValue() const")]
        pub fn get_value(&self) -> i32;
    }
}

hicc::import_lib! {
    #![link_name = "explicit_ctor"]

    class Widget;

    #[cpp(func = "std::unique_ptr<Widget> std::make_unique<Widget>(int)")]
    pub fn widget_new_with_v_i32(v: i32) -> Widget;

    #[cpp(func = "std::unique_ptr<Widget> std::make_unique<Widget>(double)")]
    pub fn widget_new_with_v_f64(v: f64) -> Widget;
}
