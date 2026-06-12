// 此文件为 cpp2rust-demo 工具对 024_template_function 自动生成的支架黄金文件，
// 仅供 L1 golden 测试（test_024_template_function）校验工具默认产物的生成准确性。
//
// v7 起，模板函数骨架默认生成，但以**注释**形式输出：函数模板 `do_swap<T>` 会输出一段
// 泛型 `#[cpp(func = ...)]` 注释骨架（带 `cpp2rust-todo[TMPL]` 占位），其中 `<T>` 为泛型
// 占位、并非可直接编译的具体类型，且未实例化的函数模板没有可链接符号。以注释形式呈现，
// 既保证工具默认产物始终可被 Rust 编译器接受（L6 gen-verify），又指引用户按实际实例化
// 类型补全后取消注释。`lib.rs` 则保留可编译、可链接的具体 C 包装函数 swap_int /
// swap_double / ... 供 L2/L3/冒烟测试使用。

hicc::cpp! {
    #include "template_function.h"
}

hicc::import_lib! {
    #![link_name = "template_function"]

    #[cpp(func = "void swap_int(int*, int*)")]
    pub unsafe fn swap_int(a: *mut i32, b: *mut i32);

    #[cpp(func = "void swap_double(double*, double*)")]
    pub unsafe fn swap_double(a: *mut f64, b: *mut f64);

    #[cpp(func = "void swap_char(unsigned char*, unsigned char*)")]
    pub unsafe fn swap_char(a: *mut u8, b: *mut u8);

    #[cpp(func = "void swap_int_array(int*, int, int)")]
    pub unsafe fn swap_int_array(arr: *mut i32, i: i32, j: i32);

    #[cpp(func = "int get_int_array(int*, int)")]
    pub unsafe fn get_int_array(arr: *mut i32, idx: i32) -> i32;

    #[cpp(func = "void set_int_array(int*, int, int)")]
    pub unsafe fn set_int_array(arr: *mut i32, idx: i32, value: i32);

    // cpp2rust-todo[TMPL]: 模板函数骨架（已注释），需按实例化类型声明（如 do_swap<int>(int*, int*)）后取消注释；
    // 下方 <T> 为泛型占位，请替换为实际实例化类型并确认安全性。
    // #[cpp(func = "void do_swap<T>(T*, T*)")]
    // pub unsafe fn do_swap(a: *mut T, b: *mut T);
}
