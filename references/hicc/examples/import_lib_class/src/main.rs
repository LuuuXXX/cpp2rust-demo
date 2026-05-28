hicc::cpp! {
    #include <iostream>

    class Foo {
    public:
        static Foo* new_instance() { return new Foo; }
        static void free_instance(Foo* foo) { delete foo; }
        void bar() const {
            std::cout << "Foo::bar" << std::endl;
        }
    };

    template<typename T>
    class Generic {
    public:
        static Generic<T>* new_instance() { return new Generic<T>; }
        static void free_instance(Generic<T>* g) { delete g; }
        void display() const {
            std::cout << "Generic::display" << std::endl;
        }
    };

    template class Generic<int>;
    template class Generic<double>;

    Generic<int>* hicc_new_generic_int() { return Generic<int>::new_instance(); }
    Generic<double>* hicc_new_generic_double() { return Generic<double>::new_instance(); }
}

hicc::import_lib! {
    #![link_name = "example"]

    class Foo;
    class Generic<T>;

    // Non-generic class: associated fn (no self) gets extracted and generates
    // `impl Foo { fn new() ... }` via #[member] auto-added by class-in-lib
    #[cpp(class = "Foo")]
    class Foo {
        #[cpp(method = "void bar() const")]
        fn bar(&self);

        #[cpp(func = "Foo* Foo::new_instance()")]
        fn new() -> Foo;
    }

    // Generic class: methods (with self) stay in class and generate import_class!
    // Associated functions (no self) must NOT use generic type parameters T
    #[cpp(class = "Generic")]
    class Generic<T> {
        #[cpp(method = "void display() const")]
        fn display(&self);

        // Factory functions with concrete types - no generic params T used
        #[cpp(func = "Generic<int>* hicc_new_generic_int()")]
        fn new()-> Generic<hicc::Pod<i32>>;

        #[cpp(func = "Generic<double>* hicc_new_generic_double()")]
        fn new_double() -> Generic<hicc::Pod<f64>>;
    }
}

fn main() {
    let foo = Foo::new();
    std::mem::forget(foo);

    let gen_int = Generic::<hicc::Pod<i32>>::new();
    std::mem::forget(gen_int);

    let gen_double = Generic::<hicc::Pod<f64>>::new_double();
    std::mem::forget(gen_double);

    println!("All tests passed!");
}
