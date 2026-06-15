# Delta: examples/ + references/ + tests/ + 文档

**Change ID:** `slim-and-hicc-direct-binding`
**Affects:** `examples/`、`references/`、`tests/`、`README.md`、`DEVELOPMENT.md`、`usage/`、`.github/workflows/ci.yml`

---

## ADDED

### Requirement: examples 改造为 direct 模式

每个 example 的 `cpp/` 去除 extern-C shim 函数，`rust_hicc/` 改用 `#[cpp(method = "...")]` 直接绑 C++ 类方法。

#### Scenario: 单个 example 的 cpp/ 改造
- GIVEN 任一 `examples/{NNN_name}/cpp/` 目录
- WHEN 改造完成后
- THEN `cpp/*.h` 与 `cpp/*.cpp` 中**不**含 `extern "C"` 块、**不**含 `foo_new` / `foo_delete` / `foo_<method>` 形式的自由函数
- AND 仅含原生 C++ 类定义与实现
- AND `cpp/*.h` 仍可被 standalone g++ 编译

#### Scenario: 单个 example 的 rust_hicc/ 改造
- GIVEN 任一 `examples/{NNN_name}/rust_hicc/` 目录
- WHEN 改造完成后
- THEN `src/lib.rs` 使用 `hicc::import_class!` + `#[cpp(method = "...")]` 模式（参考 hicc-usages/006）
- AND `src/lib.rs` **不**含 `import_lib!` 中对 `counter_get` / `counter_inc` 等访问器的绑定
- AND 工厂函数通过 `make_unique<T>` 模式实现（参考 hicc-usages）

#### Scenario: 单个 example 的 tests/smoke.rs 改造
- GIVEN 任一 `examples/{NNN_name}/rust_hicc/tests/smoke.rs`
- WHEN 改造完成后
- THEN 至少含 1 个 `#[test]` 函数
- AND 至少 1 个测试包含 `assert!` / `assert_eq!` 等断言（不只是 `let _ = ...`）
- AND `cargo test --test smoke` 退出码 0

---

### Requirement: references/ 瘦身

`references/` 目录从约 17 MB 瘦身至 ≤ 3 MB。

#### Scenario: 历史快照删除
- GIVEN 改造前的 `references/` 含 `rapidjson-refactoring/`（12 MB）与 `c2rust-demo/`（4 MB）
- WHEN Phase 3 完成
- THEN `references/rapidjson-refactoring/` 与 `references/c2rust-demo/` 已被 `git rm`
- AND `references/` 总体积 ≤ 3 MB

#### Scenario: 5 个 submodule 保留
- GIVEN `.gitmodules` 含 5 个 submodule（tinyxml2 / pugixml / sqlite / nlohmann-json / fmtlib）
- WHEN Phase 3 完成
- THEN 5 个 submodule 路径仍存在
- AND `git submodule status` 显示 5 个 submodule 状态正常

---

### Requirement: 测试体系精简

`tests/` 目录文件数从 16 降至 ≤ 9；删除冗余 e2e 测试。

#### Scenario: 删除冗余 e2e 测试
- GIVEN Phase 4 完成
- THEN `tests/rapidjson_e2e_test.rs` 已删除（依赖 rapidjson-refactoring）
- AND `tests/pugixml_e2e_test.rs` 已删除
- AND `tests/sqlite3_e2e_test.rs` 已删除
- AND `tests/nlohmann_json_e2e_test.rs` 已删除
- AND `tests/fmtlib_e2e_test.rs` 已删除
- AND `tests/multi_feature_e2e_test.rs` 已删除

#### Scenario: 保留的测试文件
- GIVEN Phase 4 完成
- THEN 以下测试保留并通过：
  - `tests/l1_golden_tests.rs`（含 direct 模式新 fixture）
  - `tests/l2_compile_tests.rs`
  - `tests/tinyxml2_e2e_test.rs`（唯一端到端回归）
  - `tests/gen_verify_e2e_test.rs`（缩减为 8 个代表性样本，其余 `#[ignore]`）
  - `tests/dynamic_cast_gen_tests.rs`（单元测试）
  - `tests/proxy_gen_tests.rs`（单元测试）
  - `tests/template_gen_tests.rs`（单元测试）

