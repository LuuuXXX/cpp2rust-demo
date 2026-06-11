//! 001_hello_world 冒烟测试
//!
//! 验证生成的 Rust FFI 绑定可编译、可链接 C++ 实现，且基本行为正确。

use hello_world::*;

#[test]
fn smoke_hello_world_callable() {
    // hello_world() 返回 void，仅验证可调用且不 panic。
    hello_world();
}
