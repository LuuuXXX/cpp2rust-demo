//! 008_class_copy 冒烟测试
//!
//! 验证生成的 Rust FFI 绑定可编译、可链接 C++ 实现，且基本行为正确。

use class_copy::*;
use hicc::AbiClass;

#[test]
fn smoke_buffer_new_with_size() {
    let buf = buffer_new_with_size(5);
    assert_eq!(buf.get_size(), 5, "缓冲区大小应为 5");
}

#[test]
fn smoke_buffer_set_get() {
    let mut buf = buffer_new_with_size(3);
    buf.set(0, 10);
    buf.set(1, 20);
    buf.set(2, 30);
    assert_eq!(buf.get(0), 10);
    assert_eq!(buf.get(1), 20);
    assert_eq!(buf.get(2), 30);
}

#[test]
fn smoke_buffer_copy_independence() {
    let mut buf1 = buffer_new_with_size(3);
    buf1.set(0, 100);
    buf1.set(1, 200);
    buf1.set(2, 300);

    let buf2 = buffer_new_copy(&buf1.as_ptr());
    // Verify copy matches original
    assert_eq!(buf2.get(0), 100);
    assert_eq!(buf2.get(1), 200);
    assert_eq!(buf2.get(2), 300);

    // Modify original — copy should be unchanged
    buf1.set(0, 999);
    assert_eq!(buf2.get(0), 100, "拷贝应独立于原始缓冲区");
}
