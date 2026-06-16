use class_move::*;

fn main() {
    let mut src = UniqueVector::new_2(3);
    for i in 0..3 {
        src.set(i, (i + 1) * 100);
    }
    println!("src size: {} src[0]: {}", src.size(), src.get(0));

    // 移动：把 src 的资源转移到 dest，src 被置空。
    let mut dest = UniqueVector::new();
    println!("dest size before move: {}", dest.size());
    dest.move_from(&mut src);

    println!("dest size after move: {} dest[0]: {}", dest.size(), dest.get(0));
    println!("src size after move: {} (emptied)", src.size());
}
