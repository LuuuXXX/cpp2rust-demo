hicc::cpp! {
    #include <iostream>
    #include <cstring>
    #include <string>

    #include "inheritance_single.h"
}

hicc::import_class! {
    #[cpp(class = "Animal", destroy = "animal_delete")]
    class Animal {
        #[cpp(method = "const char* getName() const")]
        fn get_name(&self) -> *const i8;

        #[cpp(method = "void speak() const")]
        fn speak(&self);
    }
}

hicc::import_class! {
    #[cpp(class = "Dog", destroy = "dog_delete")]
    class Dog {
        #[cpp(method = "const char* getName() const")]
        fn get_name(&self) -> *const i8;

        #[cpp(method = "void bark() const")]
        fn bark(&self);

        #[cpp(method = "void speak() const")]
        fn speak(&self);
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
}

fn main() {
    // Create Animal
    let animal_name = "Generic Animal\0";
    let animal = unsafe { animal_new(animal_name.as_ptr() as *const i8) };

    println!("Animal name: {}", decode_cstr(animal.get_name()));
    animal.speak();

    println!();

    // Create Dog
    let dog_name = "Buddy\0";
    let dog = unsafe { dog_new(dog_name.as_ptr() as *const i8) };

    println!("Dog name: {}", decode_cstr(dog.get_name()));
    dog.speak();  // Call inherited speak method
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

