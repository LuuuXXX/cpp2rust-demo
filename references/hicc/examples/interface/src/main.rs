hicc::cpp! {
    #include <hicc/std/memory.hpp>
    #include <iostream>
    struct Foo {
        virtual ~Foo() {};
        virtual void foo() const = 0;
    };

    struct Bar: public Foo {
        virtual void bar() const = 0;
    };

    struct Baz: public Bar {
        virtual void foo() const override {
            std::cout << "C++ Baz::foo" << std::endl;
        }
        virtual void bar() const override {
            std::cout << "C++ Baz::bar" << std::endl;
        }
        void baz() const {
            std::cout << "C++ Baz::baz" << std::endl;
        }
        ~Baz() {
            std::cout << "C++ Baz::~Baz" << std::endl;
        }
    };
}

hicc::import_class! {
    #[interface]
    class Foo {
        #[cpp(method = "void foo() const")]
        fn foo(&self);
    }

    #[interface]
    class Bar: Foo {
        #[cpp(method = "void bar() const")]
        fn bar(&self);
    }

    #[cpp(class = "Baz", ctor = "Baz()")]
    class Baz: Bar { 
        #[cpp(method = "void baz() const")]
        fn baz(&self);
    }
}

hicc::import_lib! {
    #![link_name = "example"]

    class Baz;

    #[cpp(func = "Baz @make_proxy<Baz>()")]
    #[interface(name = "Bar")]
    fn new_rust_baz(intf: hicc::Interface<Baz>) -> Baz;

    #[cpp(func = "std::unique_ptr<Baz> std::make_unique<Baz>()")]
    fn new_cpp_baz() -> Baz;
}

struct RustBaz;

impl Bar for RustBaz {
    fn bar(&self) {
        println!("Rust Baz::bar");
    }
}

impl Foo for RustBaz {
    fn foo(&self) {
        println!("Rust Baz::foo");
    }
}

impl Drop for RustBaz {
    fn drop(&mut self) {
        println!("Rust Baz::~Baz");
    }
}

fn main() {
    let cpp_baz = new_cpp_baz();
    cpp_baz.foo();
    cpp_baz.bar();
    cpp_baz.baz();

    let rust_baz = new_rust_baz(RustBaz);
    rust_baz.foo();
    rust_baz.bar();
    rust_baz.baz();
}
