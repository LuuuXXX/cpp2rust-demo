use array_basic::*;

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
    let arr = unsafe { int_array5_new_from(values.as_ptr()) };

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

