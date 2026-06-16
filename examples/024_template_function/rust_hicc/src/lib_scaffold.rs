// 024_template_function 工具默认产物支架（hicc 直出，去 shim）。
//
// 用于 L1 黄金比对：校验 `init` 对「命名空间函数模板 + 锚点」默认生成的 hicc 骨架。
// 函数模板（do_swap<T>/max_value<T>）必须在使用点按具体类型实例化，无法直接绑定，
// 故工具默认仅绑定可链接的非模板锚点 template_function_anchor()；具体实例化（swap_i32、
// max_i32 等）由手写 `lib.rs` 经 hicc::cpp! 命名包装函数补全。

hicc::cpp! {
    #include "template_function.h"
}

hicc::import_lib! {
    #![link_name = "template_function"]

    #[cpp(func = "int template_function_ns::template_function_anchor()")]
    pub fn template_function_anchor() -> i32;
}
