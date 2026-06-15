//! 008_class_copy 冒烟测试：深拷贝构造的独立性。

use class_copy::*;

#[test]
fn default_ctor_size_zero() {
    let b = Buffer::new();
    assert_eq!(b.size(), 0);
}

#[test]
fn sized_ctor_set_get() {
    let mut b = Buffer::new_2(3);
    assert_eq!(b.size(), 3);
    b.set(0, 10);
    b.set(1, 20);
    b.set(2, 30);
    assert_eq!(b.get(0), 10);
    assert_eq!(b.get(1), 20);
    assert_eq!(b.get(2), 30);
}

#[test]
fn copy_is_independent() {
    let mut b1 = Buffer::new_2(3);
    b1.set(0, 100);
    b1.set(1, 200);
    b1.set(2, 300);

    let b2 = Buffer::from_copy(&b1);
    assert_eq!(b2.get(0), 100);
    assert_eq!(b2.get(1), 200);
    assert_eq!(b2.get(2), 300);

    // 修改原对象，拷贝应保持不变。
    b1.set(0, 999);
    assert_eq!(b1.get(0), 999);
    assert_eq!(b2.get(0), 100, "深拷贝应独立于原始缓冲区");
}
