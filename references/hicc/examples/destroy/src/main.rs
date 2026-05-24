use hicc::AbiClass;

hicc::cpp! {

class Foo {
    ~Foo() {
            std::cout << "Foo::~Foo" << std::endl;
    }
public:
    static Foo* new_instance() { return new Foo; }
    static void free_instance(Foo* foo) { delete foo; }
    void bar() const {
            std::cout << "Foo::bar" << std::endl;
    }
};

}

hicc::import_class! {

    #[cpp(class = "Foo", destroy = "Foo::free_instance")]
    class Foo {
        #[cpp(method = "void bar() const")]
        fn bar(&mut self);
        fn new() -> Self {
            foo_new()
        }
    }
}

hicc::import_lib! {
    #![link_name = "example"]

    #[cpp(func = "Foo* Foo::new_instance()")]
    fn foo_new() -> Foo;
}

fn main() {
    let mut foo = Foo::new();
    foo.bar();
    let mut foo = unsafe { foo.into_unique() };
    foo.bar();
    std::mem::drop(foo);
    println!("exit");
}
