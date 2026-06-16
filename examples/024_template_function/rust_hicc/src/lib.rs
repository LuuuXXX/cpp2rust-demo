//! 024_template_function: 函数模板（命名空间模板 + 具体实例化包装）。
//!
//! C++ 函数模板（`do_swap<T>`、`max_value<T>`）是「编译期蓝图」，每个实例化
//! （`do_swap<int>`、`do_swap<double>` …）才是独立的具体函数，无法整体绑定到 Rust。
//! hicc 直出仅绑定可链接的非模板锚点 `template_function_anchor()`（见 `lib_scaffold.rs`）。
//! 各类型实例化在手写 `lib.rs` 中以 `hicc::cpp!` 命名包装函数补全：每个包装函数显式实例化
//! 模板（如 `do_swap<int>`），再用 `#[cpp(func = ...)]` 绑定为普通自由函数。

hicc::cpp! {
    #include "template_function.h"

    using namespace template_function_ns;

    // 显式实例化：每个具体类型的包装即一次模板实例化。
    void swap_i32(int* a, int* b) { do_swap<int>(a, b); }
    void swap_f64(double* a, double* b) { do_swap<double>(a, b); }

    int max_i32(int a, int b) { return max_value<int>(a, b); }
    double max_f64(double a, double b) { return max_value<double>(a, b); }
}

hicc::import_lib! {
    #![link_name = "template_function"]

    #[cpp(func = "void swap_i32(int*, int*)")]
    pub unsafe fn swap_i32(a: *mut i32, b: *mut i32);

    #[cpp(func = "void swap_f64(double*, double*)")]
    pub unsafe fn swap_f64(a: *mut f64, b: *mut f64);

    #[cpp(func = "int max_i32(int, int)")]
    pub fn max_i32(a: i32, b: i32) -> i32;

    #[cpp(func = "double max_f64(double, double)")]
    pub fn max_f64(a: f64, b: f64) -> f64;

    #[cpp(func = "int template_function_anchor()")]
    pub fn template_function_anchor() -> i32;
}
