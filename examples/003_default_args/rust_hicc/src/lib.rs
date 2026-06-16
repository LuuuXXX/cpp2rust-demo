hicc::cpp! {
    #include "default_args.h"
}

hicc::import_lib! {
    #![link_name = "default_args"]

    #[cpp(func = "int default_args_ns::greet(const char*, int)")]
    pub unsafe fn greet(name: *const i8, times: i32) -> i32;
}
