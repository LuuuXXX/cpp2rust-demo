use operator_overload::*;

fn main() {
    let a = Number::new(10);
    let b = Number::new(3);
    println!("a+b={}", a.op_add(&b).value());
    println!("a-b={}", a.op_sub(&b).value());
    println!("a*b={}", a.op_mul(&b).value());
    println!("a/b={}", a.op_div(&b).value());
    println!("-a={}", a.op_neg().value());
    println!("compare(a,b)={}", a.compare(&b));
}
