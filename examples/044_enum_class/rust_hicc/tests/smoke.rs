//! 044_enum_class 冒烟测试
//!
//! 验证生成的 Rust FFI 绑定可编译、可链接 C++ 实现，且强类型枚举操作行为正确。

use enum_class::*;

#[test]
fn smoke_operation_result_state_is_per_object() {
    let mut result = OperationResult::new();
    result.set_error(3);
    assert_eq!(result.get_error(), 3);

    result.set_state(1);
    assert_eq!(result.get_state(), 1);

    result.set_flags(7);
    assert_eq!(result.get_flags(), 7);
}

#[test]
fn smoke_flags_helpers() {
    assert_eq!(combine_flags(1, 2), 3);
    assert_eq!(has_flag(7, 4), 1);
    assert_eq!(has_flag(1, 4), 0);
}