#### Scenario: gen_verify_e2e_test 缩减
- GIVEN `tests/gen_verify_e2e_test.rs` 含 48 个示例的端到端验证
- WHEN Phase 4 完成
- THEN 仅 8 个代表性样本默认运行：001_hello_world、006_class_basic、013_inheritance_single、024_template_function、029_unique_ptr、034_vector_basic、042_exception_basic、048_summary
- AND 其余 40 项标记 `#[ignore]`，可通过 `--include-ignored` 显式触发

---

### Requirement: 文档瘦身

README.md 从 50 KB 瘦身至 ≤ 20 KB；DEVELOPMENT.md 从 28 KB 瘦身至 ≤ 15 KB。

#### Scenario: README 瘦身
- GIVEN Phase 5 完成
- WHEN 阅读 `README.md`
- THEN 保留章节：标题、工作原理（≤ 200 字）、命令参考、快速开始（Linux/macOS/Windows）、最小示例
- AND 删除章节：shim 工作流、L3-L5 测试说明长篇、踩坑总览（迁移到 `docs/`）
- AND 文件体积 ≤ 20 KB

#### Scenario: DEVELOPMENT.md 瘦身
- GIVEN Phase 5 完成
- WHEN 阅读 `DEVELOPMENT.md`
- THEN 保留章节：架构图、模块说明、如何添加新示例、测试层级说明
- AND 文件体积 ≤ 15 KB

#### Scenario: usage/ 精简
- GIVEN Phase 5 完成
- THEN `usage/verify-rapidjson-ffi.sh` 已删除（references 已删 rapidjson）
- AND `usage/verify-tinyxml2-ffi.sh` 新增（替代品，作为本地验证脚本）
- AND `usage/README.md` 精简

---

## MODIFIED

### Requirement: CI 矩阵裁剪

`.github/workflows/ci.yml` 总 job 数从约 30 降至 ≤ 15。

#### Scenario: 删除已废弃测试对应的 CI job
- GIVEN Phase 5 完成
- WHEN 阅读 `.github/workflows/ci.yml`
- THEN 以下 job 已删除：
  - `l4-rapidjson-e2e*`（Linux/Windows MinGW/Windows MSVC/macOS 共 4 个）
  - `l4-e2e-libraries*`（4 个平台共 4 个）
  - `l4-multi-feature-e2e*`（4 个）
  - `usage-verify-rapidjson-ffi*`（3 个）
  - 部分 `l5-nm-symbols*`（保留 Linux，删除其他平台）

#### Scenario: 保留的 CI job
- GIVEN Phase 5 完成
- THEN 保留以下 job：
  - `build` / `lint` / `unit-tests`（Linux + Windows MinGW）
  - `l1-golden`（Linux + Windows MinGW）
  - `l2-compile`（Linux + Windows MinGW）
  - `l_smoke`
  - `gen-verify`（Linux + Windows MinGW）
  - `tinyxml2_e2e`（Linux）
  - `l5-nm-symbols`（Linux）

#### Scenario: macOS / MSVC 改为手动触发
- GIVEN Phase 5 完成
- WHEN 查看 ci.yml
- THEN macOS 系列 job 与 Windows MSVC 系列 job 改为 `on: workflow_dispatch`
- AND 默认 push/PR 不触发这些 job

---

## REMOVED

### Requirement: shim 工作流作为推荐路径

原 README 中"对纯 C++ 库使用 shim 工作流"章节不再作为推荐路径。

#### Scenario: shim 工作流不再推荐
- GIVEN 改造完成后
- WHEN 用户阅读 README
- THEN 主流程展示 direct 模式（直接绑 C++ 类）
- AND shim 工作流仅作为向后兼容的备选方案在 `docs/direct-vs-shim-binding.md` 中说明

> **注**：shim 模式的代码路径（`extractor` + `hicc_codegen`）**保留**，不删除。只是不再作为推荐文档路径。

---

### Requirement: references/rapidjson-refactoring 与 references/c2rust-demo

这两个历史快照目录已被删除。

#### Scenario: 目录不存在
- GIVEN Phase 3 完成
- THEN `references/rapidjson-refactoring/` 不存在
- AND `references/c2rust-demo/` 不存在
- AND 任何代码 / 测试 / 文档不再引用这两个路径
