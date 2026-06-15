# Changelog

本文件遵循 [Keep a Changelog](https://keepachangelog.com/) 格式。旧版本归档见 [docs/changelog-archive.md](docs/changelog-archive.md)。

## [Unreleased]

### 变更（v7：高级映射能力默认生成，移除环境变量开关）

- **移除全部 `CPP2RUST_GEN_*` 生成开关**：生成路径收敛为「IR 非空即输出」单路径。
- **模板类/函数/别名/工厂、`@make_proxy`、`@dynamic_cast`、冒烟测试一律默认生成**。模板骨架以注释形式输出（带 `cpp2rust-todo[TMPL]`），proxy/dynamic_cast 为可编译活动绑定。

### 变更（v8：Slim & HICC Direct Binding）

- **移除 rapidjson/pugixml/sqlite3/nlohmann-json/fmtlib E2E 测试及 CI jobs**，保留 tinyxml2 为唯一 L4 E2E 门禁。
- **移除 multi-feature E2E 测试及 CI job**。
- **移除 `usage/verify-rapidjson-ffi.sh`**，新增 `usage/verify-tinyxml2-ffi.sh`。
- **README.md / DEVELOPMENT.md 大幅瘦身**（50KB→7KB / 28KB→7KB），详细内容迁移至 `docs/` 目录。
- **新增 `docs/direct-vs-shim-binding.md`**（两种绑定模式对比）和 `docs/feature-matrix.md`（完整特性矩阵）。
- **CHANGELOG.md 归档旧版本**至 `docs/changelog-archive.md`。
- **CI 简化**：macOS/MSVC 改为 workflow_dispatch；总 job 数 ≤ 15。

### 移除

- `CPP2RUST_GEN_*` 环境变量及 `*_enabled()` / `*_ENV` 基础设施。
- `layout::SmokeTestEntry` 及 `api-manifest.md` 冒烟测试章节。
- `tests/rapidjson_e2e_test.rs`、L4 五大库 E2E、multi-feature E2E。
- `usage/verify-rapidjson-ffi.sh` 及相关 CI jobs。

### 修复

- `block_parser.rs`：`parse_class_content` 正确处理 `pub class Foo {` 形式。

### 优化

- `common/mod.rs`：`normalize` 保留含 `cpp2rust-todo` 的注释行。
- `init.rs`：`first_pass_parse` 改为"收集所有失败"策略。
- `hicc_codegen.rs`：用索引判断代替字符串末尾 hack。
