# Changelog

## [0.1.0] — 2026-06-01

### 新增

- **五层测试体系**：L1 黄金文件测试、L2 编译测试、L3 运行测试、L4 端到端 E2E 测试（rapidjson / tinyxml2 / pugixml / sqlite3 / nlohmann_json / fmtlib）、L5 `nm` 符号验证测试。
- **完整 hicc 三段式代码生成**：从 C++ 源文件生成 `hicc::cpp!` / `hicc::import_class!` / `hicc::import_lib!` 三段式 FFI 脚手架。
- **五阶段处理流水线**：
  - Phase 1：编译拦截（`hook.cpp` / `capture.rs`）
  - Phase 2：AST 解析（`ast_parser.rs`，基于 libclang）
  - Phase 3：IR 提取（`extractor/`，输出 `FfiSpec`）
  - Phase 4：后处理（`postprocessor/`，菱形继承 + 运算符重载处理）
  - Phase 5：代码生成（`generator/`，输出 `lib.rs` + 冒烟测试）
  - Phase 6：多 feature 合并（`merger/`，输出可独立编译的 Rust 项目）
- **多 feature 合并支持**：`merge --features a,b,c` 将多个 `.cpp2rust` feature 合并为统一 crate。
- **类型映射**：支持 C++ 原始类型、指针、引用、C 函数指针 → Rust FFI 类型自动映射（遵循 LP64 约定）。
- **关联函数归属**：ctor/dtor/factory 函数自动归属对应 `ClassSpec::associated_fns`。
- **菱形继承处理**：自动去重菱形继承场景下的重复方法绑定。
- **运算符重载处理**：自动识别并标注比较运算符、赋值运算符等绑定类别。
- **冒烟测试生成**：`init` 阶段自动生成 `tests/smoke_test.rs`，验证 FFI 符号可链接。
- **API manifest 输出**：`merge` 后在 Rust 项目根目录生成 `api_manifest.json`，汇总所有导出接口。
- **完整 README / INTRODUCTION 文档**：包含快速入门、类型映射规则、流水线架构说明和 10+ 个示例。
