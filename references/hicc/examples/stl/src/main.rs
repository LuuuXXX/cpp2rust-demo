use std::ffi::CStr;

hicc::cpp! {
    #include <string>
    static std::string hello_world() {
        return "hello_world";
    }
}

hicc::import_class! {
    #[cpp(class = "std::string")]
    class string {
        #[cpp(method = "const char* c_str() const")]
        fn c_str(&self) -> *const i8;
        fn as_cstr(&self) -> &CStr {
            unsafe { CStr::from_ptr(self.c_str()) } 
        }
    }
}

hicc::import_lib! {
    #![link_name = "example"]

    class string;
    
    #[cpp(func = "std::string hello_world()")]
    fn hello_world() -> string;
}

fn main() {
    let hello = hello_world();
    println!("{:?}", hello.as_cstr());
}

