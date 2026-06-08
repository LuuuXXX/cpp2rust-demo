hicc::cpp! {
    #include "hello_world.h"
}

hicc::import_lib! {
    #![link_name = "hello_world"]

    #[cpp(func = "void hello_world()")]
    fn hello_world();
}

fn main() {
    hello_world();
    println!("Rust FFI: hello_world() called successfully!");
}



