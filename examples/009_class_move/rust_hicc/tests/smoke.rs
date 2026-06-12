//! 009_class_move 冒烟测试
//!
//! 验证生成的 Rust FFI 绑定可编译、可链接 C++ 实现，且基本行为正确。

use class_move::*;
use hicc::AbiClass;

#[test]
fn smoke_unique_vector_new() {
    let v = unique_vector_new();
    assert_eq!(v.get_size(), 0, "新创建的 UniqueVector 大小应为 0");
}

#[test]
fn smoke_unique_vector_with_data() {
    unsafe {
        let mut data = vec![10, 20, 30];
        let v = unique_vector_new_with_data(data.as_mut_ptr(), 3);
        assert_eq!(v.get_size(), 3, "大小应为 3");
        assert_eq!(v.get(0), 10);
        assert_eq!(v.get(1), 20);
        assert_eq!(v.get(2), 30);
    }
}

#[test]
fn smoke_unique_vector_move() {
    unsafe {
        let mut data = vec![100, 200, 300];
        let mut src = unique_vector_new_with_data(data.as_mut_ptr(), 3);
        let mut dest = unique_vector_new();

        assert_eq!(dest.get_size(), 0);
        unique_vector_move(&dest.as_mut_ptr(), &src.as_mut_ptr());

        assert_eq!(dest.get_size(), 3, "move 后 dest 大小应为 3");
        assert_eq!(dest.get(0), 100);
        assert_eq!(dest.get(1), 200);
        assert_eq!(dest.get(2), 300);

        // src should be emptied after move
        assert_eq!(src.get_size(), 0, "move 后 src 应为空");
    }
}
