//! 009_class_move 冒烟测试
//!
//! 验证移动语义：资源从 src 转移到 dest 后 src 置空。

use class_move::*;
use hicc::AbiClass;

#[test]
fn smoke_unique_vector_with_data() {
    let mut data = vec![10, 20, 30, 40, 50];
    let v = unsafe { unique_vector_new_with_data(data.as_mut_ptr(), 5) };
    assert_eq!(v.get_size(), 5, "带数据构造后大小应为 5");
    assert_eq!(v.get(0), 10, "首元素应为 10");
}

#[test]
fn smoke_unique_vector_move() {
    let mut data = vec![10, 20, 30, 40, 50];
    let mut src = unsafe { unique_vector_new_with_data(data.as_mut_ptr(), 5) };
    let mut dest = unique_vector_new();
    assert_eq!(dest.get_size(), 0, "移动前 dest 为空");

    unsafe { unique_vector_move(&dest.as_mut_ptr(), &src.as_mut_ptr()) };

    assert_eq!(dest.get_size(), 5, "移动后 dest 接管资源");
    assert_eq!(dest.get(0), 10, "移动后 dest 首元素为 10");
    assert_eq!(src.get_size(), 0, "移动后 src 被置空");
}
