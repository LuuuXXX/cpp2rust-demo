hicc::cpp! {
    #include "namespace_nested.h"
}

hicc::import_lib! {
    #![link_name = "namespace_nested"]

    #[cpp(func = "void* config_manager_new()")]
    pub fn config_manager_new() -> *mut u8;

    #[cpp(func = "void config_manager_delete(void*)")]
    pub unsafe fn config_manager_delete(self_: *mut u8);

    #[cpp(func = "void config_manager_set_value(void*, const char*, int)")]
    pub unsafe fn config_manager_set_value(self_: *mut u8, key: *const i8, value: i32);

    #[cpp(func = "int config_manager_get_value(void*, const char*)")]
    pub unsafe fn config_manager_get_value(self_: *mut u8, key: *const i8) -> i32;

    #[cpp(func = "int string_length(const char*)")]
    pub unsafe fn string_length(str: *const i8) -> i32;

    #[cpp(func = "void* data_processor_new()")]
    pub fn data_processor_new() -> *mut u8;

    #[cpp(func = "void data_processor_delete(void*)")]
    pub unsafe fn data_processor_delete(self_: *mut u8);

    #[cpp(func = "int data_processor_process(void*, int)")]
    pub unsafe fn data_processor_process(self_: *mut u8, input: i32) -> i32;

    #[cpp(func = "const char* get_version()")]
    pub unsafe fn get_version() -> *const i8;

    #[cpp(func = "int get_build_number()")]
    pub fn get_build_number() -> i32;
}
