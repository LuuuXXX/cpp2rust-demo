use noexcept_basic::*;

fn main() {
    println!("=== 047_noexcept_basic - noexcept ===\n");

    // noexcept functions
    println!("--- noexcept Functions ---");
    println!("noexcept_add(10, 20) = {}", noexcept_add(10, 20));
    println!("noexcept_multiply(6, 7) = {}", noexcept_multiply(6, 7));
    println!("conditional_abs(-42) = {}", conditional_abs(-42));

    // noexcept move semantics
    println!("\n--- noexcept Move Semantics ---");
    let mut mover1 = noexcept_mover_new(100);
    println!("Original mover created, value = {}", mover1.get_value());
    use hicc::AbiClass;
    let mover2 = unsafe { noexcept_mover_move(&mover1.as_mut_ptr()) };
    println!("Mover moved (noexcept), new value = {}", mover2.get_value());

    println!("\n--- Summary ---");
    println!("1. noexcept declares function won't throw");
    println!("2. Move constructors and move assignment operators often use noexcept");
    println!("3. noexcept move operations have better performance in STL containers");
    println!("4. noexcept functions cannot call potentially throwing functions");
    println!("5. In FFI, noexcept is part of function signature");
}
