use inheritance_multiple::*;

fn main() {
    let b1 = Base1::new(10);
    let b2 = Base2::new(20);
    let d = Derived::new(10, 20, 12);
    println!("b1.value1={}", b1.value1());
    println!("b2.value2={}", b2.value2());
    println!("derived={} compute={}", d.derived_value(), d.compute());
}
