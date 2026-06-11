# v6 开发进展记录 — 模板支持（Phase A/B）

> 本文档记录 v6 方案（见 `automated-cpp2rust-ffi-v6.md`）中**模板类 / 模板函数泛型骨架生成**
> 阶段的开发目标、详细方案、详细进展与后续计划。全程使用简体中文。

---

## 1. 开发目标

在 PR #137 完成 **Phase D（冒烟测试生成）** 与部分 **Phase G（文档）** 的基础上，
按 v6 方案继续推进，本阶段聚焦 **Phase A（AST 提取）** 与 **Phase B（提取器 + 生成器）**：

1. **Phase A**：让 AST 解析层不再丢弃模板类（`ClassTemplate`）与模板函数（`FunctionTemplate`）
   的结构化信息，补齐泛型参数、成员方法、参数/返回类型等签名信息。
2. **Phase B**：由提取器构建模板绑定 IR，并由生成器输出**泛型 hicc 骨架**：
   - 模板类 → `import_class!` 中的泛型 `pub class Name<T> { ... }`；
   - 模板函数 → `import_lib!` 中的 `#[cpp(func = "ret name<T>(...)")]`。
3. **硬约束（来自 v6 方案 §1.3 / §9）**：
   - `init` / `merge` 命令、参数、`.cpp2rust/<feature>/` 目录结构完全不变；
   - **默认产物逐字节不变**——新能力通过环境变量开关控制，默认关闭。

---

## 2. 详细方案

### 2.1 总体策略：默认关闭的环境变量开关

为满足「默认产物逐字节不变」的硬约束，引入环境变量 **`CPP2RUST_GEN_TEMPLATES`**：

- 默认关闭，仅当取值为 `1` / `true` / `yes` / `on`（忽略大小写）时启用；
- 关闭时，生成器不输出任何模板相关内容，因此所有既有 L1 黄金 / L2 / L3 / L4 / L5
  基线均不受影响；
- 判定逻辑集中在 `generator::hicc_codegen::templates_enabled()`，常量名
  `GEN_TEMPLATES_ENV`。

提取器侧（`extractor::template_spec`）**始终**构建模板 IR（开销极小），但是否输出由生成器
统一裁决，使 `FfiSpec` 始终携带完整信息、便于测试与未来扩展，同时保证产物不变。

### 2.2 Phase A：AST 层提取

文件：`src/ast_parser/mod.rs`、`src/ast_parser/collector.rs`

- 新增数据结构 `TemplateClassInfo` / `TemplateFunctionInfo`，并在 `CppAst` 中新增
  `template_classes` / `template_functions` 字段（与既有的 `template_class_ranges`
  互补：后者保存源码文本供内联到 `hicc::cpp!`，新结构保存结构化签名）。
- `collect_template_params()`：遍历模板实体子节点，收集 `TemplateTypeParameter` /
  `NonTypeTemplateParameter` / `TemplateTemplateParameter` 的名称（如 `T`、`Allocator`、`N`）。
- `extract_template_class()`：复用既有 `extract_method` 收集成员方法、字段、基类。
- `extract_template_function()`：注意 `FunctionTemplate` 不通过 `get_arguments()` 暴露参数，
  需遍历 `ParmDecl` 子节点提取参数类型。
- 来源过滤：仅纳入来自当前编译单元（`is_from_current_file`）的模板声明，避免把被
  `#include` 的三方库模板纳入绑定（与 v5 对普通类/函数的策略一致）。

### 2.3 Phase B：提取器 + 生成器

- IR（`src/ffi_model.rs`）：新增 `TemplateClassSpec`（name / type_params / methods）与
  `TemplateFnSpec`（name / type_params / cpp_sig / rust_name / params / ret_type）；
  `FfiSpec` 新增 `template_classes` / `template_functions` 字段。
- 提取器（`src/extractor/template_spec.rs`）：`build_template_specs()` 由 `CppAst`
  构建上述 IR。模板类成员方法复用 `class_spec::build_method_binding`（签名中保留泛型 `T`）；
  模板函数按 hicc 要求构建「参数列表只含类型、不含名字」的 `func` 签名，如
  `void do_swap<T>(T*, T*)`。
