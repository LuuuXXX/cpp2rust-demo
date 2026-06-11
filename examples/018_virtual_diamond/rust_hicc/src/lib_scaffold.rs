// 此文件为 cpp2rust-demo 工具对 018_virtual_diamond 自动生成的支架黄金文件，
// 仅供 L1 golden 测试（test_018_virtual_diamond）进行生成准确性验证。
//
// 由于 hicc 的 member_addr 用 union 将 16 字节成员函数指针截断为 8 字节 void*，
// 通过虚继承调用 C::getCValue() 时会丢失 this 调整量，导致运行时返回错误值。
// 实际使用的 lib.rs 已在 cpp! 块中添加 d_get_c_value_w 包装函数并相应更新
// import_class! 绑定，以修复该问题。本文件保留工具原始生成结果用于对比验证。
hicc::cpp! {
    #include <iostream>

    #include "virtual_diamond.h"
    int d_get_a_value(D* self) {
        return self->getAValue();
    }

}

hicc::import_class! {
    #[cpp(class = "D", destroy = "d_delete")]
    pub class D {
        #[cpp(method = "int getBValue() const")]
        pub fn get_b_value(&self) -> i32;

        #[cpp(method = "int getCValue() const")]
        pub fn get_c_value(&self) -> i32;

        #[cpp(method = "int getDValue() const")]
        pub fn get_d_value(&self) -> i32;

        #[cpp(method = "void compute() const")]
        pub fn compute(&self);
    }
}

hicc::import_lib! {
    #![link_name = "virtual_diamond"]

    class D;

    #[cpp(func = "D* d_new(int, int, int, int)")]
    pub fn d_new(a: i32, b: i32, c: i32, d: i32) -> D;

    #[cpp(func = "int d_get_a_value(D*)")]
    pub fn d_get_a_value(self_: *mut D) -> i32;
}
