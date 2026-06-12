//! 001_hello_world 冒烟测试
//!
//! 验证生成的 Rust FFI 绑定可编译、可链接 C++ 实现，且基本行为正确。

#[test]
fn smoke_hello_world_call() {
    // hello_world 只打印到 stdout，验证调用不 panic
    hello_world::hello_world();
}
