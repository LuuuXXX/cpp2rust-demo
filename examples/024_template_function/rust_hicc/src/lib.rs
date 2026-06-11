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
}
