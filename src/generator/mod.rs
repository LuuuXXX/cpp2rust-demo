//! 代码生成器模块（Phase 5）
//!
//! 负责将 `FfiSpec` IR 序列化为可用的 Rust 项目。
//!
//! ## 子模块分工
//!
//! - [`hicc_codegen`]：将单个 `FfiSpec` 渲染为 `hicc` DSL 格式的 `.rs` 源文件，
//!   包含 `cpp! { ... }`、`import_class! { ... }` 和 `import_lib! { ... }` 块。
//! - [`project_generator`]：负责 init 阶段的项目脚手架生成，
//!   创建完整的 Cargo 工程目录结构（`Cargo.toml`、`build.rs`、`src/` 等），
//!   并将各编译单元的 `.rs` 文件写入正确位置。
//! - [`smoke_test_gen`]：生成 `tests/smoke.rs` 冒烟测试，实现"生成即验证"。

pub mod hicc_codegen;
pub mod project_generator;
pub mod smoke_test_gen;
