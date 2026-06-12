# Changelog

本文件遵循 [Keep a Changelog](https://keepachangelog.com/) 格式。

## [Unreleased]

### 变更（v7：高级映射能力默认生成，移除环境变量开关）

- **移除全部 `CPP2RUST_GEN_*` 生成开关**：删除 `CPP2RUST_GEN_TEMPLATES` / `CPP2RUST_GEN_PROXY` / `CPP2RUST_GEN_DYNAMIC_CAST` / `CPP2RUST_GEN_SMOKE` 四个环境变量及相关的 `*_enabled()` / `*_ENV` 基础设施（`hicc_codegen` 的 `templates_enabled` / `proxy_enabled` / `dynamic_cast_enabled` / `env_switch_enabled`）。生成路径由「开/关双路径」收敛为「IR 非空即输出」单路径。`smoke_test_gen` 模块本身保留，但不再受环境变量控制。
- **模板类 / 模板函数 / 实例化别名 / 构造工厂、`@make_proxy`、`@dynamic_cast`、冒烟测试 `tests/smoke.rs` 一律默认生成**，命令签名（`init` + `merge`）与目录结构不变；以「文件级幂等」（已存在的用户改动不覆盖）替代开关。
- **模板骨架以注释形式输出**：因「未实例化的模板没有可链接符号、泛型 `<T>` 不可直接编译」，模板类 / 函数 / 别名 / 工厂默认以**注释骨架**（带 `cpp2rust-todo[TMPL]` 指引）输出，保证工具默认产物始终可通过 L6 gen-verify 编译；`@make_proxy` / `@dynamic_cast` 使用 hicc 内建指令、对接具体类型，默认输出为可编译的活动绑定。
- **行为变更提示**：依赖上述开关「默认关闭、产物逐字节不变」的旧行为不再成立——首次对含模板/接口/多态的示例运行 `init` 会额外生成对应骨架（模板为注释、proxy/dynamic_cast 为活动绑定）。

### 测试 / 文档

- 重写 `tests/{template,proxy,dynamic_cast}_gen_tests.rs`，去掉环境变量串行化（`set_var`/`remove_var`），改为断言默认产物；新增「模板骨架须为注释行」契约断言。
- `tests/l1_golden_tests.rs` 新增 `golden_test_scaffold!` 宏；024 模板函数示例黄金 `lib_scaffold.rs` 更新为注释骨架形式。
- README / INTRODUCTION / hicc.md / DEVELOPMENT 全面对齐「默认生成、无开关」；新增 `docs/plans/v7/`。

### 移除

- **FFI 冒烟测试环境变量控制**：移除 `smoke_test_gen::smoke_enabled` 环境变量开关及相关代码，冒烟测试默认生成且不再受 `CPP2RUST_GEN_SMOKE` 控制。`src/generator/smoke_test_gen.rs` 模块及 `project_generator::write_smoke_test` 函数保留，冒烟测试在 `init` 阶段默认生成。
- 移除以下已废弃的基础设施：
  - `layout::SmokeTestEntry` 结构体及 `ApiManifest::smoke_tests` 字段
  - `layout::parse_smoke_test_entries` 函数
  - `api-manifest.md` 中的冒烟测试章节
  - 测试文件 `tests/l1_smoke_test_gen_tests.rs`
  - `tests/rapidjson_e2e_test.rs` 中两个冒烟测试相关函数
  - CI 中 5 个冒烟测试专属 job（`l1-smoke-test-gen` 四平台 + `smoke-test-cargo-check`）

### 修复

- **`block_parser.rs`**：`parse_class_content` 现在正确处理 `pub class Foo {` 形式（codegen 生成的标准格式），修复跳过所有方法绑定的 bug。

### 优化（源码）

- **`lib_spec.rs`**：将文件头模块注释从日语改为中文，与代码库其他文件保持一致。
- **`init.rs`**：`first_pass_parse` 改为"收集所有失败"策略，解析失败的文件记录为警告并跳过，最终汇总打印失败文件列表，而非遇到第一个失败就中止。
- **`merge.rs`**：消除 `run_single_feature_merge` / `run_multi_feature_merge` 中重复的 `current_dir()` + `find_project_root()` 两行代码，统一在 `run_merge` 中获取后传入各子函数。
- **`hicc_codegen.rs`**：用索引判断代替 `out.ends_with("\n\n") { out.pop() }` 的字符串末尾 hack，通过 `if i + 1 < methods.len()` 有条件添加方法间空行。
- **`type_mapper.rs`**：统一使用 `#[cfg(target_os = "windows")]` 并添加注释说明与 `#[cfg(windows)]` 等价，选择前者以保持平台 cfg 写法一致性。
- **`capture.rs`**：将 `hook_dir()` 中 `[Option; 2].into_iter().flatten()` 改写为 `filter_map(|opt| opt)`，提升可读性。

### 优化（测试）

- **`tests/common/mod.rs`**：`normalize` 函数保留含 `cpp2rust-todo` 的降级标记注释行，不再将其当作普通注释剥除；`assert_valid_hicc_format` 改为块级精确检查，避免跨块的 `fn ` 误判；新增 `assert_contains_todo_tag` 辅助断言。
- **`tests/l1_golden_tests.rs`**：新增 `todo_tag_test!` 宏和 4 个降级标记专项断言（031/039/040/047），直接验证 `cpp2rust-todo[FP]` 注释是否正确生成。
- **`tests/l2_compile_tests.rs`**：`compile_test!` 宏加入 `#[cfg_attr(not(feature = "full-test"), ignore)]` 保护，在无 C++ 工具链的 CI 环境中自动忽略，与 L1 保护策略一致。
- **`tests/l4_merge_integration_tests.rs`**（新增）：新增 merge 集成测试，覆盖 `merge_in_place` 备份/rename、重复运行幂等性、`merge_units` 去重与类提取、降级签名收集以及 `collect_unit_rs_files` 目录扫描，无需 C++ 工具链。

### 优化（错误路径测试）

- **`src/error.rs`**：新增 5 个 `Cpp2RustError::Display` 格式测试，覆盖所有错误变体。
- **`src/layout/io.rs`**：新增 7 个单元测试，覆盖 `save_init_report`、`save_merge_report` 的正常路径和边界情况。

### 优化（文档）

- **`README.md`**：补充 `--output-dir` 多 feature 输出目录结构说明及 CI/CD 集成典型使用场景示例（CMake 构建后导出、交叉编译多平台、GitHub Actions 工件上传）。
- **`docs/INTRODUCTION.md`**：补充 Phase 6（merger）技术细节：`merge_in_place` 原子性 rename 机制、跨翻译单元 `cpp_lines` 去重策略、模板特化分组、冲突检测与报告生成。
- **`DEVELOPMENT.md`**：目录树补全 `extractor/lib_spec.rs`、`extractor/class_spec.rs`、`extractor/cpp_block.rs` 三个子模块；六阶段描述与 CHANGELOG 统一。

---

## [0.1.0] — 2026-06-01

### 新增

- **五层测试体系**：L1 黄金文件测试、L2 编译测试、L3 运行测试、L4 端到端 E2E 测试（rapidjson / tinyxml2 / pugixml / sqlite3 / nlohmann_json / fmtlib）、L5 `nm` 符号验证测试。
- **完整 hicc 三段式代码生成**：从 C++ 源文件生成 `hicc::cpp!` / `hicc::import_class!` / `hicc::import_lib!` 三段式 FFI 脚手架。
- **五阶段处理流水线**：
  - Phase 1：编译拦截（`hook.cpp` / `capture.rs`）
  - Phase 2：AST 解析（`ast_parser.rs`，基于 libclang）
  - Phase 3：IR 提取（`extractor/`，输出 `FfiSpec`）
  - Phase 4：后处理（`postprocessor/`，菱形继承 + 运算符重载处理）
  - Phase 5：代码生成（`generator/`，输出 `lib.rs`）
  - Phase 6：多 feature 合并（`merger/`，输出可独立编译的 Rust 项目）
- **多 feature 合并支持**：`merge --feature a --feature b` 将多个 `.cpp2rust` feature 合并为统一 crate。
- **类型映射**：支持 C++ 原始类型、指针、引用、C 函数指针 → Rust FFI 类型自动映射（遵循 LP64 约定）。
- **关联函数归属**：ctor/dtor/factory 函数自动归属对应 `ClassSpec::associated_fns`。
- **菱形继承处理**：自动去重菱形继承场景下的重复方法绑定。
- **运算符重载处理**：自动识别并标注比较运算符、赋值运算符等绑定类别。
- **API manifest 输出**：`merge` 后生成 `meta/api-manifest.md`，汇总所有导出接口（Markdown 格式）。
- **完整 README / INTRODUCTION 文档**：包含快速入门、类型映射规则、流水线架构说明和 10+ 个示例。
