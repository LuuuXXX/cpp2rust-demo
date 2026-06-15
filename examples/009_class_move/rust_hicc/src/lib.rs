hicc::cpp! {
    #include <iostream>
    #include <cstring>

    #include "class_move.h"
}

hicc::import_class! {
    #[cpp(class = "UniqueVector")]
    pub class UniqueVector {
        #[cpp(method = "int get(int index) const")]
        pub fn get(&self, index: i32) -> i32;

        #[cpp(method = "void set(int index, int value)")]
        pub fn set(&mut self, index: i32, value: i32);

        #[cpp(method = "int getSize() const")]
        pub fn get_size(&self) -> i32;

        #[cpp(method = "void moveFrom(UniqueVector & src)")]
        pub fn move_from(&mut self, src: &mut UniqueVector);
    }
}

hicc::import_lib! {
    #![link_name = "class_move"]

    class UniqueVector;

    #[cpp(func = "std::unique_ptr<UniqueVector> hicc::make_unique<UniqueVector>()")]
    pub fn unique_vector_new() -> UniqueVector;

    #[cpp(func = "std::unique_ptr<UniqueVector> std::make_unique<UniqueVector>(int*, int)")]
    pub unsafe fn unique_vector_new_2(data: *mut i32, size: i32) -> UniqueVector;
}
