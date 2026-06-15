use class_copy::*;

fn main() {
    let mut b1 = Buffer::new_2(5);
    for i in 0..5 {
        b1.set(i, (i + 1) * 10);
    }

    // 深拷贝：b2 与 b1 内存独立。
    let b2 = Buffer::from_copy(&b1);
    println!("b2 size: {}", b2.size());

    print!("b2 values: ");
    for i in 0..5 {
        print!("{} ", b2.get(i));
    }
    println!();

    // 修改原对象不影响拷贝。
    b1.set(0, 999);
    println!("after b1[0]=999: b1[0]={} b2[0]={} (unchanged)", b1.get(0), b2.get(0));
}
