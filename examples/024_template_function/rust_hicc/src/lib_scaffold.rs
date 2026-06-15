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
