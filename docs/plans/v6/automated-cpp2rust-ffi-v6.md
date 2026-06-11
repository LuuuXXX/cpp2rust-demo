# C++ 到 Rust Safe FFI 自动化工具 - 方案 v6

> **定位**：v6 是 v5 的**增强**而非重写。v5 已经完整落地（49/49 L1 通过、L2–L5 全绿、init + merge 工作流稳定）。
> v6 在**完全保留 init + merge 现有使用方法**的前提下，补齐 v5 主动丢弃的信息，**充分发挥 hicc 的表达能力**。

---

## 1. 背景与动机

### 1.1 v5 的核心取舍

v5 的指导理念是「**C++ 模板的价值在于实例化结果，而非模板本身**」，因此在 AST 提取阶段主动**丢弃了大量声明信息**：

| 被丢弃的信息 | v5 当前行为 | 后果 |
|------------|-----------|------|
| `ClassTemplate` / `FunctionTemplate` 模板**声明** | 完全忽略，只看 `ClassTemplateSpecialization` | 用户在生成代码里看不到 `Stack<T>`，只看到扁平化的 `IntStack` / `DoubleStack`，无法对标原 C++ |
| 模板类的**泛型方法签名** | 不输出，仅对具体实例化生成方法 | 丢失「这是一个泛型方法」的语义 |
| 模板函数的**泛型形态** | 只输出 `swap_int` / `swap_double` 等命名 shim | 用户无从得知这些 shim 来自同一个 `do_swap<T>` |
| `TypedefDecl` / `TypeAliasDecl` | 仅在参数类型间接处理，不单独遍历 | 类型别名信息丢失 |
| 实例化与模板的**对应关系** | 不记录 | 报告里看不到「IntStack ← Stack<int>」的映射 |

### 1.2 用户的实际诉求（问题陈述）

> 「实际使用过程中有大量信息会被跳过，比如模板方法、模板类的声明等等。这些在 hicc 里面应该很好处理，且这些声明也有利于用户对标原 C++ 代码，同时理解 Rust 代码。」

核心矛盾：**v5 为了"能编译"牺牲了"可读性 / 可对标性"**。但 hicc 本身**完全支持泛型类声明**，v5 没有用到这层能力。

### 1.3 hicc 已具备但 v5 未利用的能力（关键依据）

参考 `references/hicc/examples/import_lib_class/src/main.rs`，hicc 原生支持**保留模板声明**：

```rust
hicc::cpp! {
    template<typename T>
    class Generic { /* ... */ };

    template class Generic<int>;        // 显式实例化
    template class Generic<double>;
    Generic<int>*  hicc_new_generic_int()  { return new Generic<int>; }
    Generic<double>* hicc_new_generic_double() { return new Generic<double>; }
}

hicc::import_lib! {
    class Generic<T>;                    // ★ 泛型类声明（v5 没有生成）

    #[cpp(class = "Generic")]
    class Generic<T> {                   // ★ 泛型类 + 泛型方法（v5 没有生成）
        #[cpp(method = "void display() const")]
        fn display(&self);

        #[cpp(func = "Generic<int>* hicc_new_generic_int()")]
        fn new() -> Generic<hicc::Pod<i32>>;          // 具体实例化工厂
        #[cpp(func = "Generic<double>* hicc_new_generic_double()")]
        fn new_double() -> Generic<hicc::Pod<f64>>;
    }
}
```

对照 `references/hicc/hicc-std/src/std_test/*` 的 `pub class VecInt = vector<hicc::Pod<i32>>;` 语法，hicc 还支持**类型别名形式的实例化声明**。

> **结论**：v5 把 `Stack<int>` → `IntStack`「拍平」是一种降级；hicc 完全能表达 `class Stack<T>` + 具体实例化。v6 应改为**保留泛型形态 + 同时给出具体实例化**，让生成代码与原 C++ 一一对应。

### 1.4 hicc 的验证方式（冒烟测试依据）

