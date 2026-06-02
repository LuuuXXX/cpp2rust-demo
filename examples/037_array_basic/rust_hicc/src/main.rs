hicc::cpp! {
    #include <stddef.h>
    #include <iostream>
    #include <array>
    #include <string>
    #include <cstring>

    #include "array_basic.h"
}

hicc::import_class! {
    #[cpp(class = "IntArray5", destroy = "int_array5_delete")]
    pub class IntArray5 {
        #[cpp(method = "size_t size() const")]
        fn size(&self) -> usize;

        #[cpp(method = "bool empty() const")]
        fn empty(&self) -> bool;

        #[cpp(method = "void set(size_t i, int val)")]
        fn set(&mut self, i: usize, val: i32);

        #[cpp(method = "int get(size_t i) const")]
        fn get(&self, i: usize) -> i32;

        #[cpp(method = "int at(size_t i) const")]
        fn at(&self, i: usize) -> i32;

        #[cpp(method = "int* data()")]
        fn data(&mut self) -> *mut i32;
    }
}

hicc::import_class! {
    #[cpp(class = "DoubleArray3", destroy = "double_array3_delete")]
    pub class DoubleArray3 {
        #[cpp(method = "size_t size() const")]
        fn size(&self) -> usize;
    }
}

hicc::import_class! {
    #[cpp(class = "StringArray4", destroy = "string_array4_delete")]
    pub class StringArray4 {
        #[cpp(method = "size_t size() const")]
        fn size(&self) -> usize;
    }
}

hicc::import_lib! {
    #![link_name = "array_basic"]

    class IntArray5;
    class DoubleArray3;
    class StringArray4;

    #[cpp(func = "IntArray5* int_array5_new()")]
    fn int_array5_new() -> IntArray5;

    #[cpp(func = "IntArray5* int_array5_new_from(const int*)")]
    fn int_array5_new_from(values: *const i32) -> IntArray5;

    #[cpp(func = "DoubleArray3* double_array3_new()")]
    fn double_array3_new() -> DoubleArray3;

    #[cpp(func = "DoubleArray3* double_array3_new_from(const double*)")]
    fn double_array3_new_from(values: *const f64) -> DoubleArray3;

    #[cpp(func = "StringArray4* string_array4_new()")]
    fn string_array4_new() -> StringArray4;
}

fn main() {
    println!("=== 037_array_basic - std::array ===\n");

    // IntArray5 demo
    println!("--- IntArray5 Demo ---");
    let mut arr = int_array5_new();

    println!("Size: {}", arr.size());
    println!("Empty: {}", arr.empty());

    // Set elements
    for i in 0..5 {
        arr.set(i, (i * 10) as i32);
    }

    // Access elements
    println!("Elements:");
    for i in 0..5 {
        let val = arr.get(i);
        println!("  [{}] = {}", i, val);
    }

    // at() access
    let val = arr.at(2);
    println!("at(2) = {}", val);

    // data() pointer
    let data_ptr = arr.data();
    println!("Data pointer: {:?}", data_ptr);

    println!();

    // IntArray5 from values
    println!("--- IntArray5 from values Demo ---");
    let values = [1, 2, 3, 4, 5];
    let arr = int_array5_new_from(values.as_ptr());

    println!("Size: {}", arr.size());
    println!("Elements:");
    for i in 0..5 {
        let val = arr.get(i);
        println!("  [{}] = {}", i, val);
    }

    println!("\nRust FFI: std::array 映射");
    println!("1. std::array 是固定大小的数组容器");
    println!("2. 大小在编译时确定（模板参数）");
    println!("3. data() 返回原始指针用于批量访问");
    println!("4. 与 Rust 的 [T; N] 数组语义相似");
}

