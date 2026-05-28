hicc::cpp! {
    #include <iostream>
    #include <cstring>
    #include <string>

    class Animal {
    protected:
        std::string name;
    public:
        Animal(const char* n);
        virtual ~Animal();
        const char* getName() const;
        virtual void speak() const;
    };

    class Dog : public Animal {
    public:
        Dog(const char* n);
        ~Dog() override;
        void bark() const;
        void speak() const override;
    };

    Animal::Animal(const char* n) : name(n) {}

    Animal::~Animal() {}

    const char* Animal::getName() const {
        return name.c_str();
    }

    void Animal::speak() const {
        std::cout << name << " makes a sound" << std::endl;
    }

    Dog::Dog(const char* n) : Animal(n) {}

    Dog::~Dog() {}

    void Dog::bark() const {
        std::cout << name << " barks: Woof! Woof!" << std::endl;
    }

    void Dog::speak() const {
        std::cout << name << " barks: Woof! Woof!" << std::endl;
    }

    Animal* animal_new(const char* name) {
        return new Animal(name);
    }

    void animal_delete(Animal* self) {
        delete self;
    }

    Dog* dog_new(const char* name) {
        return new Dog(name);
    }

    void dog_delete(Dog* self) {
        delete self;
    }
}

hicc::import_class! {
    #[cpp(class = "Animal")]
    class Animal {
        #[cpp(method = "const char* getName() const")]
        fn get_name(&self) -> *const i8;

        #[cpp(method = "void speak() const")]
        fn speak(&self);
    }
}

hicc::import_class! {
    #[cpp(class = "Dog")]
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
    unsafe fn animal_new(name: *const i8) -> *mut Animal;

    #[cpp(func = "void animal_delete(Animal* self)")]
    unsafe fn animal_delete(self_: *mut Animal);

    #[cpp(func = "Dog* dog_new(const char*)")]
    unsafe fn dog_new(name: *const i8) -> *mut Dog;

    #[cpp(func = "void dog_delete(Dog* self)")]
    unsafe fn dog_delete(self_: *mut Dog);
}

fn main() {
    // Create Animal
    let animal_name = "Generic Animal\0";
    let animal = unsafe { animal_new(animal_name.as_ptr() as *const i8) };

    println!("Animal name: {}", decode_cstr(animal.get_name()));
    animal.speak();
    unsafe {
        animal_delete(&animal);
    }

    println!();

    // Create Dog
    let dog_name = "Buddy\0";
    let dog = unsafe { dog_new(dog_name.as_ptr() as *const i8) };

    println!("Dog name: {}", decode_cstr(dog.get_name()));
    dog.speak();  // Call inherited speak method
    dog.bark();   // Call Dog's own bark method
    unsafe {
        dog_delete(&dog);
    }

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