参考 `references/hicc/hicc-std/src/std_vector.rs`：hicc 自身通过**文档测试（doctest）**验证 FFI——在 `///` 注释里写 `assert_eq!(vec.size(), 0);` 等可执行断言，`cargo test` 时作为 doctest 编译运行。这正是「参考 hicc 里面是怎么验证的」要采纳的模式。

---

## 2. v6 核心目标

1. **保留声明信息**：模板类声明（`class Foo<T>;`）、模板方法签名、模板函数泛型形态、`typedef`/`using` 别名，全部在生成代码中以 hicc 原生语法体现。
2. **可对标 C++**：每个生成条目都能追溯到原始 C++ 声明（注释 + 报告中的映射表）。
3. **可验证**：为每个特性提供 / 生成**冒烟测试**，真正运行 Rust FFI 校验行为正确，而不止于「能编译」。
4. **零破坏**：`init` 与 `merge` 的命令行接口、参数、输出目录结构**完全不变**。所有增强都是「在已有产物中补充更多内容」，而非「改变调用方式」。

---

## 3. 总体设计原则

| 原则 | 说明 |
|------|------|
| **增量叠加** | 新增 `import_class!` / `import_lib!` 条目和泛型声明，不删除 v5 已有的扁平化实例化产物（保证 L1/L2/L3 现有黄金文件不被破坏，或同步更新黄金文件）。 |
| **降级可回退** | 若泛型形态无法安全生成（如复杂偏特化），回退到 v5 的扁平化实例化策略，并打 `[TPL]` TODO 标记。 |
| **声明与实现分离** | 模板**声明**进 `import_lib! { class Foo<T>; }`；具体**实例化工厂**进 `import_lib!` 的具体类型函数；**实现**进 `hicc::cpp!`。 |
| **接口稳定** | `init` / `merge` 的 CLI、环境变量、`.cpp2rust/<feature>/` 目录结构保持 v5 不变。 |

---

## 4. 优化方案 A：模板 / 声明信息的保留（核心能力）

### 4.1 AST 提取层增强（`src/ast_parser/collector.rs`）

- 新增对 `EntityKind::ClassTemplate` 的收集：提取模板参数列表（`T`、`N`、`typename...`）、泛型方法签名、泛型字段。
- 新增对 `EntityKind::FunctionTemplate` 的收集：记录模板函数的泛型签名，作为其各命名 shim（`swap_int` 等）的「来源声明」。
- 保留并扩展现有 `ClassTemplateSpecialization` / `ClassTemplatePartialSpecialization` 收集：建立 **模板声明 ↔ 实例化** 的映射关系（如 `Stack<T>` → {`Stack<int>`, `Stack<double>`}）。
- 新增对 `TypedefDecl` / `TypeAliasDecl` 的独立收集（当前仅间接处理），用于生成 hicc 的 `class Alias = ...;` 形态。
- 数据结构层面：在 `CppAst` / `ClassInfo` 中新增「模板参数」「实例化来源」「别名目标」等字段（向后兼容，默认空）。

### 4.2 代码生成层增强（`src/generator/hicc_codegen.rs`、`project_generator.rs`）

按 hicc 原生语法生成三类新内容：

1. **泛型类声明**：在 `import_lib!` 块顶部生成 `class Stack<T>;`（对标原 C++ `template<class T> class Stack`）。
2. **泛型类方法绑定**：生成 `#[cpp(class = "Stack")] class Stack<T> { ... }`，把泛型方法签名以 hicc 语法列出；具体实例化通过 `fn new() -> Stack<hicc::Pod<i32>>` 等工厂函数表达。
3. **实例化映射注释**：在每个具体实例化条目上方加注释，例如 `// 来自模板 Stack<T> 的实例化：Stack<int>`，便于用户对标。

> **兼容策略**：保留 v5 已有的扁平命名实例化（`IntStack`）作为可选/过渡产物，或在 v6 中以泛型形态替代——**该取舍在实现阶段按示例逐个评估，并同步更新对应黄金文件**。

