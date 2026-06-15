use class_move::*;

fn main() {
    unsafe {
        let mut data = vec![10, 20, 30, 40, 50];
        let mut src_with_data = unique_vector_new_2(data.as_mut_ptr(), 5);

        println!("src_with_data size: {}", src_with_data.get_size());
        println!("src_with_data[0]: {}", src_with_data.get(0));

        let mut dest = unique_vector_new();
        println!("dest size before move: {}", dest.get_size());

        dest.move_from(&mut src_with_data);

        println!("dest size after move: {}", dest.get_size());
        println!("dest[0]: {}", dest.get(0));

        println!("src_with_data size after move: {}", src_with_data.get_size());
    }

    println!("\nRust FFI: Move semantics work!");
}
