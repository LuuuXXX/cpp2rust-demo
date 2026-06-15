use virtual_diamond::*;

fn main() {
    let a = A::new(1);
    let b = B::new(1, 2);
    let c = C::new(1, 3);
    let d = D::new(1, 2, 3, 4);
    println!("a={} b={} c={}", a.a_value(), b.b_value(), c.c_value());
    println!("d_value={} compute={}", d.d_value(), d.compute());
}
