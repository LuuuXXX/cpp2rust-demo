hicc::cpp! {
    #include "hello_world.h"
}

hicc::import_lib! {
    #![link_name = "hello_world"]

    #[cpp(func = "void hello_world_ns::hello_world()")]
    pub fn hello_world();
}
