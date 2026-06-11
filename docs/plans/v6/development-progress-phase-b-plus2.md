# v6 开发进展记录 — Phase B 增强（续）：实例化追踪扩展 + 构造工厂骨架

> 本文档记录 v6 方案（见 `automated-cpp2rust-ffi-v6.md`）中 **Phase B 增强（续）**
> 阶段的开发目标、详细方案、详细进展与后续计划。全程使用简体中文。
>
> 前序进展：
> - `development-progress.md`（Phase A/B：模板类 / 模板函数泛型骨架）
> - `development-progress-phase-b-plus.md`（Phase B 增强：模板实例化别名）
>
> 已合并的相关 PR：[#137](https://github.com/LuuuXXX/cpp2rust-demo/pull/137)、
> [#139](https://github.com/LuuuXXX/cpp2rust-demo/pull/139)、
> [#140](https://github.com/LuuuXXX/cpp2rust-demo/pull/140)。

---

## 1. 开发目标

在「模板类 → 泛型 `import_class!`」「模板函数 → 泛型 `import_lib!`」「模板实例化 →
类型别名」已落地的基础上，本阶段实现前序进展文档「后续计划」中列为首项的
**Phase B 增强（续）**：

> **实例化别名的配套工厂函数**（构造函数在 `import_lib!` 中按实例化类型声明），以及把
> 追踪来源从「字段类型」扩展到**方法参数 / 返回类型**中的实例化使用点。

具体目标：

1. **扩展实例化追踪来源**：除包装类字段类型外，额外扫描当前编译单元中类的方法参数 /
   返回类型、以及全局函数的参数 / 返回类型，识别 `Name<具体类型>` 形式的实例化使用点。
2. **生成构造工厂骨架**：由模板类的公有构造函数派生工厂函数，将类型参数 `T` 替换为
   实例化的具体类型，在 `import_lib!` 中输出 hicc 工厂骨架，使实例化别名可向真实构造
   调用靠拢。
3. **硬约束（来自 v6 方案 §1.3 / §9）**：
   - `init` / `merge` 命令、参数、`.cpp2rust/<feature>/` 目录结构完全不变；
   - **默认产物逐字节不变** —— 本阶段能力沿用 Phase B 的 `CPP2RUST_GEN_TEMPLATES`
     环境变量开关，默认关闭。

---

## 2. 详细方案

### 2.1 沿用既有开关，保证默认产物不变

实例化追踪扩展与工厂骨架的输出**复用** `CPP2RUST_GEN_TEMPLATES` 开关
（`generator::hicc_codegen::templates_enabled`）：

- 提取器始终构建别名与工厂规格（开销极小），便于测试与未来扩展；
- 生成器仅在开关开启（`1` / `true` / `yes` / `on`，忽略大小写）时输出工厂骨架；
- 关闭时不输出任何模板相关内容，因此所有既有 L1 黄金 / L2 / L3 / L4 / L5 基线不受影响。

### 2.2 实例化追踪来源扩展

文件：`src/extractor/template_spec.rs`（`build_template_instances` /
`collect_instance_from_type`）

将原先仅扫描「类字段类型」的策略，抽取为统一的
`collect_instance_from_type(type_name, ...)` 辅助函数，并应用到以下来源：

1. **字段类型**（既有）：如 `Stack<int> impl;`；
2. **方法参数 / 返回类型**（新增）：如 `void use(Stack<short>& s)`、`Stack<int> make()`；
3. **全局函数参数 / 返回类型**（新增）：同上，仅限当前编译单元（`is_from_current_file`）。

辅助函数复用既有的 `strip_type_decorations`（剥离指针 / 引用 / cv 限定）与
`split_template_use`（正确处理嵌套尖括号），并对 `(模板名, 实参列表)` 全局去重。仍只为
「本文件声明的模板类」生成别名，避免把 `std::vector` 等三方库模板误纳入。

### 2.3 构造工厂骨架派生

文件：`src/extractor/template_spec.rs`（`build_template_factories` /
`substitute_type_params`）

对每个实例化别名规格：

- 在 `ast.template_classes` 中按名称匹配本文件模板类，要求其类型参数个数与实例化实参
  个数一致（否则跳过，无法可靠替换）；
- 收集模板类的**公有构造函数**，对每个构造函数派生一个工厂规格：
  - **Rust 工厂名**：`<别名 snake_case>_new`（如 `StackI32` → `stack_i32_new`）；
    模板类含多个构造函数时追加序号 `_0` / `_1` 以避免重名；
  - **C++ 工厂签名**：`<模板名><<具体实参>>* <工厂名>(<替换后的参数类型>)`，
    如 `Stack<int>* stack_i32_new(int initial)`；
  - **Rust 参数 / 返回类型**：参数类型经 `T` 替换后映射为 Rust 类型；返回实例化别名
    （如 `StackI32`）。

**类型参数替换**（`substitute_type_params` / `replace_ident`）：以**完整标识符**为单位
替换（前后不能是字母 / 数字 / 下划线），因此 `T` 不会误伤 `Time`、`TT`、`vector_T`
等子串。

### 2.4 IR 与生成器

- IR（`src/ffi_model.rs`）：
  - `TemplateInstanceSpec` 新增 `cpp_args` 字段（原始具体 C++ 实参，用于派生工厂签名）；
  - 新增 `TemplateFactorySpec`（`rust_name` / `alias_name` / `cpp_sig` / `params`），
    `FfiSpec` 新增 `template_factories` 字段。
- 生成器（`src/generator/hicc_codegen.rs`）：新增 `emit_template_factory`，在 `import_lib!`
  块内紧随模板函数骨架之后输出工厂；并把工厂纳入「`import_lib!` 是否需要生成」的判定，
  避免仅有工厂时整块被跳过。所有输出仍由 `templates_enabled()` 开关裁决。

工厂对应的 C++ 符号通常需用户在 C++ 侧显式实例化 / 包装后才存在，故每个工厂均附
`cpp2rust-todo[TMPL]` 提示用户提供符号并校验签名（符合 v6 方案 §8 的降级策略）。

### 2.5 测试

- 单元测试（`src/extractor/template_spec.rs`）：新增
  - `replace_ident_only_matches_whole_identifiers`：验证完整标识符替换、子串不误伤；
  - `substitute_type_params_replaces_each_param`：验证多类型参数替换；
  - `build_instance_spec_produces_alias` 扩展断言 `cpp_args`。
- 集成测试（`tests/template_gen_tests.rs`）：在模板类 `Stack<T>` 上新增构造函数
  `Stack(T initial)` 与方法参数实例化使用点 `StackUser::use_short(Stack<short>& s)`，
  分别验证：
  - **默认关闭**时不输出工厂骨架（`stack_i32_new`）；
  - **开启开关**时：
    - 从方法参数收集到 `pub type StackI16 = Stack<hicc::Pod<i16>>;`；
    - 输出 `#[cpp(func = "Stack<int>* stack_i32_new(int initial)")]` 与
      `pub unsafe fn stack_i32_new(initial: i32) -> StackI32;` 等工厂骨架。

---

## 3. 详细进展

| 阶段 | 状态 | 说明 |
|------|------|------|
| IR：`TemplateInstanceSpec.cpp_args` / `TemplateFactorySpec` / `FfiSpec.template_factories` | ✅ 已完成 | 新增工厂规格 IR |
| 提取器：实例化追踪扩展（字段 → 方法 / 全局函数参数 / 返回类型） | ✅ 已完成 | `collect_instance_from_type` 统一收集 |
| 提取器：构造工厂派生（`T` 替换为具体类型） | ✅ 已完成 | `build_template_factories` + `substitute_type_params` |
| 生成器：工厂骨架输出 | ✅ 已完成 | `emit_template_factory`，受 `CPP2RUST_GEN_TEMPLATES` 控制 |
| 单元测试 | ✅ 已完成 | 标识符替换 / 多参数替换 / cpp_args 断言 |
| 集成测试 | ✅ 已完成 | `template_gen_tests` 验证默认关闭 / 开启两态（含工厂与方法参数追踪） |
| 回归验证（lib / L1 黄金） | ✅ 已完成 | 252 lib 单测 + 52 L1 黄金全绿，默认产物逐字节不变 |
| 文档（INTRODUCTION / hicc.md / DEVELOPMENT / README / 本进展文档） | ✅ 已完成 | Phase G 部分文档对齐 |

**回归验证结论**：开关默认关闭时，L1 黄金（52）与 lib 单测（252，含本阶段新增 2 个）
全部通过，确认默认产物与改动前逐字节一致；开启开关后泛型骨架、实例化别名与构造工厂
骨架按预期输出。

---

## 4. 后续计划

以下为 v6 方案中尚未落地的阶段，建议作为后续独立 PR 推进（与既有节奏一致：先做风险最低、
可独立验证的垂直切片）：

- **Phase B 增强（再续）**：把实例化追踪来源进一步扩展到显式实例化
  `template class Foo<int>;`（AST 中以 `ClassTemplateSpecialization` 形式出现），并支持
  非默认 / 多参数构造函数的更精细签名映射。
- **Phase C（高级映射）**：在纯虚接口已映射为 `#[interface]`（`class_spec.rs`）的基础上，
  补充可选 `@make_proxy` 工厂（让 Rust 侧实现 C++ 抽象类）；RTTI 场景 → `@dynamic_cast`；
  私有析构 → `destroy = "..."` 属性增强。
- **Phase E（examples 改造）**：将 024/025/026/027 + 虚函数 / STL 选定示例从「手写包装
  降级」升级为「原生 hicc 模板映射」，并为每个改造示例补充 `tests/smoke.rs`；同步更新各
  示例 README。涉及 `main.rs` → `lib.rs` + `main.rs` 的结构调整与 L1 黄金提取目标变更，
  需分批、逐示例验证，风险较高。
- **Phase F（测试 / CI）**：L1 黄金适配模板骨架 / 别名 / 工厂片段；新增 `smoke` job 与
  `gen-verify` 端到端 job（`init` → 生成目录 `cargo test`）。
- **Phase G（剩余文档）**：随 examples 改造同步更新各示例 README 的「模板映射 + 冒烟测试」
  说明。

**风险提示**：构造工厂当前生成的是「骨架」——对应的 C++ 符号需用户显式实例化 / 包装后
才存在；复杂或类类型实参仍需用户结合 hicc 类型补全。模板能力默认关闭的设计为上述后续
阶段提供了安全的灰度通道：可在开关开启下先验证生成质量，再决定是否纳入黄金基线。