### 4.3 报告增强（`init-interface-report.md`）

新增「**模板与实例化映射表**」与「**被保留的声明清单**」两个章节，列出：模板声明 → 实例化列表、模板方法 → 绑定方法、typedef → 别名映射。让用户一眼看清「哪些声明被保留、对应原 C++ 的哪一行」。

---

## 5. 优化方案 B：examples 优化 + 冒烟测试（Rust FFI 验证）

### 5.1 examples 内容优化

- **重点强化模板相关示例**（`024_template_function`、`025_template_class`、`026_template_specialization`、`027_template_instantiation`、`028_variadic_template`）：在 `rust_hicc/src/main.rs` 中体现「泛型声明 + 具体实例化」两层结构，并在注释中标注对应原 C++ 声明。
- **统一示例结构**：每个 `examples/NNN_*/` 维持现有 `cpp/` + `rust_hicc/`，README 增补「原 C++ 声明 ↔ 生成的 hicc 声明」对照小节。

### 5.2 为每个示例补充冒烟测试（参考 hicc 的 doctest 模式）

在每个 `examples/NNN_*/rust_hicc/` 中新增**冒烟测试**，验证转换后的 Rust FFI 行为正确（不止编译，而是真正运行断言）。两种可选载体：

- **集成测试**：`examples/NNN_*/rust_hicc/tests/smoke_test.rs`，用 `#[test]` + `assert_eq!` 调用生成的 FFI（如 `let mut s = intstack_new(); s.push(10); assert_eq!(s.top(), 10);`）。
- **doctest**：仿照 `hicc-std` 在 `///` 注释中嵌入可运行断言。

> **设计要求**：冒烟测试断言的是 **FFI 行为**（调用结果），覆盖每个特性最小可观察行为；与 L3 的「stdout 比对」互补——冒烟测试关注 API 语义，L3 关注端到端输出。

### 5.3 冒烟测试与库构建

- 冒烟测试复用示例已有的 `build.rs`（hicc-build 编译 `hicc::cpp!`）。
- 在 `scripts/build_cpp_libs.sh` / `.ps1` 中确保冒烟测试所需共享库被构建（与现有 L3 流程一致，避免重复编译开销）。

---

## 6. 优化方案 C：支持「生成」冒烟测试（工具能力）

> 注意 repository 记忆：早期 `smoke_test_gen` 曾被引入又移除（commit 59851fab）。v6 **重新引入但定位明确**：作为**可选产物**，默认行为与 v5 一致，**不改变 init + merge 接口**。

### 6.1 触发方式（不破坏现有接口）

- **默认关闭**：`init` / `merge` 不带新参数时，行为与 v5 **完全一致**，不生成冒烟测试。
- **显式开启**：通过**新增可选 flag**（如 `--with-smoke-test`）或**环境变量**（如 `CPP2RUST_SMOKE_TEST=1`）开启。新增 flag 是纯增量，不影响既有调用。

### 6.2 生成内容

- 生成位置：`.cpp2rust/<feature>/rust/tests/smoke_test.rs`（与生成的 Rust 项目同构，不污染 `src/`）。
- 生成策略：基于已提取的 FFI 模型（`FfiSpec`），为每个导出函数 / 类方法生成**最小可运行断言骨架**：
  - 工厂函数 → 创建对象；
  - 有返回值的方法 → `assert!` 返回值类型合理 / 非 panic；
  - 纯过程函数 → 调用不 panic。
- 对于无法自动推断期望值的方法，生成 `// cpp2rust-todo[SMOKE]: 补充断言` 占位，让用户补全。
- 参考 hicc 的验证风格：优先生成 doctest 友好或 `#[test]` 友好的断言。

### 6.3 生成器实现

