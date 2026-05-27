hicc::cpp! {
    #include <iostream>

    class Base1 {
    protected:
        int value1;
    public:
        Base1(int v);
        virtual ~Base1();
        int getValue1() const;
    };

    class Base2 {
    protected:
        int value2;
    public:
        Base2(int v);
        virtual ~Base2();
        int getValue2() const;
    };

    class Derived : public Base1, public Base2 {
        int derived_value;
    public:
        Derived(int v1, int v2, int dv);
        ~Derived() override;
        int getDerivedValue() const;
        void compute() const;
    };

    Base1::Base1(int v) : value1(v) {}

    Base1::~Base1() {}

    int Base1::getValue1() const {
        return value1;
    }

    Base2::Base2(int v) : value2(v) {}

    Base2::~Base2() {}

    int Base2::getValue2() const {
        return value2;
    }

    Derived::Derived(int v1, int v2, int dv) : Base1(v1), Base2(v2), derived_value(dv) {}

    Derived::~Derived() {}

    int Derived::getDerivedValue() const {
        return derived_value;
    }

    void Derived::compute() const {
        std::cout << "Computing: " << value1 << " + " << value2 << " + " << derived_value
                  << " = " << (value1 + value2 + derived_value) << std::endl;
    }

    Derived* derived_new(int v1, int v2, int dv) {
        return new Derived(v1, v2, dv);
    }

    void derived_delete(Derived* self) {
        delete self;
    }
}

hicc::import_class! {
    #[cpp(class = "Derived")]
    class Derived {
        #[cpp(method = "int getValue1() const")]
        fn get_value1(&self) -> i32;

        #[cpp(method = "int getValue2() const")]
        fn get_value2(&self) -> i32;

        #[cpp(method = "int getDerivedValue() const")]
        fn get_derived_value(&self) -> i32;

        #[cpp(method = "void compute() const")]
        fn compute(&self);
    }
}

hicc::import_lib! {
    #![link_name = "inheritance_multiple"]

    class Derived;

    #[cpp(func = "Derived* derived_new(int, int, int)")]
    fn derived_new(v1: i32, v2: i32, dv: i32) -> *mut Derived;

    #[cpp(func = "void derived_delete(Derived* self)")]
    unsafe fn derived_delete(self_: *mut Derived);
}

fn main() {
    let derived = unsafe { derived_new(10, 20, 30) };

    println!("Base1 value: {}", derived.get_value1());
    println!("Base2 value: {}", derived.get_value2());
    println!("Derived value: {}", derived.get_derived_value());

    derived.compute();

    unsafe {
        derived_delete(&derived);
    }

    println!("\nRust FFI: Multiple inheritance with hicc pattern");
}


