hicc::cpp!{
    #include <iostream>
    #include <string>
    #include <memory>
    #include <cstdarg>

    std::string foo(const std::string& name) {
        std::cout << "foo(const std::string&) " << name << std::endl;
        return name;
    }

    std::unique_ptr<std::string> foo(std::string&& name) {
        std::cout << "foo(std::string&&) " << name << std::endl;
        return std::make_unique<std::string>(name);
    }

    const std::string& foo(std::string* name) {
        std::cout << "foo(std::string&) " << *name << std::endl;
        return *name; 
    }
}

hicc::import_class! {
    #[cpp(class = "std::string")]
    class string {}
}

hicc::import_lib! {
    #![link_name = "example"]

    class string;

    #[cpp(func = "std::string foo(const std::string&)")]
    fn foo_const(name: &string) -> string;

    #[cpp(func = "std::string foo(const std::string&)")]
    fn foo(name: &string);

    #[cpp(func = "std::unique_ptr<std::string> foo(std::string&&)")]
    fn foo_rr(name: string) -> string;

    #[cpp(func = "const std::string& foo(std::string*)")]
    fn foo_mut(name: &mut string) -> &string;

    #[cpp(func = "std::unique_ptr<std::string> hicc::make_unique<std::string, const char*>(const char*&&)")]
    unsafe fn string_new(s: *const u8) -> string;

    #[cpp(func = "int printf(const char* , ...)")]
    unsafe fn printf(format: *const u8, ...) -> i32; 

    #[cpp(func = "int vprintf(const char* , va_list)")]
    unsafe fn vprintf(format: *const u8, ...) -> i32; 
}

fn main() {
    let cname = b"hello world!\0";

    let name = unsafe { string_new(cname.as_ptr()) };
    let name = foo_const(&name);
    let mut name = foo_rr(name);
    let name = foo_mut(&mut name);
    foo(&name);

    let format = b"printf %s\n\0";
    unsafe { printf()(format.as_ptr(), cname.as_ptr()) };
    unsafe { vprintf()(format.as_ptr(), cname.as_ptr()) };
}

