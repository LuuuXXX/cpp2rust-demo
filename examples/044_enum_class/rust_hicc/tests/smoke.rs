//! 044_enum_class 冒烟测试
//!
//! 验证 lib 中导出的位标志组合与检查函数（combine_flags / has_flag）。

use enum_class::*;

const FLAG_READ: u32 = 1;
const FLAG_WRITE: u32 = 2;
const FLAG_EXECUTE: u32 = 4;

#[test]
fn smoke_combine_flags() {
    assert_eq!(combine_flags(FLAG_READ, FLAG_EXECUTE), FLAG_READ | FLAG_EXECUTE);
    assert_eq!(combine_flags(FLAG_READ, FLAG_WRITE), 3);
}

#[test]
fn smoke_has_flag() {
    let flags = FLAG_READ | FLAG_WRITE;
    assert_ne!(has_flag(flags, FLAG_READ), 0);
    assert_ne!(has_flag(flags, FLAG_WRITE), 0);
    assert_eq!(has_flag(flags, FLAG_EXECUTE), 0);
}
