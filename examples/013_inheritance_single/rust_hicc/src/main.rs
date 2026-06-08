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
        fn get_name(&self) -> *const i8;
    }
}

hicc::import_class! {
    #[cpp(class = "Dog", destroy = "dog_delete")]
    pub class Dog {
        #[cpp(method = "const char* getName() const")]
        fn get_name(&self) -> *const i8;

        #[cpp(method = "void bark() const")]
        fn bark(&self);
    }
}

hicc::import_lib! {
    #![link_name = "inheritance_single"]

    class Animal;
    class Dog;

    #[cpp(func = "Animal* animal_new(const char*)")]
    unsafe fn animal_new(name: *const i8) -> Animal;

    #[cpp(func = "Dog* dog_new(const char*)")]
    unsafe fn dog_new(name: *const i8) -> Dog;

    // 虚函数通过 C ABI 包装调用，避免 macOS ARM64 vtable 兼容问题
    #[cpp(func = "void animal_speak(Animal*)")]
    fn animal_speak(self_: *mut Animal);

    #[cpp(func = "void dog_speak(Dog*)")]
    fn dog_speak(self_: *mut Dog);
}

fn main() {
    use hicc::AbiClass;

    // Create Animal
    let animal_name = "Generic Animal\0";
    let mut animal = unsafe { animal_new(animal_name.as_ptr() as *const i8) };

    println!("Animal name: {}", decode_cstr(animal.get_name()));
    animal_speak(&animal.as_mut_ptr());

    println!();

    // Create Dog
    let dog_name = "Buddy\0";
    let mut dog = unsafe { dog_new(dog_name.as_ptr() as *const i8) };

    println!("Dog name: {}", decode_cstr(dog.get_name()));
    dog_speak(&dog.as_mut_ptr());  // Call inherited speak method
    dog.bark();   // Call Dog's own bark method

    println!("\nRust FFI: Single inheritance with hicc pattern");
}

fn decode_cstr(ptr: *const i8) -> String {
    if ptr.is_null() {
        return String::new();
    }
    unsafe { std::ffi::CStr::from_ptr(ptr) }
        .to_string_lossy()
        .to_string()
}

