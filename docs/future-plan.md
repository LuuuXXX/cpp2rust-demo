# 后续功能计划（cpp2rust-demo 工具侧）

本文档列出 **cpp2rust-demo 工具层面**可落地的功能改进及其实现状态。这些特性与 hicc 能力无关，是工具本身的实现局限，原则上可在不改动 hicc 的前提下解决。

> 仅讨论 `ToolConservative` / `ToolLimit` 分类的特性。  
> hicc 本身的 `HiccLimitation` 项（如析构函数、`std::string` 参数等）不在本文档范围内，  
> 需要在 hicc 上游解决，或通过手写 C++ shim 绕过（见 `docs/cpp-features.md` §8.1）。

> **当前状态**：§1～§7 所列各项均已在源码中实现，无待完成的工具层面改进项。

---

## §1  模板类：改进无别名时的提示与引导（✅ 已实现）

工具在接口报告中为每个 `tool_conservative` 的模板跳过项自动生成**可直接复制的 `using` 别名建议**（`SkippedDecl.suggested_alias`）。

同时提供 `cpp2rust-demo suggest-aliases --feature <name>` 子命令，从已有 AST JSON 中提取所有跳过的模板特化，输出建议别名列表到 stdout。

**实现文件**：`src/ast.rs`（`SkippedDecl.suggested_alias`）、`src/codegen.rs`（报告渲染）、`src/main.rs`（`suggest-aliases` 子命令）

---

## §2  多重继承：提取全部 public 基类（✅ 已实现）

`extract_class_body()` 现已遍历 `node.bases` 并提取**所有** public 非虚基类，`ClassIR.bases: Vec<String>` 存储完整列表。`render_import_class()` 以 `, ` 分隔生成 `class C: A, B` 语法。

> **注意**：hicc 本身不支持多重继承运行时语义，生成的 `class C: A, B` 骨架无法直接编译使用；需手写 C++ 委托包装后以单继承绑定。骨架保留是为了完整呈现继承关系。

**实现文件**：`src/ast.rs`（`ClassIR.bases` 提取逻辑）、`src/codegen.rs`（`render_import_class`）

---

## §3  链式类型别名（AliasRegistry 传递性解析）（✅ 已实现）

`AliasRegistry::resolve_transitive()` 在收集完所有别名后执行传递性闭合（transitive closure）：
若 `B → A` 且 `A → GenericFoo<T>`，则 `B` 能正确解锁 `GenericFoo` 对应的模板提取。

算法迭代直到稳定（fixed-point），支持任意链深度。

**实现文件**：`src/ast.rs`（`AliasRegistry::resolve_transitive()`）

---

## §4  Virtual 继承（菱形继承）检测与提示（✅ 已实现）

`BaseSpecifier.is_virtual` 字段读取 clang AST 中的 `isVirtual` 标记。`extract_class_body()` 遇到虚基类时将其加入 `ClassIR.skipped_virtual_bases: Vec<String>` 列表并跳过；接口报告在对应类节追加 `⚠️ Virtual bases (skipped — hicc does not support virtual inheritance)` 警告。

**实现文件**：`src/ast.rs`（`BaseSpecifier.is_virtual`、`ClassIR.skipped_virtual_bases`、`extract_class_body`）、`src/codegen.rs`（接口报告渲染）

---

## §5  `std::string` 参数/返回自动生成 shim 建议（✅ 已实现）

含 `std::string`/`std::function`/函数指针参数或返回值的函数/方法被跳过时，工具自动生成**可直接复制的 C++ shim 函数原型**并写入 `SkippedDecl.suggested_shim`。接口报告的 `Shim Suggestions` 节展示这些建议；C++ shim 原型同时追加到 `meta/operator_shims.hpp`。

**实现文件**：`src/ast.rs`（`SkippedDecl.suggested_shim`、类型识别）、`src/codegen.rs`（报告渲染 + `operator_shims.hpp` 写入）

---

## §6  `--dry-run` 模式（✅ 已实现）

`cpp2rust-demo init --dry-run` 标志：执行真实构建命令与 AST dump，但不写入 `rust/src/` 目录，接口报告输出到 stdout。AST JSON 仍保存到 `ast/` 供调试。

**实现文件**：`src/main.rs`（`InitArgs.dry_run`）

---

## §7  函数指针参数：自动生成接口包装建议（✅ 已实现）

工具识别含 `(*)` 的函数指针参数类型，分类为 `ToolConservative`，并在 `suggested_shim` 中自动生成**纯虚接口类骨架**（`FooHandler`）以及 `@make_proxy` 调用示例，统一在接口报告的 `Shim Suggestions` 节展示。

**实现文件**：`src/ast.rs`（`is_function_pointer_type()`、`categorize_unsupported_type()`、`generate_unsupported_type_shim()`）、`src/codegen.rs`（报告渲染）

---

## 实现状态汇总

| 序号 | 功能 | 状态 |
|------|------|:----:|
| §1 | 模板类别名建议与 `suggest-aliases` 命令 | ✅ 已实现 |
| §2 | 多重继承：提取全部 public 基类 | ✅ 已实现 |
| §3 | 链式别名传递性解析 | ✅ 已实现 |
| §4 | 虚继承检测与提示 | ✅ 已实现 |
| §5 | `std::string` / `std::function` / 函数指针 shim 建议 | ✅ 已实现 |
| §6 | `--dry-run` 模式 | ✅ 已实现 |
| §7 | 函数指针接口建议 | ✅ 已实现 |
