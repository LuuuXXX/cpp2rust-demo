# 后续功能计划（cpp2rust-demo 工具侧）

本文档列出当前 **cpp2rust-demo 工具层面**可以落地的功能改进——这些特性与 hicc 能力无关，是工具本身的实现局限，原则上可在不改动 hicc 的前提下解决。

> 仅讨论 `ToolConservative` / `ToolLimit` 分类的特性。  
> hicc 本身的 `HiccLimitation` 项（如析构函数、`std::string` 参数等）不在本文档范围内，  
> 需要在 hicc 上游解决，或通过手写 C++ shim 绕过（见 `docs/cpp-features.md` §8.1）。

---

## §1  模板类：改进无别名时的提示与引导

### 现状

当 clang AST 中的 `ClassTemplateSpecializationDecl` 没有对应的 `typedef`/`using` 别名时，
工具直接跳过该类，在接口报告中标记 `tool_conservative`。
用户如果不熟悉 AliasRegistry 机制，往往不知道如何解锁。

### 改进目标

1. 在接口报告中，为每个 `tool_conservative` 的模板类自动生成**可直接复制到 entry.cpp 的 using 别名建议**，例如：
   ```
   // Suggested alias for skipped ClassTemplateSpecializationDecl:
   // using Foo = ns::GenericFoo<ns::UTF8<char>>;
   ```
2. 提供 `cpp2rust-demo suggest-aliases --feature <name>` 子命令，从已有 AST 中提取所有跳过的模板特化，输出建议别名列表到 stdout。

### 落地方案

1. **修改 `ast.rs`**：在 `extract_class_body()` 的 `tool_conservative` 跳过分支中，收集特化的完整类型名 `qual_type` 和裸模板名，写入 `SkippedDecl` 的新字段 `suggested_alias: Option<String>`。
2. **修改 `codegen.rs`**（接口报告渲染）：在 `Skipped declarations` 节追加 `Suggested alias` 列，显示工具生成的 `using` 建议。
3. **新增 `suggest-aliases` 子命令**（`main.rs`）：读取 `meta/init-interface-report.md` 或直接读 AST JSON，输出 `using <Alias> = <FullType>;` 格式的建议行。

### 影响范围

- `src/ast.rs`（`SkippedDecl` 结构体 + 提取逻辑）
- `src/codegen.rs`（报告渲染）
- `src/main.rs`（新子命令）
- 对现有输出无破坏性变更

---

## §2  多重继承：提取全部 public 基类

### 现状

`extract_class_body()` 在遍历 `node.bases` 时，只取**第一个** public 基类，其余忽略。
例如 `class C: public A, public B`，只生成 `class C: A`，`B` 被丢弃。

hicc `import_class!` 的 `class Derived: Base1, Base2, ...` 语法（多 trait 继承）理论上是否支持，
需要查阅 hicc 最新版 API。如果 hicc 支持多基类，则工具侧直接扩展即可。

### 改进目标

1. 提取所有 `access == "public"` 的基类，按声明顺序排列。
2. 在生成的 `import_class!` 中使用 `class C: A + B`（若 hicc 支持）或分别生成多个 trait bound。
3. 对当前被忽略的额外基类，在接口报告中追加 `secondary_bases` 字段和警告。

### 落地方案

1. **修改 `ast.rs`** 中 `ClassIR` 结构体：将 `bases: Vec<String>` 已有字段改为存储全部 public 基类（当前逻辑只 push 第一个），移除 early `break`。
2. **修改 `codegen.rs`** 中 `render_import_class()`：若 `bases.len() > 1`，先确认 hicc 语法，生成 `class Foo: Base1 + Base2` 或多个 `class` 块（作为临时方案）。
3. **接口报告**：列出 `secondary_bases`，注明当前 hicc 支持状态。

### 影响范围

- `src/ast.rs`（`ClassIR.bases` 提取逻辑）
- `src/codegen.rs`（import_class 渲染）
- 需验证 hicc 当前版本对多基类语法的支持情况

---

## §3  链式类型别名（AliasRegistry 传递性解析）

### 现状

`AliasRegistry` 在收集 `TypedefDecl`/`TypeAliasDecl` 时只做**单层**映射：
```
using A = GenericFoo<T>;   ← 注册 "GenericFoo" → "A"
using B = A;               ← 只记录 "A" → "B"，但不解析 "B" 是否对应 "GenericFoo"
```
当代码中存在 `using B = A;` 的链式别名时，`B` 无法解锁模板提取。

### 改进目标

1. AliasRegistry 在所有 alias 收集完毕后，执行**传递性关闭**（transitive closure）：
   若 `B → A` 且 `A → GenericFoo`，则补充注册 `B → GenericFoo`。
2. 同时在 `alias_to_type` 中补充 `B → A 的完整类型` 的映射。

### 落地方案

1. **修改 `ast.rs`** 中 `AliasRegistry::collect_from_ast()` 结束后，增加 `resolve_transitive()` 方法：
   ```
   for each (alias, target_type) in alias_to_type:
       bare = bare_template_name(target_type)
       if template_to_alias.contains(bare):
           # alias 已是某模板的别名，无需再追踪
           continue
       # target_type 本身是一个别名名
       if alias_to_type.contains(target_type):
           real_type = alias_to_type[target_type]
           alias_to_type[alias] = real_type         // 传递性更新
           bare2 = bare_template_name(real_type)
           if not template_to_alias.contains(bare2):
               template_to_alias[bare2] = alias      // 注册链式别名到模板名映射
   ```
2. 该方法在 `collect_from_ast()` 最后调用，最多循环 N 轮直到稳定（N = 最大链深度，通常 ≤ 5）。

### 影响范围

