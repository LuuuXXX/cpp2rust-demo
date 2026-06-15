use virtual_pure::*;

fn main() {
    let c = Circle::new(2.0);
    let r = Rectangle::new(3.0, 4.0);
    println!("circle.area={} radius={}", c.area(), c.radius());
    println!("rect.area={}", r.area());
}
