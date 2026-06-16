//! 009_class_move: 移动语义（命名空间类 + make_unique 工厂）。
//!
//! hicc 模式：默认 / `int` 构造各派生一个 make_unique 工厂；移动构造/移动赋值
//! 是 C++ 内部 O(1) 资源转移语义，经成员方法 `move_from` 暴露（窃取 src 资源、
//! 将 src 置空）。析构由 hicc 的 `Drop` 自动负责。本示例 `lib.rs` 与工具默认
//! 支架 `lib_scaffold.rs` 一致（无需手写补全）。

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