- `src/ast.rs`（`AliasRegistry` 结构体 + `collect_from_ast`）
- 对现有输出无破坏性变更；仅扩展解锁能力

---

## §4  Virtual 继承（菱形继承）检测与提示

### 现状

C++ 菱形继承使用 `virtual` 基类：
```cpp
class A { ... };
class B: virtual public A { ... };
class C: virtual public A { ... };
class D: public B, public C { ... };  // 菱形
```
当前工具只取首个 public 基类，对 `virtual` 基类标记无特殊处理，
生成错误或不完整的继承链。

### 改进目标

1. 检测 `node.bases` 中的 `isVirtual: true` 标记，在接口报告中明确提示"此类使用 virtual 继承，当前工具不完整支持"。
2. 仍然生成首个非虚基类的绑定（保持现有行为），同时报告跳过的虚基类列表。
3. （可选扩展）若 hicc 在未来支持菱形继承语义，工具可在此基础上直接启用。

### 落地方案

1. **修改 `ast.rs`** 中 `BaseSpecifier` 结构体：添加 `is_virtual: bool` 字段，读取 clang AST 中的 `isVirtual` 字段。
2. **修改 `extract_class_body()`**：在处理 `bases` 时，若 `is_virtual == true`，将该基类加入 `skipped_virtual_bases: Vec<String>` 列表（新字段）并跳过；否则正常处理。
3. **修改接口报告渲染**：在 `ClassIR` 节追加 `Virtual bases (skipped)` 子节，列出跳过的虚基类。

### 影响范围

- `src/ast.rs`（`BaseSpecifier` + `ClassIR` + `extract_class_body`）
- `src/codegen.rs`（接口报告渲染）

---

## §5  `std::string` 参数/返回自动生成 shim 建议

### 现状

含 `std::string` 参数或返回值的函数/方法，被工具跳过并标记 `hicc_limitation`。
用户需要手工编写 C++ shim 将其转换为 `const char*`。

### 改进目标

工具在接口报告中，对每个含 `std::string` 的跳过项自动生成**可直接复制使用的 C++ shim 函数原型**，例如：
```cpp
// Suggested shim for: std::string Document::getTitle() const
static inline const char* document_get_title(const Document& self) {
    static std::string _buf;
    _buf = self.getTitle();
    return _buf.c_str();
}
```

### 落地方案

1. **修改 `ast.rs`** 中 `SkippedDecl`：新增 `suggested_shim: Option<String>` 字段。
2. 在跳过 `std::string` 参数/返回的分支中，生成 shim 原型字符串并存入 `suggested_shim`。
3. **修改 `codegen.rs`**（接口报告 + `operator_shims.hpp` 渲染）：将 `suggested_shim` 追加到 `operator_shims.hpp` 中（或单独生成 `string_shims.hpp`）。

### 影响范围

- `src/ast.rs`（`SkippedDecl` + 类型跳过分支）
- `src/codegen.rs`（shim 文件渲染）

---

## §6  `--dry-run` 模式

### 现状

用户每次需要完整执行 `init`（含编译命令、AST dump）才能看到提取结果。
对于调试别名/类型配置，这成本较高。

### 改进目标

添加 `cpp2rust-demo init --dry-run` 选项：
- 执行真实构建命令、生成中间件和 AST JSON
- 仅输出"将要生成"的接口报告（到 stdout），**不写入** `rust/src/` 目录
- 适合快速验证 entry.cpp 配置

### 落地方案

1. **修改 `src/main.rs`**：`init` 子命令添加 `--dry-run: bool` flag。
2. 在 `init` 主流程末尾，若 `dry_run == true`，跳过 `codegen::generate_rust_project()` 调用，仅将接口报告打印到 stdout。
3. 影响最小，不改动 AST 解析或 codegen 逻辑。

### 影响范围

- `src/main.rs`（CLI + init 主流程）
- 对现有输出零影响

---

## §7  函数指针参数：自动生成接口包装建议

### 现状

含函数指针参数（如 `void (*callback)(int, void*)`）的函数/方法被跳过，标记 `hicc_limitation`。

### 改进目标

工具在接口报告中，为每个含函数指针参数的跳过项自动生成：
1. 对应的 C++ 纯虚接口类定义（建议模板）
2. 使用 `@make_proxy` 反向绑定的 Rust 端调用示例

这使用户无需从零开始设计接口。

### 落地方案

1. **修改 `ast.rs`**：识别函数指针参数类型（含 `(*)`），提取其签名并存入新字段。
2. **修改接口报告渲染**：生成接口类模板和 `@make_proxy` 使用示例片段（注释形式）。
3. 不修改实际提取逻辑，仅扩充报告信息。

### 影响范围

- `src/ast.rs`（skip 分支 + `SkippedDecl`）
- `src/codegen.rs`（报告渲染）

---

## 优先级建议

| 序号 | 功能 | 用户价值 | 实现复杂度 | 建议优先级 |
|------|------|---------|----------|-----------|
| §1 | 模板类别名建议与 suggest-aliases 命令 | ⭐⭐⭐ 高（解决最常见卡点）| 低 | P1 |
| §3 | 链式别名传递性解析 | ⭐⭐ 中 | 中 | P1 |
| §5 | `std::string` shim 建议生成 | ⭐⭐ 中 | 低 | P2 |
| §6 | `--dry-run` 模式 | ⭐⭐ 中 | 低 | P2 |
| §2 | 多重继承支持 | ⭐ 低（需 hicc 支持确认）| 中 | P3 |
| §4 | 虚继承检测与提示 | ⭐ 低 | 低 | P3 |
| §7 | 函数指针接口建议 | ⭐ 低 | 中 | P3 |
