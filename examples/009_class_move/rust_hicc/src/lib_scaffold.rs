// 009_class_move 工具默认产物支架（hicc 直出，去 shim）。
//
// 用于 L1 黄金比对：校验 `init` 对「只移动命名空间类」默认生成的 hicc 骨架。
// 默认构造与 `int` 构造派生 make_unique 工厂（`new`/`new_2`）；移动构造/移动赋值
// 为 C++ 内部资源转移语义，由 hicc `Drop` 与成员方法 `move_from` 体现，无需额外工厂。

hicc::cpp! {
    #include "class_move.h"
}

hicc::import_class! {
    #[cpp(class = "class_move_ns::UniqueVector")]
    pub class UniqueVector {
        #[cpp(method = "void set(int index, int value)")]
        pub fn set(&mut self, index: i32, value: i32);

        #[cpp(method = "int get(int index) const")]
        pub fn get(&self, index: i32) -> i32;

        #[cpp(method = "int size() const")]
        pub fn size(&self) -> i32;

        #[cpp(method = "void move_from(class_move_ns::UniqueVector & src)")]
        pub fn move_from(&mut self, src: &mut UniqueVector);

        pub fn new() -> Self { unique_vector_new() }

        pub fn new_2(size: i32) -> Self { unique_vector_new_2(size) }
    }
}

hicc::import_lib! {
    #![link_name = "class_move"]

    #[cpp(func = "std::unique_ptr<class_move_ns::UniqueVector> hicc::make_unique<class_move_ns::UniqueVector>()")]
    pub fn unique_vector_new() -> UniqueVector;

    #[cpp(func = "std::unique_ptr<class_move_ns::UniqueVector> hicc::make_unique<class_move_ns::UniqueVector, int>(int&&)")]
    pub fn unique_vector_new_2(size: i32) -> UniqueVector;
}
