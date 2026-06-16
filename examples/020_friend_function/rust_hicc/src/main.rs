use friend_function::*;

fn main() {
    let a = MyClass::new(10);
    let b = MyClass::new(3);
    println!("a.get_value()={}", a.get_value());
    println!("getSum(a,b)={}", a.friend_sum(&b));
    println!("getProduct(a,b)={}", a.friend_product(&b));
    println!("compare(a,b)={}", a.friend_compare(&b));

    let mut c = MyClass::new(10);
    c.set_value(3);
    println!("compare(c,b) after set_value={}", c.friend_compare(&b));
    println!("--- end main ---");
}
