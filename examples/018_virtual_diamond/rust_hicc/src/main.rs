hicc::cpp! {
    #include <iostream>

    class A {
    protected:
        int a_value;
    public:
        A(int v);
        virtual ~A();
        int getAValue() const;
    };

    class B : public virtual A {
    protected:
        int b_value;
    public:
        B(int a, int b);
        ~B() override;
        int getBValue() const;
    };

    class C : public virtual A {
    protected:
        int c_value;
    public:
        C(int a, int c);
        ~C() override;
        int getCValue() const;
    };

    class D : public B, public C {
        int d_value;
    public:
        D(int a, int b, int c, int d);
        ~D() override;
        int getDValue() const;
        void compute() const;
    };

    A::A(int v) : a_value(v) {}

    A::~A() {}

    int A::getAValue() const {
        return a_value;
    }

    B::B(int a, int b) : A(a), b_value(b) {}

    B::~B() {}

    int B::getBValue() const {
        return b_value;
    }

    C::C(int a, int c) : A(a), c_value(c) {}

    C::~C() {}

    int C::getCValue() const {
        return c_value;
    }

    D::D(int a, int b, int c, int d) : A(a), B(a, b), C(a, c), d_value(d) {}

    D::~D() {}

    int D::getDValue() const {
        return d_value;
    }

    void D::compute() const {
        std::cout << "D::compute: a=" << a_value << " b=" << b_value
                  << " c=" << c_value << " d=" << d_value << std::endl;
        std::cout << "Sum: " << (a_value + b_value + c_value + d_value) << std::endl;
    }

    D* d_new(int a, int b, int c, int d) {
        return new D(a, b, c, d);
    }

    void d_delete(D* self) {
        delete self;
    }
}

hicc::import_class! {
    #[cpp(class = "D")]
    class D {
        #[cpp(method = "int getAValue() const")]
        fn get_a_value(&self) -> i32;

        #[cpp(method = "int getBValue() const")]
        fn get_b_value(&self) -> i32;

        #[cpp(method = "int getCValue() const")]
        fn get_c_value(&self) -> i32;

        #[cpp(method = "int getDValue() const")]
        fn get_d_value(&self) -> i32;

        #[cpp(method = "void compute() const")]
        fn compute(&self);
    }
}

hicc::import_lib! {
    #![link_name = "virtual_diamond"]

    class D;

    #[cpp(func = "D* d_new(int, int, int, int)")]
    fn d_new(a: i32, b: i32, c: i32, d: i32) -> *mut D;

    #[cpp(func = "void d_delete(D* self)")]
    unsafe fn d_delete(self_: *mut D);
}

fn main() {
    println!("=== Diamond Inheritance FFI with hicc ===\n");
    println!("Diamond inheritance structure:");
    println!("       A");
    println!("      / \\");
    println!("     B   C");
    println!("      \\ /");
    println!("       D");
    println!();
    println!("Virtual inheritance ensures only ONE A subobject in D\n");

    let mut d = unsafe { d_new(1, 2, 3, 4) };

    println!("Values:");
    println!("  A value (via B): {}", d_get_a_value(&mut d));
    println!("  B value: {}", d.get_b_value());
    println!("  C value: {}", d.get_c_value());
    println!("  D value: {}", d.get_d_value());

    println!();
    d.compute();

    unsafe {
        d_delete(&d);
    }

    println!("\nRust FFI: Diamond inheritance works correctly with hicc!");
}


