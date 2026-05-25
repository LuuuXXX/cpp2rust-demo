use std::ffi::CStr;

hicc::cpp! {
    #include <hicc/std/string.hpp>
    #include <hicc/std/memory.hpp>
}

hicc::import_lib! {
    #![link_name = "example"]

    // 对应`c++`的容器类型
    class string = hicc_std::string;
    // 具体容器类型的创建接口.
    #[cpp(func = "size_t hicc::size_of<std::string>()")]
    fn string_size() -> usize;

    #[cpp(func = "hicc::AbiClass<std::string> hicc::placement_new<std::string, const char*>(void*, size_t, const char*&&)")]
    fn cpp_string_ctor(buf: *mut i8, len: usize, s: *const i8) -> &'static mut string;

    fn cpp_string_new<'a>(buf: &'a mut [i8], s: &CStr) -> hicc::ClassRefMut<'a, hicc_std::string> {
        cpp_string_ctor(buf.as_mut_ptr(), buf.len(), s.as_ptr())
    }
}

fn main() {
    println!("string size: {}", string_size());
    let mut buf = [1_i8; 100];
    let rs = cpp_string_new(&mut buf, c"hello");
    println!("rs {:?}", rs.as_cstr());
}
