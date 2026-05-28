hicc::cpp! {
    #include <iostream>

    struct Foo {
        static size_t g_count;

        size_t count = 0;
        Foo() {
            std::cout << "Foo::Foo()" << std::endl;
            ++Foo::g_count;
        }
        ~Foo() {
            std::cout << "Foo::~Foo()" << std::endl;
            --Foo::g_count;
        }
        void foo() {
            ++count;
            std::cout << "Foo::foo()" << std::endl;
        }
        void foo() const {
            std::cout << "Foo::foo()" << std::endl;
        }
    };

    static Foo g_foo;

    size_t Foo::g_count = 0;
}

hicc::import_class! {
    #[cpp(class = "Foo")]
    class Foo {
        #[cpp(method = "void foo()")]
        fn foo_mut(&mut self);

        #[cpp(method = "void foo() const")]
        fn foo_const(&self);

        #[cpp(field = "count")]
        fn count(&self) -> &usize;
    }
}

hicc::import_lib! {
    #![link_name = "example"]

    class Foo;

    #[cpp(data = "Foo::g_count")]
    fn foo_count() -> &'static usize;

    #[cpp(func = "std::unique_ptr<Foo> hicc::make_unique<Foo>()")]
    fn foo_new() -> Foo;

    #[cpp(data = "g_foo")]
    fn static_foo() -> &'static Foo;
}

fn main() {
    println!("Foo::count = {}", *foo_count());
    let mut foo = foo_new();
    println!("Foo::count = {}", *foo_count());
    println!("foo.count = {}", *foo.count());
    foo.foo_mut();
    foo.foo_mut();
    println!("foo.count = {}", *foo.count());
    std::mem::drop(foo);
    println!("Foo::count = {}", *foo_count());

    let foo = static_foo();
    foo.foo_const();
    println!("static_foo.count = {}", *foo.count());
}
