# v6 开发进展记录 — Phase B 增强（收尾）：局部变量声明实例化追踪

> 本文档记录 v6 方案（见 `automated-cpp2rust-ffi-v6.md`）中 **Phase B 增强（收尾）**
> 阶段的开发目标、详细方案、详细进展与后续计划。全程使用简体中文。
>
> 前序进展：
> - `development-progress.md`（Phase A/B：模板类 / 模板函数泛型骨架）
> - `development-progress-phase-b-plus.md`（Phase B 增强：模板实例化别名）
> - `development-progress-phase-b-plus2.md`（Phase B 增强（续）：实例化追踪扩展 + 构造工厂骨架）
> - `development-progress-phase-b-plus3.md`（Phase B 增强（再续）：显式实例化追踪）
>
> 已合并的相关 PR：[#137](https://github.com/LuuuXXX/cpp2rust-demo/pull/137)、
> [#139](https://github.com/LuuuXXX/cpp2rust-demo/pull/139)、
> [#140](https://github.com/LuuuXXX/cpp2rust-demo/pull/140)、
> [#141](https://github.com/LuuuXXX/cpp2rust-demo/pull/141)、
> [#142](https://github.com/LuuuXXX/cpp2rust-demo/pull/142)。

---

## 1. 开发目标

在「模板类 → 泛型 `import_class!`」「模板函数 → 泛型 `import_lib!`」「模板实例化 →
类型别名 + 构造工厂骨架」「显式实例化 `template class Foo<T>;` 追踪」均已落地的基础上，
本阶段实现前序进展文档（`development-progress-phase-b-plus3.md`）「后续计划」中列为首项的
**Phase B 增强（收尾）**：

> 将实例化追踪来源进一步扩展到**局部变量声明**与 `new Foo<T>()` 等表达式级使用点。

具体目标：

1. **新增 AST 收集能力**：递归收集当前编译单元中函数 / 方法体内**局部变量声明**
   （`VarDecl`）的类型，覆盖 `Stack<int> s;`、`Stack<int>* p = new Stack<int>();`
   等表达式级实例化使用点（`auto p = new Stack<int>();` 会被 libclang 推导为
   `Stack<int> *`，同样可被捕获）。
2. **新增实例化追踪来源（来源 5）**：由提取器消费上述局部变量类型，识别其中
   `Name<具体类型>` 形式且 `Name` 属于本文件模板类的使用点，记录 `(模板名, 实参)`。
3. **硬约束（来自 v6 方案 §1.3 / §9）**：
   - `init` / `merge` 命令、参数、`.cpp2rust/<feature>/` 目录结构完全不变；
   - **默认产物逐字节不变** —— 本阶段能力沿用 Phase B 的 `CPP2RUST_GEN_TEMPLATES`
     环境变量开关，默认关闭。

---

## 2. 详细方案

### 2.1 沿用既有开关，保证默认产物不变

局部变量声明追踪产出的别名与构造工厂骨架**复用** `CPP2RUST_GEN_TEMPLATES` 开关
（`generator::hicc_codegen::templates_enabled`）：

- AST 始终收集局部变量类型（成本受限，见 §2.2），提取器始终构建实例化规格；
- 生成器仅在开关开启（`1` / `true` / `yes` / `on`，忽略大小写）时输出别名 / 工厂骨架；
- 关闭时不输出任何模板相关内容，因此所有既有 L1 黄金 / L2 / L3 / L4 / L5 基线不受影响。

### 2.2 AST 层：局部变量类型收集

文件：`src/ast_parser/mod.rs`（`parse_preprocessed` 第三遍）、
`src/ast_parser/collector.rs`（新增 `collect_local_var_types`）

- `CppAst` 新增字段 `local_var_types: Vec<String>`，保存当前编译单元函数 / 方法体内
  局部变量声明的类型显示名。
- 在既有两遍扫描之后增加**第三遍**：从翻译单元根实体递归遍历，
  `collect_local_var_types`：
  - **跳过位于系统头的子树**（`location().is_in_system_header()`），将遍历成本限制在
    用户代码范围内，避免遍历庞大的标准库实体树；
  - 仅记录落在当前编译单元字节范围（`cpp_ranges`，经 `entity_is_from_current_file`
    判定）内的 `VarDecl` 类型。
- 区分说明：函数参数为 `ParmDecl`、类字段为 `FieldDecl`，均**不是** `VarDecl`，因此不会在
  此重复收集；静态成员是 `VarDecl`，与既有「字段类型」来源（来源 1）可能重叠的部分由
  提取器按 `(模板名, 实参)` 去重消化。

> 该字段此前不存在，且仅在 `CPP2RUST_GEN_TEMPLATES` 开关开启时被生成器消费，故新增收集
> 不影响默认产物。

### 2.3 提取器：实例化追踪来源 5

文件：`src/extractor/template_spec.rs`（`build_template_instances`）

- 在既有来源 1–4（字段类型 / 方法参数 / 全局函数参数 / 显式实例化）之后新增
  **来源 5**：遍历 `ast.local_var_types`，复用既有的
  `collect_instance_from_type`（先 `strip_type_decorations` 剥离指针 / 引用 / cv 限定，
  再 `split_template_use` 拆分嵌套尖括号，最后 `record_instance` 去重）。
- 仍只为「本文件声明的模板类」生成别名，避免把 `std::vector` 等三方库模板误纳入。
- 收集到的别名同样复用既有 `build_template_factories` 派生构造工厂骨架。

### 2.4 测试

- 单元测试（`src/extractor/template_spec.rs`）：新增
  `collect_instance_from_local_var_type_records_alias`，验证
  - 直接类型 `Stack<int>` 与指针类型 `Stack<int> *`（来自 `new Stack<int>()` 推导）
    收集到同一别名 `StackI32` 并去重；
  - 非本文件模板（`std::vector<int>`）不被纳入。
- 集成测试（`tests/template_gen_tests.rs`）：在源码中新增函数
  `make_stacks()`，内含局部变量 `Stack<unsigned int> local(0u);` 与
  `Stack<unsigned int>* heap = new Stack<unsigned int>(1u);`，分别验证：
  - **默认关闭**时不输出 `pub type StackU32` 别名；
  - **开启开关**时输出 `pub type StackU32 = Stack<hicc::Pod<u32>>;`。

---

## 3. 详细进展

| 阶段 | 状态 | 说明 |
|------|------|------|
| AST：`CppAst.local_var_types` 字段 + 第三遍递归收集 | ✅ 已完成 | `collector::collect_local_var_types`，跳过系统头、限当前编译单元 |
| 提取器：实例化追踪来源 5（局部变量声明） | ✅ 已完成 | `build_template_instances` 消费 `local_var_types` |
| 别名 / 工厂复用 | ✅ 已完成 | 复用既有 `collect_instance_from_type` / `build_template_factories` |
| 单元测试 | ✅ 已完成 | 局部变量类型（含指针）收集 + 去重 + 三方库模板排除 |
| 集成测试 | ✅ 已完成 | `template_gen_tests` 验证默认关闭 / 开启两态（含局部变量追踪） |
| 回归验证（lib / L1 黄金 / 模板生成） | ✅ 已完成 | 254 lib 单测 + 52 L1 黄金 + 模板生成测试全绿，默认产物逐字节不变 |

**回归验证结论**：开关默认关闭时，L1 黄金（52）与 lib 单测（254，含本阶段新增 1 个）
全部通过，确认默认产物与改动前逐字节一致；开启开关后，函数 / 方法体内
`Stack<unsigned int>` 局部变量与 `new Stack<unsigned int>()` 也能正确生成实例化别名。

---

## 4. 后续计划

以下为 v6 方案中尚未落地的阶段，建议作为后续独立 PR 推进（与既有节奏一致：先做风险最低、
可独立验证的垂直切片）：

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

**风险提示**：局部变量声明追踪收集到的别名与工厂仍是「骨架」——对应的 C++ 符号需用户显式
实例化 / 包装后才存在；复杂或类类型实参仍需用户结合 hicc 类型补全。模板能力默认关闭的
设计为上述后续阶段提供了安全的灰度通道：可在开关开启下先验证生成质量，再决定是否纳入
黄金基线。
