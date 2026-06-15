# Repo Structure Specification

**Change ID:** `slim-and-hicc-direct-binding`
**Created:** 2026-06-15

---

## Requirements

### Requirement: examples 改造为 direct 模式

每个 example 的 `cpp/` 去除 extern-C shim 函数，`rust_hicc/` 改用 `#[cpp(method = "...")]` 直接绑 C++ 类方法。

#### Scenario: 单个 example 的 cpp/ 改造
- GIVEN 任一 `examples/{NNN_name}/cpp/` 目录
- WHEN 改造完成后
- THEN `cpp/*.h` 与 `cpp/*.cpp` 中**不**含 `extern "C"` 块、**不**含 shim 自由函数
- AND 仅含原生 C++ 类定义与实现

#### Scenario: 单个 example 的 rust_hicc/ 改造
- GIVEN 任一 `examples/{NNN_name}/rust_hicc/` 目录
- WHEN 改造完成后
- THEN `src/lib.rs` 使用 `hicc::import_class!` + `#[cpp(method)]` 模式
- AND 工厂函数通过 `make_unique<T>` 模式实现

#### Scenario: 单个 example 的 tests/smoke.rs
- THEN 至少含 1 个 `#[test]` 函数
- AND 至少 1 个测试包含 assert 断言

---

### Requirement: references/ 瘦身

`references/` 目录从约 17 MB 瘦身至 ≤ 35 MB（含新增 rapidjson submodule 约 4.4 MB）。

#### Scenario: 历史快照删除
- THEN `references/rapidjson-refactoring/` 与 `references/c2rust-demo/` 已被 `git rm`
- AND `references/` 总体积 ≤ 35 MB

#### Scenario: 5 个 submodule 保留
- THEN 7 个 submodule 路径仍存在（tinyxml2 / pugixml / sqlite / nlohmann-json / fmtlib / rapidjson / hicc）

---

### Requirement: 测试体系精简

`tests/` 目录文件数从 16 降至 ≤ 9。

#### Scenario: 删除冗余 e2e 测试
- THEN 以下已删除：rapidjson_e2e_test.rs, pugixml_e2e_test.rs, sqlite3_e2e_test.rs, nlohmann_json_e2e_test.rs, fmtlib_e2e_test.rs, multi_feature_e2e_test.rs

#### Scenario: 保留的测试文件
- THEN 以下保留：l1_golden_tests.rs, l2_compile_tests.rs, tinyxml2_e2e_test.rs, gen_verify_e2e_test.rs（8 active + 40 #[ignore]）

---

### Requirement: 文档瘦身

README.md ≤ 20 KB；DEVELOPMENT.md ≤ 15 KB。

#### Scenario: README 瘦身
- THEN 保留：标题、工作原理、命令参考、快速开始、最小示例
- AND 删除：shim 工作流、L3-L5 测试说明长篇

#### Scenario: usage/ 精简
- THEN `verify-rapidjson-ffi.sh` 已删除
- AND `verify-tinyxml2-ffi.sh` 新增

---

## Deprecated

### Requirement: shim 工作流作为推荐路径 (Removed: 2026-06-15)

Reason: Direct 模式（make_unique + #[cpp(method)]) 成为推荐路径。Shim 模式保留为向后兼容备选方案，在 docs/direct-vs-shim-binding.md 说明。

### Requirement: references/rapidjson-refactoring 与 references/c2rust-demo (Removed: 2026-06-15)

Reason: 历史快照已删除，references/ 从 17 MB 瘦身至 1.6 MB。

---

### Requirement: CI 矩阵裁剪

`.github/workflows/ci.yml` 总 job 数从约 30 降至 ≤ 15。

#### Scenario: macOS / MSVC 改为手动触发
- THEN macOS 与 MSVC job 改为 `workflow_dispatch`
- AND 默认 push/PR 不触发这些 job
