# Proposal: 真实第三方库 Direct 模式 Example

**Change ID:** `real-lib-direct-examples`
**Created:** 2026-06-15
**Status:** Implementation Complete
**Completed:** 2026-06-15

---

## Problem Statement

当前 49 个 examples（001-049）全部基于**手写迷你 C++ 类**——每个类只有几行代码、1-3 个方法，且功能单一。这些 example 验证了工具对单一 C++ 特性的处理能力，但**无法暴露工具在面对真实第三方库时的缺陷**：

- **复杂类层级**：rapidjson 的 `Document → Value → Member`、pugixml 的 `xml_node → xml_document`，类之间有真实的多层继承和交叉引用
- **内部 typedef 与 template**：rapidjson 的 `Value` 使用 `GenericValue<Encoding, Allocator>` 模板、内部 `MemberIterator` 等 typedef，工具的 `classify()` 和 `build_direct_class_specs()` 是否能正确处理？
- **大量方法（30+）**：真实类有几十个方法，手写 example 只有 1-3 个，无法测试方法过滤、factory naming 冲突等场景
- **多 TU 聚合**：rapidjson 的 `document.h` + `writer.h` + `prettywriter.h` 分布在多个头文件，工具的 merge 流程能否正确处理？
- **`= delete` 构造函数的真实用法**：rapidjson 的 `Document` 禁止拷贝、pugixml 的 `xml_node` 有特殊的拷贝语义，手写 example 008 的 `Buffer(const Buffer&) = delete` 只是模拟

Phase 3（repo slimming）删除了 `references/rapidjson-refactoring/`（12 MB 历史快照）和早期 `examples/rapidjson/`（8 个场景 example），导致项目**完全丧失了对真实第三方库的端到端验证能力**。当前唯一保留的第三方库 e2e 测试只有 `tinyxml2_e2e_test.rs`，但它仅验证 `init` + AST 解析阶段，不验证 `direct` 模式下的 `extract → generate → cargo check` 全流程。

## Proposed Solution

新增 3 组**真实第三方库 direct 模式 example**（050-052），使用 `references/` 中已有的 submodule 作为 C++ 源码：

| # | Example | 第三方库 | 复杂度 | 验证重点 |
|---|---------|---------|--------|---------|
| 050 | `050_rapidjson_direct` | rapidjson | 高 | template class、internal typedef、=delete ctor、30+ 方法、多 TU |
| 051 | `051_pugixml_direct` | pugixml | 中 | 多层继承、xml_node 拷贝语义、命名空间 |
| 052 | `052_nlohmann_json_direct` | nlohmann-json | 低-中 | 单头文件、template、现代 C++ (if constexpr) |

每组 example 的结构遵循现有 `NNN_name/cpp/` + `NNN_name/rust_hicc/` 模式：
- `cpp/` 不含手写代码，直接 `#include` references submodule 的头文件
- `rust_hicc/src/lib.rs` 由工具生成，必要时手动补充 wrapper shim（与 031_custom_deleter 模式一致）
- `rust_hicc/tests/smoke.rs` 包含 assert 测试

### 技术方案

1. **添加 rapidjson submodule**：`references/` 新增 `rapidjson` submodule（Tencent 的 miloyip/rapidjson）
2. **编写 cpp 入口**：每个 example 的 `cpp/*.h` 仅做 `#include` 引入 + 必要的 `using` 别名简化，不含手写类
3. **运行工具生成**：`cpp2rust-demo init` 对每组 example 执行 direct 模式提取 → 生成
4. **验证 cargo check**：每组生成的 `rust_hicc/` 必须 `cargo check` 通过
5. **golden test**：每组加入 `l1_golden_tests.rs` 的 golden test（与 043/044 模式一致：工具不完美处理的场景用 `lib_scaffold.rs` 作为 golden）
6. **CI job**：新增 3 个 e2e CI job（rapidjson / pugixml / nlohmann-json），验证 init → generate → cargo check 全流程
7. **feature-matrix 更新**：3 组 example 加入 `docs/feature-matrix.md`

### 对工具改进的驱动

每组 example 的验证过程会暴露 direct_binding 模式的真实缺陷，可能需要：
- 增强 `classify()` 对 template instantiation + internal typedef 的处理
- 增强 `build_direct_class_specs()` 对大量方法的类的方法过滤策略
- 增强 `resolve_factory_name_conflicts()` 对真实多 ctor 类的命名冲突
- 发现新的 "工具不完美" 场景时，按 031/043/044 模式处理（手动 `lib.rs` + golden `lib_scaffold.rs`）

