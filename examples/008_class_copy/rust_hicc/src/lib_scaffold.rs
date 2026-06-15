// 008_class_copy 工具默认产物支架（hicc 直出，去 shim）。
//
// 用于 L1 黄金比对：校验 `init` 对「含深拷贝构造的命名空间类」默认生成的 hicc 骨架。
// 默认构造与 `int` 构造派生 make_unique 工厂（`new`/`new_2`）；拷贝构造
// `Buffer(const Buffer&)` 被默认排除（不派生工厂），需手写 `lib.rs` 用
// `hicc::make_unique<Buffer, const Buffer&>` 补全，故不在默认支架内。

hicc::cpp! {
    #include "class_copy.h"
}

hicc::import_class! {
    #[cpp(class = "class_copy_ns::Buffer")]
    pub class Buffer {
        #[cpp(method = "void set(int index, int value)")]
        pub fn set(&mut self, index: i32, value: i32);

        #[cpp(method = "int get(int index) const")]
        pub fn get(&self, index: i32) -> i32;

        #[cpp(method = "int size() const")]
        pub fn size(&self) -> i32;

        pub fn new() -> Self { buffer_new() }

        pub fn new_2(sz: i32) -> Self { buffer_new_2(sz) }
    }
}

hicc::import_lib! {
    #![link_name = "class_copy"]

    #[cpp(func = "std::unique_ptr<class_copy_ns::Buffer> hicc::make_unique<class_copy_ns::Buffer>()")]
    pub fn buffer_new() -> Buffer;

    #[cpp(func = "std::unique_ptr<class_copy_ns::Buffer> hicc::make_unique<class_copy_ns::Buffer, int>(int&&)")]
    pub fn buffer_new_2(sz: i32) -> Buffer;
}
