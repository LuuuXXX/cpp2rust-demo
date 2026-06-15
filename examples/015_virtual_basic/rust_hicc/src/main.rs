use virtual_basic::*;

fn main() {
    let s = Shape::new();
    let c = Circle::new(2.0);
    println!("shape.area={}", s.area());
    println!("circle.area={} radius={}", c.area(), c.radius());
}
