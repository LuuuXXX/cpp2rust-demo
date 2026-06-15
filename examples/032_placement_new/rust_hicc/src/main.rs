use placement_new::*;

fn main() {
    println!("=== 032_placement_new - Placement New ===\n");

    // 创建预分配缓冲区
    let capacity = 1024;
    let mut buffer = buffer_new_with_capacity(capacity);
    println!("Buffer created with capacity: {}", capacity);

    let data_ptr = buffer.data();
    println!("Buffer data at: {:?}", data_ptr);

    let buf_capacity = buffer.capacity();
    println!("Buffer capacity: {}", buf_capacity);

    let buf_size = buffer.size();
    println!("Buffer constructed size: {}", buf_size);

    println!("\n--- VectorBuffer Demo ---");

    // VectorBuffer 示例
    let vec_buffer = vector_buffer_new_2(10, 4);
    let elem_size = vec_buffer.element_size();
    println!("VectorBuffer element size: {}", elem_size);

    println!("\nRust FFI: Placement New 模式");
    println!("1. 在预分配内存中构造对象");
    println!("2. 使用 placement new: new (address) Constructor(args)");
    println!("3. 适用于内存池、STL 容器实现");
    println!("4. Rust 需要手动管理内存布局");
}
