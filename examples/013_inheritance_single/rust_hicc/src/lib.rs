hicc::cpp! {
    #include <iostream>
    #include <cstring>
    #include <string>

    #include "inheritance_single.h"
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

    #[cpp(func = "std::unique_ptr<Animal> std::make_unique<Animal>(const char*)")]
    pub unsafe fn animal_new_with_n(n: *const i8) -> Animal;

    #[cpp(func = "std::unique_ptr<Dog> std::make_unique<Dog>(const char*)")]
    pub unsafe fn dog_new_with_n(n: *const i8) -> Dog;
}