- 生成器（`src/generator/hicc_codegen.rs`）：`emit_template_class` / `emit_template_fn`
  输出泛型骨架，并附 `cpp2rust-todo[TMPL]` 占位注释，提示用户按实际实例化类型
  校验签名与 `AbiType` 约束（复杂依赖类型如 `T::OutputRef` 由用户补全，符合 v6 §8 降级策略）。

### 2.4 测试

- 单元测试：既有 243 个 lib 单测在 IR 字段扩展后全部通过（修复了若干 `FfiSpec` 测试
  辅助构造）。
- 集成测试 `tests/template_gen_tests.rs`：对一段含模板类 `Stack<T>` 与模板函数
  `do_swap<T>(T*, T*)` 的 C++ 源码，分别验证：
  - **默认关闭**时不输出任何模板骨架与 `cpp2rust-todo[TMPL]` 占位；
  - **开启开关**时输出正确的泛型 `import_class!` / `import_lib!` 骨架。
  因开关为进程级环境变量，断言集中在单个 `#[test]` 中串行执行，避免并发竞态。

---

## 3. 详细进展

| 阶段 | 状态 | 说明 |
|------|------|------|
| Phase A：AST 提取 | ✅ 已完成 | `CppAst.template_classes` / `template_functions` 已落地 |
| Phase B：提取器 + 生成器 | ✅ 已完成 | 泛型 `import_class!` / `import_lib!` 骨架，受 `CPP2RUST_GEN_TEMPLATES` 控制 |
| 单元测试 + 模板生成测试 | ✅ 已完成 | 243 lib 单测 + `template_gen_tests` 全绿 |
| 回归验证（L1 黄金 / lib） | ✅ 已完成 | 52 L1 黄金 + 243 lib 单测全绿，默认产物逐字节不变 |
| 文档（目标 / 方案 / 进展 / 后续） | ✅ 已完成 | 本文档 + README 环境变量表 + DEVELOPMENT.md |

**回归验证结论**：开关默认关闭时，L1 黄金（52）与 lib 单测（243）全部通过，确认默认
产物与改动前逐字节一致；开启开关后模板骨架按预期输出。

---

## 4. 后续计划

以下为 v6 方案中尚未落地的阶段，建议作为后续独立 PR 推进（与本阶段一致：先做风险最低、
可独立验证的垂直切片）：

> **进展更新**：下列「Phase B 增强（实例化别名）」已在后续 PR 落地，详见
> `development-progress-phase-b-plus.md`。

- **Phase B 增强（实例化别名）**：基于 v5 既有的「实例化类型追踪」，为被显式实例化的具体
  类型生成类型别名（如 `pub class VecI32 = vector<hicc::Pod<i32>>;`）与工厂函数，
  使模板骨架可直接用于真实调用。
- **Phase C（高级映射）**：抽象类 → `#[interface]` Trait + 可选 `@make_proxy`；
  RTTI 场景 → `@dynamic_cast`；私有析构 → `destroy = "..."` 属性增强。
- **Phase E（examples 改造）**：将 024/025/026/027 + 虚函数 / STL 选定示例从「手写包装
  降级」升级为「原生 hicc 模板映射」，并为每个改造示例补充 `tests/smoke.rs`；
  同步更新各示例 README。
- **Phase F（测试 / CI）**：L1 黄金适配模板骨架片段；新增 `smoke` job 与 `gen-verify`
  端到端 job（`init` → 生成目录 `cargo test`）。
- **Phase G（剩余文档）**：`docs/INTRODUCTION.md`、`docs/references/hicc.md` 补充模板类 /
  模板函数 / 接口映射章节，与 `reference.md` 对齐。

**风险提示**：Phase E/F 涉及 examples 结构调整（`main.rs` → `lib.rs` + `main.rs`）与
真实库 E2E 基线，需分批改造、逐示例验证黄金，避免回归。模板骨架默认关闭的设计为这些
后续阶段提供了安全的灰度通道：可在开关开启下先验证生成质量，再决定是否纳入黄金基线。
