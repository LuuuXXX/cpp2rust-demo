hicc::cpp! {
    #include <iostream>
    #include <cstring>
    #include <string>

    #include "inheritance_single.h"

    std::unique_ptr<Animal> _cpp2rust_make_unique_animal_with_n(const char* n) { return std::make_unique<Animal>(n); }
    std::unique_ptr<Dog> _cpp2rust_make_unique_dog_with_n(const char* n) { return std::make_unique<Dog>(n); }
}

hicc::import_class! {
    #[cpp(class = "Animal")]
    pub class Animal {
        #[cpp(method = "const char* getName() const")]
        pub fn get_name(&self) -> *const i8;

        #[cpp(method = "void speak() const")]
        pub fn speak(&self);
    }
}

hicc::import_class! {
    #[cpp(class = "Dog")]
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

    #[cpp(func = "std::unique_ptr<Animal> _cpp2rust_make_unique_animal_with_n(const char*)")]
    pub unsafe fn animal_new_with_n(n: *const i8) -> Animal;

    #[cpp(func = "std::unique_ptr<Dog> _cpp2rust_make_unique_dog_with_n(const char*)")]
    pub unsafe fn dog_new_with_n(n: *const i8) -> Dog;
}
