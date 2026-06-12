hicc::cpp! {
    #include <iostream>
    #include <cstring>
    #include <string>

    #include "inheritance_single.h"
}

hicc::import_class! {
    #[cpp(class = "Animal", destroy = "animal_delete")]
    pub class Animal {
        #[cpp(method = "const char* getName() const")]
        pub fn get_name(&self) -> *const i8;

        #[cpp(method = "void speak() const")]
        pub fn speak(&self);
    }
}

hicc::import_class! {
    #[cpp(class = "Dog", destroy = "dog_delete")]
    pub class Dog {
        #[cpp(method = "const char* getName() const")]
        pub fn get_name(&self) -> *const i8;

        #[cpp(method = "void bark() const")]
        pub fn bark(&self);

        #[cpp(method = "void speak() const")]
        pub fn speak(&self);
    }
}

hicc::import_lib! {
    #![link_name = "inheritance_single"]

    class Animal;
    class Dog;

    #[cpp(func = "Animal* animal_new(const char*)")]
    pub unsafe fn animal_new(name: *const i8) -> Animal;

    #[cpp(func = "Dog* dog_new(const char*)")]
    pub unsafe fn dog_new(name: *const i8) -> Dog;
}

pub fn decode_cstr(ptr: *const i8) -> String {
    if ptr.is_null() {
        return String::new();
    }
    unsafe { std::ffi::CStr::from_ptr(ptr) }
        .to_string_lossy()
        .to_string()
}