- 在 `src/generator/` 下新增（或恢复）`smoke_test_gen.rs`，输入为 `FfiSpec`，输出 `smoke_test.rs` 文本。
- 在 `project_generator.rs` 中新增 `write_smoke_test`（仅在开启时调用）。
- 单元测试覆盖生成器：给定一组 `FfiSpec` 条目，断言生成文本包含预期调用。

---

## 7. 优化方案 D：文档优化

### 7.1 `docs/INTRODUCTION.md`

- 「会导出 vs 不会导出」表格更新：把「模板声明本身 → 不会导出」改为「**模板声明 → 以 hicc 泛型语法保留**」，并说明实例化映射。
- 新增「模板与声明保留」章节，配 `Stack<T>` ↔ `IntStack` 的对照示例（引用 hicc `import_lib_class` 语法）。
- 「测试体系」表格新增冒烟测试层（见 §8）。

### 7.2 `README.md` / `DEVELOPMENT.md`

- README：在特性表中标注哪些示例现在保留泛型声明；新增 `--with-smoke-test`（或环境变量）的可选用法说明，并明确「默认行为不变」。
- DEVELOPMENT：补充冒烟测试的本地运行方式、生成器的开发与测试方法。

### 7.3 examples 各 README

- 每个模板相关示例的 README 增补「原 C++ 模板声明 ↔ 生成的 hicc 声明 ↔ 具体实例化」三栏对照表，呼应用户「对标原 C++ 代码」的诉求。

### 7.4 hicc 能力对标文档

- 在 `docs/references/hicc.md` 中补充 `import_lib_class` 泛型类用法的引用与说明（当前该文档已覆盖 class-in-lib，但未突出泛型类声明的对标价值）。

---

## 8. 优化方案 E：测试与 CI 优化

### 8.1 测试分层扩展

在现有 L1–L5 基础上，新增冒烟测试层（建议命名 **L6**，避免与现有层冲突）：

| 层 | 验证什么 | 方法 | 触发 |
|----|---------|------|------|
| L1 | 生成代码 vs 黄金文件 | 文本逐段比对 | 每次提交 |
| L2 | 生成项目可编译 | `cargo build` | 每次提交 |
| L3 | 运行输出正确 | `cargo run` stdout 比对 | 合并前 |
| L4 | 真实开源项目 E2E | init + merge | 合并前 |
| L5 | 符号链接完整 | `nm` 双向比对 | 合并前 |
| **L6（新增）** | **FFI 行为正确** | **运行 examples 冒烟测试（`cargo test`/doctest）** | **每次提交** |

- 同步更新受模板声明保留影响的 **L1 黄金文件**（`rust_hicc/src/main.rs` 中的 hicc 三段），保证 L1 仍 100% 通过。
- 新增针对 `collector.rs` 模板收集、`smoke_test_gen.rs` 生成器的**单元测试**（`cargo test --lib`）。

### 8.2 CI（`.github/workflows/ci.yml`）

- 新增 **L6 job**：在所有 examples 上运行冒烟测试（`cargo test` / `--doc`），复用现有「预构建 C++ 共享库」步骤以控制时长。
- L6 至少在 Linux 上运行；视耗时决定是否纳入 Windows/macOS 矩阵（参考现有 L2/L3 的多平台策略）。
- 保持 CI 既有合并门禁不变：`cargo fmt --check`、`cargo clippy -- -D warnings`、`cargo test`（参考 `DEVELOPMENT.md`）。
- 若引入 `--with-smoke-test`，增加一个轻量 job 验证「开启时生成的 `tests/smoke_test.rs` 能编译」。

### 8.3 验证脚本

- `usage/verify-rapidjson-ffi.sh`：在不破坏现有 § 0–§ 7 流程的前提下，可选地演示 `--with-smoke-test` 产物（按记忆，§ 5c 冒烟断言此前已删除——v6 以新的独立小节重新引入，且明确为可选）。

---

## 9. 「不改变现有使用方法」的保证（init + merge）

