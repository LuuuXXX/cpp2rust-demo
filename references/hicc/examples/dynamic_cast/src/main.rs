hicc::cpp! {
    #include <iostream>

    struct Foo {
        virtual void foo() const {
            std::cout << "Foo::foo" << std::endl;
        }
        virtual ~Foo() {
            std::cout << "Foo::~Foo" << std::endl;
        }
        Foo() {
            std::cout << "Foo::Foo" << std::endl;
        }
    };

    struct Bar: Foo {
        virtual void foo() const override {
            std::cout << "Bar::foo" << std::endl;
        }
        ~Bar() {
            std::cout << "Bar::~Bar" << std::endl;
        }
        Bar() {
            std::cout << "Bar::Bar" << std::endl;
        }
    };

    void foo(const Foo& foo) {
        std::cout << "::foo" << std::endl;
        foo.foo();
    }
}

hicc::import_class! {
    #[interface]
    class FooTrait {
        #[cpp(method = "void foo() const")]
        fn foo(&self);
    }

    #[cpp(class = "Foo", ctor = "Foo()")]
    class Foo: FooTrait {
        #[cpp(func = "const Bar* @dynamic_cast<const Bar*>(const Foo*)")]
        fn as_bar(&self) -> *const Bar;
    }

    #[cpp(class = "Bar", ctor = "Bar()")]
    class Bar: FooTrait { 
        #[cpp(func = "const Foo* @dynamic_cast<const Foo*>(const Bar*)")]
        fn as_foo(&self) -> &Foo;
    }
}

hicc::import_lib! {
    #![link_name = "example"]

    class Foo;
    class Bar;

    #[cpp(func = "std::unique_ptr<Foo> hicc::make_unique<Foo>()")]
    fn foo_new() -> Foo;

    #[cpp(func = "std::unique_ptr<Bar> hicc::make_unique<Bar>()")]
    fn bar_new() -> Bar;

    #[cpp(func = "void foo(const Foo&)")]
    fn foo(foo: &Foo);
}

fn main() {
    let f = foo_new();
    foo(&f);
    let b = f.as_bar();
    println!("null bar = {}", b.is_null());

    let b = bar_new();
    b.foo();
    let f = b.as_foo();
    foo(&f);
}
