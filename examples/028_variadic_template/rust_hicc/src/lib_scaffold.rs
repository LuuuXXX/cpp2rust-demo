// 028_variadic_template 工具默认产物支架（hicc 直出，去 shim）。
//
// 用于 L1 黄金比对：校验 `init` 对「命名空间可变参数函数模板 + 锚点」默认生成的 hicc 骨架。
// 可变参数模板 sum<Args...> 不可整体绑定（每个实参个数/类型组合是一次独立实例化），
// 故工具默认仅绑定可链接的非模板锚点 variadic_template_anchor()；具体实例化
// （sum_i32_3、sum_f64_3 等）由手写 `lib.rs` 经 hicc::cpp! 命名包装函数补全。

hicc::cpp! {
    #include "variadic_template.h"
}

hicc::import_lib! {
    #![link_name = "variadic_template"]

    #[cpp(func = "int variadic_template_anchor()")]
    pub fn variadic_template_anchor() -> i32;
}