| 关注点 | v6 保证 |
|--------|--------|
| `init` 命令签名 | 不变（`cpp2rust-demo init [--feature X] -- <BUILD_CMD>`）；冒烟测试生成仅在新增可选 flag/env 下触发 |
| `merge` 命令签名 | 不变 |
| `.cpp2rust/<feature>/` 目录结构 | 不变；新增内容（泛型声明、可选 smoke test）落在既有目录内，不改变现有文件含义 |
| 默认产物 | 默认不生成冒烟测试；模板声明保留是对 `lib.rs` 内容的**增强**，不改变文件名/位置 |
| 环境变量 | 现有 `CPP2RUST_*` 不变，新增变量为可选 |

---

## 10. 实现计划（Phase 顺序）

> 沿用 v5 的「测试驱动」原则：先扩测试基线，再加功能，每个 Phase 以「相关测试通过」为完成标准。

| 阶段 | 内容 | 优先级 | 依赖 |
|------|------|--------|------|
| **Phase A** | AST 提取层：收集模板声明 / 模板方法 / typedef，建立模板↔实例化映射 | P0 | — |
| **Phase B** | 代码生成层：按 hicc 泛型语法生成 `class Foo<T>;` + 泛型方法 + 实例化工厂 + 映射注释 | P0 | A |
| **Phase C** | 更新模板相关 examples 的黄金文件，保证 L1 通过 | P0 | B |
| **Phase D** | 为 examples 补充冒烟测试（L6），接入 CI | P0 | C |
| **Phase E** | 冒烟测试生成器（`smoke_test_gen.rs`）+ 可选 flag/env，含单元测试 | P1 | A |
| **Phase F** | 文档优化（INTRODUCTION / README / DEVELOPMENT / examples README / hicc.md） | P1 | B–E |
| **Phase G** | CI 优化（新增 L6 job、可选 smoke 编译 job）+ 报告增强 | P1 | D, E |

---

## 11. 风险评估

| 风险 | 影响 | 缓解措施 |
|------|------|---------|
| 泛型形态生成失败（复杂偏特化 / 依赖型参数） | 部分模板无法保留声明 | 回退 v5 扁平化策略 + `[TPL]` TODO 标记 |
| 黄金文件大范围变更 | L1 维护成本上升 | 按示例逐个更新；保留扁平化产物作为过渡，减少一次性改动 |
| 冒烟测试无法自动推断期望值 | 生成的断言过弱 | 仅生成「不 panic / 类型合理」骨架 + `[SMOKE]` TODO，由用户补全；examples 内冒烟测试为手写强断言 |
| L6 增加 CI 时长 | 流水线变慢 | 复用预构建共享库、限定平台矩阵、与 L3 共享构建产物 |
| 新增 flag 被误认为改变接口 | 违反「不改变使用方法」 | 默认关闭，行为与 v5 完全一致；文档明确「纯增量、可选」 |
| `smoke_test_gen` 曾被移除 | 重复历史问题 | 明确新定位（可选产物 + 独立目录 + 单元测试覆盖），并在文档/CI 中说明与历史移除的区别 |

---

## 12. 验收标准

1. 模板相关 examples 的生成代码中**出现泛型类声明**（`class Foo<T>;`）与泛型方法绑定，且注释标明对应原 C++ 声明。
2. `init-interface-report.md` 含「模板 ↔ 实例化映射」章节。
3. 每个 example 具备可运行的冒烟测试，`cargo test` 全绿（新增 L6）。
4. 工具支持在**显式开启**时生成 `tests/smoke_test.rs`；默认行为与 v5 字节级一致。
5. `init` / `merge` 的 CLI、目录结构、默认产物**无破坏性变更**。
6. L1–L5 全部保持通过；`cargo fmt --check`、`cargo clippy -- -D warnings`、`cargo test` 全绿。
7. 文档（INTRODUCTION / README / DEVELOPMENT / examples README）同步更新，体现声明保留与冒烟测试。
