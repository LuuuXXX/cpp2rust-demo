use class_move::*;
use hicc::AbiClass;

fn main() {
    unsafe {
        // Create source vector with data
        let mut data = vec![10, 20, 30, 40, 50];
        let mut src_with_data = unique_vector_new_with_data(data.as_mut_ptr(), 5);

        println!("src_with_data size: {}", src_with_data.get_size());
        println!("src_with_data[0]: {}", src_with_data.get(0));

        // Create destination vector
        let mut dest = unique_vector_new();
        println!("dest size before move: {}", dest.get_size());

        // Move: transfer resources from src to dest
        unique_vector_move(&dest.as_mut_ptr(), &src_with_data.as_mut_ptr());

        println!("dest size after move: {}", dest.get_size());
        println!("dest[0]: {}", dest.get(0));

        // src should now be empty
        println!("src_with_data size after move: {}", src_with_data.get_size());
    }

    println!("\nRust FFI: Move semantics work!");
}
