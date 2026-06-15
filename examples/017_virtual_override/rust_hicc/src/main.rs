use virtual_override::*;

fn main() {
    let b = Base::new();
    let d = Derived::new(6.0);
    println!("base.area={}", b.area());
    println!("derived.area={} value={}", d.area(), d.value());
}
