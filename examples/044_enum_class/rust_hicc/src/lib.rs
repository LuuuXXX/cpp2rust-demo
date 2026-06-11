hicc::cpp! {
    #include "enum_class.h"
}

hicc::import_lib! {
    #![link_name = "enum_class"]

    #[cpp(func = "unsigned int combine_flags(unsigned int, unsigned int)")]
    pub fn combine_flags(f1: u32, f2: u32) -> u32;

    #[cpp(func = "int has_flag(unsigned int, unsigned int)")]
    pub fn has_flag(flags: u32, flag: u32) -> i32;
}
