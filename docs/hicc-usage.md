# hicc 使用说明（cpp2rust-demo）

`cpp2rust-demo` 生成的 Rust 代码统一基于 `hicc`：

- `hicc::cpp!`：引入 `*2rust` 中间件（原后缀后追加 `2rust`）
- `hicc::import_class!`：映射 C++ 类实例方法
- `hicc::import_lib!`：映射自由函数与静态方法
- `hicc_build::Build`：在 `build.rs` 中驱动适配层生成与编译

## 关键约定

1. 每个 `ffi_*.rs` 都会包含一个 `hicc::cpp! { #include "*2rust" }`
2. `build.rs` 会为 `hicc_build::Build` 注入中间件所在目录的 include path
3. `merge` 后会生成单个 `merged_ffi.rs`，仍保持上述约定

## Cargo 依赖

```toml
[dependencies]
hicc = "0.2.3"

[build-dependencies]
hicc-build = "0.2.1"
```
