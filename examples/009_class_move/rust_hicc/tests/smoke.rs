//! 009_class_move 冒烟测试：移动语义的资源转移与置空。

use class_move::*;

#[test]
fn default_ctor_size_zero() {
    let v = UniqueVector::new();
    assert_eq!(v.size(), 0);
}

#[test]
fn sized_ctor_set_get() {
    let mut v = UniqueVector::new_2(3);
    assert_eq!(v.size(), 3);
    v.set(0, 10);
    v.set(1, 20);
    v.set(2, 30);
    assert_eq!(v.get(0), 10);
    assert_eq!(v.get(1), 20);
    assert_eq!(v.get(2), 30);
}

#[test]
fn move_transfers_and_empties_source() {
    let mut src = UniqueVector::new_2(3);
    src.set(0, 100);
    src.set(1, 200);
    src.set(2, 300);

    let mut dest = UniqueVector::new();
    assert_eq!(dest.size(), 0);

    dest.move_from(&mut src);

    // 资源转移到 dest。
    assert_eq!(dest.size(), 3, "move 后 dest 大小应为 3");
    assert_eq!(dest.get(0), 100);
    assert_eq!(dest.get(1), 200);
    assert_eq!(dest.get(2), 300);

    // src 被置空。
    assert_eq!(src.size(), 0, "move 后 src 应为空");
}
