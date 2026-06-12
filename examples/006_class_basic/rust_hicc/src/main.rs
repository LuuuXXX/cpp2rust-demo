use class_basic::*;

fn main() {
    let mut counter = counter_new();
    println!("Initial value: {}", counter.get());

    counter.increment();
    counter.increment();
    counter.increment();
    println!("After 3 increments: {}", counter.get());

    counter.decrement();
    println!("After 1 decrement: {}", counter.get());

    println!("\nRust FFI: Basic class operations completed!");
}
