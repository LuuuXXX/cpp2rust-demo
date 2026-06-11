# Changelog

本文件遵循 [Keep a Changelog](https://keepachangelog.com/) 格式。

## [Unreleased]

### 新增

- **模板类具体实例化别名生成（v6 Phase B 续，默认关闭）**：在 `CPP2RUST_GEN_TEMPLATES` 开启时，除既有泛型 `import_class!` 骨架外，新增从用户代码类型用法中发现模板类实例化并在 `import_lib!` 中生成 `class StackInt = Stack<hicc::Pod<i32>>;` 形式的实例化别名：
  - `ffi_model`：`TemplateClassSpec` 新增 `instantiations` 字段，新增 `TemplateInstantiation` 结构（别名 / Rust 实例化目标 / C++ 实参）。
  - `extractor`：新增 `collect_type_usages`（收集类字段、方法/函数签名中的类型字符串）与 `template_spec::collect_instantiations`（平衡尖括号解析 `Name<...>`、POD→`hicc::Pod<T>` 与已导出类映射、去重排序）。
  - `hicc_codegen`：模板类 `#[cpp(class = ...)]` 修正为 hicc 要求的完整模板形式（`template<class T> Stack<T>`，原为裸类名 `Stack`）；`import_lib!` 中输出实例化别名（构造函数/工厂仍以 `cpp2rust-todo[TPL]` 提示手动补充）。
  - 全部生成受 `CPP2RUST_GEN_TEMPLATES` 控制，默认关闭，默认产物逐字节不变（L1 黄金 52/52、L2 基线零变更）；实例化发现仅为数据收集，不影响默认产物。
  - 测试：`template_spec` 新增 6 个单元测试（CamelCase、实参映射、尖括号解析、去重排序等）；`tests/template_gen_tests.rs` 新增实例化别名与模板形式断言。

### 移除

- **FFI 冒烟测试生成功能**：移除 `src/generator/smoke_test_gen.rs` 模块及相关代码，包括：
  - `init` 阶段不再生成 `tests/smoke_test.rs`
  - `project_generator::write_smoke_test` 函数
  - `merge` 阶段不再读取/解析冒烟测试清单
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