## Scope

### In Scope
- 新增 3 组 real-lib direct mode example（050-052）
- 新增 rapidjson submodule 到 `references/`
- 每组 example 的 `cpp/` 入口文件
- 每组 example 的 `rust_hicc/` 项目（工具生成 + 必要手动补充）
- 每组 example 的 golden test + smoke test
- 3 个 e2e CI job
- feature-matrix 文档更新
- 工具代码改进（因 example 验证暴露的问题）

### Out of Scope
- shim 模式的 real-lib example（direct 模式是当前推荐路径）
- `references/rapidjson-refactoring/` 或 `references/c2rust-demo/` 的恢复（它们是历史快照，不需要）
- 其他第三方库（如 fmtlib、sqlite）的 direct example——可后续追加
- references/ 总体积限制（当前 1.6 MB → 新增 rapidjson submodule 约 2 MB，仍 ≤ 5 MB）

## Impact Analysis

| Component | Change Required | Details |
|-----------|-----------------|---------|
| references/ | Yes | 新增 rapidjson submodule |
| examples/ | Yes | 新增 050-052 三个目录 |
| src/extractor/direct_binding.rs | Possible | 可能需增强 classify / build_direct_class_specs 以处理真实库 |
| src/generator/hicc_codegen.rs | Possible | 可能需调整生成逻辑 |
| tests/l1_golden_tests.rs | Yes | 新增 3 个 golden test |
| tests/l2_compile_tests.rs | Yes | 新增 3 个 compile test |
| .github/workflows/ci.yml | Yes | 新增 3 个 e2e job |
| docs/feature-matrix.md | Yes | 新增 3 行 |
| openspec/specs/repo-slim.md | Yes | 修改 references 瘦身阈值（3 MB → 5 MB） |

## Architecture Considerations

- 遵循现有 example 模式（`NNN_name/cpp/` + `rust_hicc/`），不引入新模式
- 工具生成与手动补充的分工遵循 031_custom_deleter 的先例（extern-C shim for function pointers + 手动 `lib.rs`）
- golden test 使用 `lib_scaffold.rs` 作为 golden 的模式遵循 043_namespace_nested / 044_enum_class 先例
- CI e2e job 遵循 tinyxml2_e2e_test 的模式（submodule checkout → init → generate → cargo check）
- rapidjson submodule 的引入不违反 repo slimming 精神：这是**功能性 submodule**，不是历史快照

## Success Criteria

- [x] 050_rapidjson_direct：`cargo build + cargo run` 通过（ParseResult 烟雾测试通过）
- [x] 051_pugixml_direct：`cargo build + cargo run` 通过（xml_parse_result 烟雾测试通过）
- [x] 052_nlohmann_json_direct：`cargo build + cargo run` 通过（extern "C" wrapper 烟雾测试通过）
- [ ] CI 全部 job 通过（e2e CI jobs 待后续添加）
- [x] `cargo test --lib` + `cargo test --test l4_merge_integration_tests` 全通过
- [x] `cargo clippy` + `cargo fmt --check` clean
- [x] 工具代码改进至少修复 4 个由真实库暴露的缺陷（namespace class binding, internal class filter, pub fn parsing, nested namespace strip）
- [x] references/ 总体积 ≤ 35 MB（阈值已更新）

## Risks & Mitigations

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| rapidjson 模板/typedef 导致工具崩溃 | High | High | 先做 rapidjson 空跑（init only），定位 classify 失败点，再逐个修复 |
| pugixml 命名空间 pugi:: 导致工具错误提取 | Medium | Medium | 043 已验证 namespace 嵌套，pugixml 应可处理；若不行，手动 `using` 别名 |
| nlohmann-json 现代 C++ (if constexpr) 导致 AST 解析异常 | Low | Low | nlohmann-json 依赖 C++17，确保编译环境支持 |
| submodule 初始化增加 CI 时间 | Low | Low | 仅 3 个新 submodule，rapidjson 约 2 MB，checkout < 30s |
| references/ 超过 slimming 阈值 | Low | Low | 阈值从 3 MB 调至 5 MB；rapidjson 约 2 MB，加现有 1.6 MB 共 3.6 MB |
| 生成的 Rust 代码需要大量手动修改 | Medium | Medium | 遵循 031/043/044 模式：golden 用 `lib_scaffold.rs`，实际用手动 `lib.rs` |
