use class_static::*;
use hicc::AbiClass;

fn main() {
    println!("Initial instance count: {}", counter_get_instance_count());

    let mut c1 = unsafe { counter_new().into_unique() };
    let mut c2 = unsafe { counter_new().into_unique() };
    let c3 = unsafe { counter_new().into_unique() };

    println!("Instance count after creating 3: {}", counter_get_instance_count());

    c1.increment();
    c1.increment();
    c2.increment();

    println!("c1 value: {}", c1.get_value());
    println!("c2 value: {}", c2.get_value());
    println!("c3 value: {}", c3.get_value());

    drop(c1);
    println!("Instance count after deleting c1: {}", counter_get_instance_count());

    drop(c2);
    drop(c3);
    println!("Instance count after deleting all: {}", counter_get_instance_count());

    counter_reset_instance_count();
    println!("Instance count after reset: {}", counter_get_instance_count());

    println!("\nRust FFI: Static members work!");
}
