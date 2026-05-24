hicc::cpp! {
    #include <iostream>
    static void hello_world() {
        std::cout << "hello world!" << std::endl;
    }
}

hicc::import_lib! {
    #![link_name = "example"]
    
    #[cpp(func = "void hello_world()")]
    fn hello_world();
}

fn main() {
    hello_world();
}
