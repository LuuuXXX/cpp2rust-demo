use class_static::*;

fn main() {
    counter_reset_instance_count();
    println!("initial live: {}", counter_instance_count());

    let mut c1 = Counter::new();
    let mut c2 = Counter::new();
    c1.increment();
    c1.increment();
    c2.increment();
    println!(
        "live: {} c1: {} c2: {}",
        counter_instance_count(),
        c1.value(),
        c2.value()
    );

    drop(c1);
    println!("after drop c1, live: {}", counter_instance_count());

    drop(c2);
    println!("after drop all, live: {}", counter_instance_count());
}
