# hicc 使用说明（cpp2rust-demo）

`cpp2rust-demo` 生成的 Rust 代码统一基于 `hicc`：

- `hicc::cpp!`：引入 `*.cpp2rust` 中间件（例如 `a.cpp.cpp2rust`）
- `hicc::import_class!`：映射 C++ 类实例方法
- `hicc::import_lib!`：映射自由函数与静态方法
- `hicc_build::Build`：在 `build.rs` 中驱动适配层生成与编译

## 关键约定

1. 每个 `mod_<group>/include/mod.rs` 都会包含 `hicc::cpp! { #include "*.cpp2rust" }`
2. `build.rs` 会为 `hicc_build::Build` 注入中间件所在目录的 include path，并始终引用 `src/...` 活跃视图路径
3. `merge` 后会生成 `rust/src.2/mod_<group>.rs` 与 `rust/src.2/merged_ffi.rs`，并将 `rust/src` 切换到 `src.2`（因此 `build.rs` 无需改成 `src.2/...`）
4. 当前版本里 `method/global/types/common` 仍以结构占位为主，实际绑定主要来自 `include/class/free`

## Cargo 依赖

```toml
[dependencies]
hicc = "0.2.3"

[build-dependencies]
hicc-build = "0.2.1"
```
