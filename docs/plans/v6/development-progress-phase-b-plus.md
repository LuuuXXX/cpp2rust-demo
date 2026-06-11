# v6 开发进展记录 — Phase B 增强（模板实例化别名）

> 本文档记录 v6 方案（见 `automated-cpp2rust-ffi-v6.md`）中 **Phase B 增强：模板实例化别名**
> 阶段的开发目标、详细方案、详细进展与后续计划。全程使用简体中文。
>
> 前序进展见 `development-progress.md`（Phase A/B：模板类 / 模板函数泛型骨架）。
> 已合并的相关 PR：[#137](https://github.com/LuuuXXX/cpp2rust-demo/pull/137)、
> [#139](https://github.com/LuuuXXX/cpp2rust-demo/pull/139)。

---

## 1. 开发目标

在 Phase A/B 已落地「模板类 → 泛型 `import_class!` 骨架」「模板函数 → 泛型 `import_lib!`
骨架」的基础上，本阶段实现 `development-progress.md` §4「后续计划」中列为**首要**的一项：

> **Phase B 增强（实例化别名）**：基于既有的「实例化类型追踪」，为被显式实例化的具体
> 类型生成类型别名（如 `pub type VecI32 = vector<hicc::Pod<i32>>;`），使模板骨架可直接
> 用于真实调用。

具体目标：

1. 从当前编译单元中识别「以具体类型实例化某个本文件声明的模板类」的使用点，收集
   `(模板名, 具体类型实参)`。
2. 为每个实例化生成 hicc 形式的**类型别名骨架**，POD 标量用 `hicc::Pod<...>` 包装，
   类类型实参保留原名并附 TODO 提示用户确认对应的 hicc 类型。
3. **硬约束（来自 v6 方案 §1.3 / §9）**：
   - `init` / `merge` 命令、参数、`.cpp2rust/<feature>/` 目录结构完全不变；
   - **默认产物逐字节不变** —— 别名生成沿用 Phase B 的 `CPP2RUST_GEN_TEMPLATES`
     环境变量开关，默认关闭。

---

## 2. 详细方案

### 2.1 沿用既有开关，保证默认产物不变

模板实例化别名的输出**复用** Phase B 的 `CPP2RUST_GEN_TEMPLATES` 开关
（`generator::hicc_codegen::templates_enabled`）：

- 提取器始终构建实例化别名规格（开销极小），便于测试与未来扩展；
- 生成器仅在开关开启（`1` / `true` / `yes` / `on`，忽略大小写）时输出别名骨架；
- 关闭时不输出任何别名内容，因此所有既有 L1 黄金 / L2 / L3 / L4 / L5 基线不受影响。

### 2.2 实例化追踪策略

文件：`src/extractor/template_spec.rs`（`build_template_instances`）

当前实现以「**本文件类的字段类型**」为实例化追踪来源：

- 收集本文件声明的模板类名集合（仅这些模板才生成别名，避免把 `std::vector` 等三方库
  模板误纳入）；
- 扫描所有 `is_from_current_file` 的类的字段类型，剥离指针 / 引用 / cv 限定后，
  匹配 `Name<args>` 形式且 `Name` 属于上述集合者，记录 `(模板名, 实参列表)`；
- 对相同的 `(模板名, 实参列表)` 去重。

该策略覆盖 v6 §3.2 中的典型写法：025 的包装类 `IntStack { Stack<int> impl; }`、
027 的 `IntMatrix { Matrix<int>* impl_; }` 等。

> 实参解析使用独立的 `split_template_use`，正确处理嵌套尖括号
> （如 `Map<int, vector<int>>`）。

### 2.3 实参类型 → hicc 形式映射

文件：`src/extractor/template_spec.rs`（`map_instance_arg`）

- **POD 标量**（经 `type_mapper::cpp_to_rust` 映射为 `i32` / `f64` / `bool` 等基础类型）
  → `hicc::Pod<i32>`，别名后缀取该 Rust 类型的 PascalCase（如 `I32`、`F64`），
  与 hicc `reference.md` 中 `type VecI32 = vector<hicc::Pod<i32>>;` 的命名风格一致；
- **类类型**（非 POD）→ 保留清理后的 C++ 类型名，别名后缀取其标识符片段
  （如 `std::string` → `StdString`），并标记 `needs_class_type`，由生成器附
  `cpp2rust-todo[TMPL]` 提示用户替换为对应的 hicc 类型（符合 v6 §8 降级策略）。

别名名称形如 `<模板名><各实参后缀拼接>`，例如 `Stack<int>` → `StackI32`、
`Stack<double>` → `StackF64`。

### 2.4 IR 与生成器

- IR（`src/ffi_model.rs`）：新增 `TemplateInstanceSpec`
  （`alias_name` / `template_name` / `hicc_args` / `needs_class_type`）；
  `FfiSpec` 新增 `template_instances` 字段。
- 生成器（`src/generator/hicc_codegen.rs`）：`emit_template_instances` 紧跟泛型模板类
  骨架之后输出别名（便于与泛型 `pub class Name<T>` 对照），形如：

  ```rust
  // cpp2rust-todo[TMPL]: 以下为模板实例化别名骨架，请确认实参类型与 AbiType 约束；
  // POD 标量已用 hicc::Pod 包装，类类型实参需替换为对应的 hicc 类（如 hicc_std::string）。
  pub type StackI32 = Stack<hicc::Pod<i32>>;
  pub type StackF64 = Stack<hicc::Pod<f64>>;
  ```

### 2.5 测试

- 单元测试（`src/extractor/template_spec.rs`）：覆盖 `strip_type_decorations`、
  `split_template_use`（含嵌套尖括号）、`map_instance_arg`（POD / 类类型）、
  `build_instance_spec`、`pascal_case_ident`。
- 集成测试（`tests/template_gen_tests.rs`）：在原 `Stack<T>` 模板基础上新增
  `IntStack { Stack<int> impl; }` 与 `DoubleStack { Stack<double> impl; }` 包装类，
  分别验证：
  - **默认关闭**时不输出 `pub type StackI32` 等别名；
  - **开启开关**时输出 `pub type StackI32 = Stack<hicc::Pod<i32>>;` 与
    `pub type StackF64 = Stack<hicc::Pod<f64>>;`。

---

## 3. 详细进展

| 阶段 | 状态 | 说明 |
|------|------|------|
| IR：`FfiSpec.template_instances` | ✅ 已完成 | 新增 `TemplateInstanceSpec` |
| 提取器：实例化追踪 + 别名构建 | ✅ 已完成 | `build_template_instances` 由字段类型收集 |
| 生成器：别名骨架输出 | ✅ 已完成 | `emit_template_instances`，受 `CPP2RUST_GEN_TEMPLATES` 控制 |
| 单元测试 | ✅ 已完成 | 7 个新单测（标量 / 类类型 / 嵌套尖括号 / 别名命名） |
| 集成测试 | ✅ 已完成 | `template_gen_tests` 验证默认关闭 / 开启两态 |
| 回归验证（lib / L1 黄金） | ✅ 已完成 | 250 lib 单测 + 52 L1 黄金全绿，默认产物逐字节不变 |

**回归验证结论**：开关默认关闭时，L1 黄金（52）与 lib 单测（250，含本阶段新增 7 个）
全部通过，确认默认产物与改动前逐字节一致；开启开关后泛型骨架与实例化别名按预期输出。

---

## 4. 后续计划

以下为 v6 方案中尚未落地的阶段，建议作为后续独立 PR 推进（与既有节奏一致：先做风险最低、
可独立验证的垂直切片）：

- **Phase B 增强（续）**：实例化别名的**配套工厂函数**（构造函数在 `import_lib!` 中按实例化
  类型声明），以及把追踪来源从「字段类型」扩展到显式实例化 `template class Foo<int>;` 与
  方法参数 / 返回类型中的实例化使用点。
- **Phase C（高级映射）**：抽象类 → `#[interface]` Trait（**纯虚接口已在
  `class_spec.rs` 落地**）的基础上补充可选 `@make_proxy` 工厂；RTTI 场景 →
  `@dynamic_cast`；私有析构 → `destroy = "..."` 属性增强。
- **Phase E（examples 改造）**：将 024/025/026/027 + 虚函数 / STL 选定示例从「手写包装
  降级」升级为「原生 hicc 模板映射」，并为每个改造示例补充 `tests/smoke.rs`；
  同步更新各示例 README。涉及 `main.rs` → `lib.rs` + `main.rs` 的结构调整与 L1 黄金
  提取目标变更，需分批、逐示例验证。
- **Phase F（测试 / CI）**：L1 黄金适配模板骨架 / 别名片段；新增 `smoke` job 与
  `gen-verify` 端到端 job（`init` → 生成目录 `cargo test`）。
- **Phase G（剩余文档）**：`docs/INTRODUCTION.md`、`docs/references/hicc.md` 补充模板类 /
  模板函数 / 实例化别名 / 接口映射章节，与 `reference.md` 对齐。

**风险提示**：实例化别名当前仅生成「类型别名骨架」，尚未生成可直接调用的构造工厂；
复杂或类类型实参需用户结合 hicc 类型补全。模板能力默认关闭的设计为上述后续阶段提供了
安全的灰度通道：可在开关开启下先验证生成质量，再决定是否纳入黄金基线。
