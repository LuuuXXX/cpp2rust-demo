
hicc::cpp! {
    #include <iostream>

    #pragma pack(8)
    class Foo {
    public:
        Foo() {
            std::cout << "Foo::Foo()" << std::endl;
        }
        ~Foo() {
            std::cout << "Foo::~Foo()" << std::endl;
        }
        int foo(int v) {
            std::cout << "foo(" << v << ")" << std::endl;
            return v;
        }
        int foo(int v) const {
            std::cout << "foo(" << v << ") const" << std::endl;
            return v;
        }
        int foo(int v) volatile {
            std::cout << "foo(" << v << ") volatile" << std::endl;
            return v;
        }
        int foo(int v) const volatile {
            std::cout << "foo(" << v << ") const volatile" << std::endl;
            return v;
        }
        Foo baz(int v) && {
            std::cout << "baz(" << v << ") &&" << std::endl;
            return Foo();
        }
        int bar(int v1, int v2 = 0) const {
            std::cout << "bar(" << v1 << ", " << v2 << ") const " << std::endl;
            throw "bar";
            return v1;
        }
    };
}

hicc::import_class! {
    #[cpp(class = "Foo")]
    class Foo {
        #[cpp(method = "int foo(int)")]
        fn foo_mut(&mut self, v: i32) -> i32;

        #[cpp(method = "int foo(int) const")]
        fn foo_const(&self, v: i32) -> i32;

        #[cpp(method = "int foo(int) volatile")]
        fn foo_volatile(&mut self, v: i32) -> i32;

        #[cpp(method = "int foo(int) const volatile")]
        fn foo_const_volatile(&self, v: i32) -> i32;

        #[cpp(method = "Foo baz(int) &&")]
        fn baz(self, v: i32) -> Self;

        #[cpp(method = "int bar(int, int) const")]
        fn bar(&self, v: i32) -> hicc::Exception<()>;
    }
}

hicc::import_lib! {
    #![link_name = "example"]
    
    class Foo;

    #[cpp(func = "std::unique_ptr<Foo> hicc::make_unique<Foo>()")]
    fn foo_new() -> Foo;
}

fn main() {
    let mut foo = foo_new();
    println!("bar return: {:?}", foo.bar(1).ok());
    println!("foo return: {}", foo.foo_mut(1));
    println!("foo_const return: {}", foo.foo_const(2));
    println!("foo_volatile return: {}", foo.foo_volatile(3));
    println!("foo_const_volatile return: {}", foo.foo_const_volatile(4));
    let foo = foo.baz(5);
    foo.foo_const(10);
    foo.baz(20);
}

