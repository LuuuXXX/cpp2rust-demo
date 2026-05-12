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
4. 当前版本中：`method` 是 `import_class!` 的唯一承接层（实例方法绑定），`free` 负责自由函数/静态方法；`types` 负责类型语义（含 C++→Rust 映射与查询函数）并进入 merged，`class` 负责类级语义结构（含关系访问函数）并进入 merged，`common/*` 会进入全局 merged 共享语义层（含共享查询函数）；`global` 在本 PR 范围内明确 defer
5. `init --no-link`（`--header-only`）用于 header-only/no-link 场景：生成的 `build.rs` 不会输出 `cargo::rustc-link-lib=<link_name>`
6. constructor/destructor、operator overload、template declarations 当前会被跳过，并在 `init-interface-report.md` 中显示 skipped 原因；virtual/pure virtual 方法按以下规则处理：非纯 virtual 直接提取、全纯虚类生成 `#[interface]` trait、混合类的纯虚方法保守跳过

## Cargo 依赖

```toml
[dependencies]
hicc = "0.2.3"

[build-dependencies]
hicc-build = "0.2.1"
```
