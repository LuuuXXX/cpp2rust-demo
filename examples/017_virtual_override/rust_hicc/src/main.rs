hicc::cpp! {
    #include <iostream>
    #include <cstring>
    #include <string>

    class Base {
    protected:
        std::string name;
    public:
        Base(const char* n);
        virtual ~Base();
        virtual double area() const;
        const char* getName() const;
    };

    class Derived : public Base {
        double value;
    public:
        Derived(double v);
        ~Derived() override;
        double area() const override;
        double getValue() const;
    };

    Base::Base(const char* n) : name(n) {}

    Base::~Base() {}

    double Base::area() const {
        return 0.0;
    }

    const char* Base::getName() const {
        return name.c_str();
    }

    Derived::Derived(double v) : Base("Derived"), value(v) {}

    Derived::~Derived() {}

    double Derived::area() const {
        return value * value; // area = value^2 for demonstration
    }

    double Derived::getValue() const {
        return value;
    }

    Base* base_create(int type) {
        if (type == 0) {
            std::cout << "Creating Base" << std::endl;
            return new Base("Base");
        } else {
            std::cout << "Creating Derived (as Base*)" << std::endl;
            return new Derived(42.0);
        }
    }

    void base_delete(Base* self) {
        delete self;
    }

    Derived* derived_new(double value) {
        return new Derived(value);
    }

    void derived_delete(Derived* self) {
        delete self;
    }
}

hicc::import_class! {
    #[cpp(class = "Base")]
    class Base {
        #[cpp(method = "double area() const")]
        fn area(&self) -> f64;

        #[cpp(method = "const char* getName() const")]
        fn get_name(&self) -> *const u8;
    }
}

hicc::import_class! {
    #[cpp(class = "Derived")]
    class Derived {
        #[cpp(method = "const char* getName() const")]
        fn get_name(&self) -> *const u8;

        #[cpp(method = "double area() const")]
        fn area(&self) -> f64;

        #[cpp(method = "double getValue() const")]
        fn get_value(&self) -> f64;
    }
}

hicc::import_lib! {
    #![link_name = "virtual_override"]

    class Base;
    class Derived;

    #[cpp(func = "Base* base_create(int)")]
    fn base_create(type_: i32) -> *mut Base;

    #[cpp(func = "void base_delete(Base* self)")]
    unsafe fn base_delete(self_: *mut Base);

    #[cpp(func = "Derived* derived_new(double)")]
    fn derived_new(value: f64) -> *mut Derived;

    #[cpp(func = "void derived_delete(Derived* self)")]
    unsafe fn derived_delete(self_: *mut Derived);
}

fn decode_cstr(ptr: *const u8) -> String {
    if ptr.is_null() {
        return String::new();
    }
    let mut len = 0;
    unsafe {
        while *ptr.add(len) != 0 {
            len += 1;
        }
        String::from_utf8_lossy(std::slice::from_raw_parts(ptr, len)).to_string()
    }
}

fn main() {
    println!("=== Virtual Override FFI with hicc ===\n");
    println!("The 'override' keyword explicitly marks method overriding in C++\n");

    // Create Base
    let base = unsafe { base_create(0) };

    // Create Derived (through base_create returning Base*)
    let derived = unsafe { base_create(1) };

    println!("--- Calling through Base pointer ---");
    println!("Name: {}", decode_cstr(base.get_name()));
    println!("Area: {:.4}", base.area());

    println!();
    println!("--- Calling through Derived (as Base*) ---");
    println!("Name: {}", decode_cstr(derived.get_name()));
    println!("Area: {:.4}", derived.area());

    println!();
    println!("override ensures Derived::area() is called not Base::area()");
    println!("This is polymorphism: same interface, different implementations\n");

    unsafe {
        base_delete(&base);
        // Note: derived is actually Derived*, but we use Base* for deletion
        // In real FFI, we need correct type information
    }

    println!("Rust FFI: override keyword works correctly through hicc!");
}


