//! 008_class_copy 冒烟测试
//!
//! 验证拷贝构造产生独立副本：修改原对象不影响副本。

use class_copy::*;
use hicc::AbiClass;

#[test]
fn smoke_buffer_set_get_roundtrip() {
    let mut buf = buffer_new_with_size(5);
    assert_eq!(buf.get_size(), 5, "buffer 大小应为构造参数");
    for i in 0..5 {
        buf.set(i, (i + 1) * 10);
    }
    for i in 0..5 {
        assert_eq!(buf.get(i), (i + 1) * 10, "set/get 应往返一致");
    }
}

#[test]
fn smoke_buffer_copy_is_independent() {
    let mut buf1 = buffer_new_with_size(5);
    for i in 0..5 {
        buf1.set(i, (i + 1) * 10);
    }
    let buf2 = buffer_new_copy(&buf1.as_ptr());
    assert_eq!(buf2.get(0), 10, "副本应复制原始数据");

    buf1.set(0, 999);
    assert_eq!(buf1.get(0), 999, "修改原对象生效");
    assert_eq!(buf2.get(0), 10, "副本不受原对象修改影响");
}
