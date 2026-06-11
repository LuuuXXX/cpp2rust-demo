# v6 开发进展记录 — Phase B 增强（再续）：显式实例化追踪

> 本文档记录 v6 方案（见 `automated-cpp2rust-ffi-v6.md`）中 **Phase B 增强（再续）**
> 阶段的开发目标、详细方案、详细进展与后续计划。全程使用简体中文。
>
> 前序进展：
> - `development-progress.md`（Phase A/B：模板类 / 模板函数泛型骨架）
> - `development-progress-phase-b-plus.md`（Phase B 增强：模板实例化别名）
> - `development-progress-phase-b-plus2.md`（Phase B 增强（续）：实例化追踪扩展 + 构造工厂骨架）
>
> 已合并的相关 PR：[#137](https://github.com/LuuuXXX/cpp2rust-demo/pull/137)、
> [#139](https://github.com/LuuuXXX/cpp2rust-demo/pull/139)、
> [#140](https://github.com/LuuuXXX/cpp2rust-demo/pull/140)、
> [#141](https://github.com/LuuuXXX/cpp2rust-demo/pull/141)。

---

## 1. 开发目标

在「模板类 → 泛型 `import_class!`」「模板函数 → 泛型 `import_lib!`」「模板实例化 →
类型别名 + 构造工厂骨架」已落地的基础上，本阶段实现前序进展文档「后续计划」中列为首项的
**Phase B 增强（再续）**：

> 把实例化追踪来源进一步扩展到**显式实例化** `template class Foo<int>;`
> （在 AST 中以带模板实参的 `ClassDecl` 形式出现）。

具体目标：

1. **修正模板实参提取**：原 `ClassInfo::template_args` 以 `format!("{:?}", arg)` 存储
   libclang 调试表示（如 `Type(Type { kind: Int, display_name: "int" })`），不可直接使用；
   改为提取干净的类型显示名（如 `int` / `double`）。
2. **新增实例化追踪来源**：扫描当前编译单元中**带模板实参且与本文件模板类同名**的
   `ClassDecl`，将其识别为显式实例化 `template class Foo<具体类型>;`，记录
   `(模板名, 具体类型实参)`。
3. **硬约束（来自 v6 方案 §1.3 / §9）**：
   - `init` / `merge` 命令、参数、`.cpp2rust/<feature>/` 目录结构完全不变；
   - **默认产物逐字节不变** —— 本阶段能力沿用 Phase B 的 `CPP2RUST_GEN_TEMPLATES`
     环境变量开关，默认关闭。

---

## 2. 详细方案

### 2.1 沿用既有开关，保证默认产物不变

显式实例化追踪产出的别名与构造工厂骨架**复用** `CPP2RUST_GEN_TEMPLATES` 开关
（`generator::hicc_codegen::templates_enabled`）：

- 提取器始终构建实例化规格（开销极小），便于测试与未来扩展；
- 生成器仅在开关开启（`1` / `true` / `yes` / `on`，忽略大小写）时输出别名 / 工厂骨架；
- 关闭时不输出任何模板相关内容，因此所有既有 L1 黄金 / L2 / L3 / L4 / L5 基线不受影响。

> 经实测确认：显式实例化 `template class Foo<int>;` 在 libclang 中表现为一个**带模板实参、
> 0 个子节点、与模板同名的 `ClassDecl`**。它本就会流经既有 `ClassDecl` 分支进入
> `ast.classes`，但因无成员方法 / 字段，对应的 `ClassSpec` 为空而被跳过，**不影响默认产物**。
> 本阶段只是在开关开启时额外消费其携带的模板实参。

### 2.2 模板实参提取修正

文件：`src/ast_parser/collector.rs`（`extract_class` / 新增 `template_arg_display`）

- 新增 `template_arg_display(arg)`：对 `TemplateArgument::Type(t)` 取
  `t.get_display_name()`（得到 `int` / `double` 等可直接复用的类型名）；非类型实参
  （整型常量等）暂回退到调试表示，供后续按需处理。
- `extract_class` 中 `template_args` 改用该函数填充，替换原 `format!("{:?}", arg)`。

该字段此前未被任何生成逻辑消费，故修正格式不影响默认产物（仅使其变得可用）。

### 2.3 显式实例化追踪来源

文件：`src/extractor/template_spec.rs`（`build_template_instances` /
新增 `record_instance`）

- 抽取统一的 `record_instance(name, args, ...)`：以**已拆分好的具体类型实参**记录实例化，
  并按 `(模板名, 实参列表)` 全局去重。既有「从类型字符串解析」的
  `collect_instance_from_type` 复用该函数（先 `strip_type_decorations` +
  `split_template_use` 拆分，再 `record_instance`）。
- 新增**来源 4**：遍历当前编译单元（`is_from_current_file`）中 `template_args` 非空、
  且名称属于本文件模板类集合的 `ClassInfo`，以其 `template_args` 直接调用
  `record_instance`，覆盖如 `template class Stack<long>;` 的显式实例化。

显式实例化收集到的别名同样复用既有的 `build_template_factories`，可派生对应的构造工厂
骨架（如 `Stack<long>` → `stack_i64_new`）。

### 2.4 测试

- 单元测试（`src/extractor/template_spec.rs`）：新增
  `record_instance_dedups_by_name_and_args`，验证显式实例化路径以已拆分实参直接记录、
  且同一 `(模板名, 实参)` 重复出现时去重（如同时来自显式实例化与字段类型）。
- 集成测试（`tests/template_gen_tests.rs`）：在模板类 `Stack<T>` 源码中新增
  `template class Stack<long>;`，分别验证：
  - **默认关闭**时不输出 `pub type StackI64` 等别名；
  - **开启开关**时输出 `pub type StackI64 = Stack<hicc::Pod<i64>>;` 与对应工厂骨架
    `pub unsafe fn stack_i64_new(initial: i64) -> StackI64;`。

---

## 3. 详细进展

| 阶段 | 状态 | 说明 |
|------|------|------|
| AST：模板实参提取修正（干净类型名） | ✅ 已完成 | `collector.rs` 新增 `template_arg_display` |
| 提取器：显式实例化追踪（来源 4） | ✅ 已完成 | `build_template_instances` + `record_instance` |
| 别名 / 工厂复用 | ✅ 已完成 | 复用既有 `build_instance_spec` / `build_template_factories` |
| 单元测试 | ✅ 已完成 | `record_instance` 去重 / 显式实参记录 |
| 集成测试 | ✅ 已完成 | `template_gen_tests` 验证默认关闭 / 开启两态（含显式实例化） |
| 回归验证（lib / L1 黄金） | ✅ 已完成 | 253 lib 单测 + 52 L1 黄金全绿，默认产物逐字节不变 |

**回归验证结论**：开关默认关闭时，L1 黄金（52）与 lib 单测（253，含本阶段新增 1 个）
全部通过，确认默认产物与改动前逐字节一致；开启开关后，显式实例化 `template class
Stack<long>;` 也能正确生成实例化别名与构造工厂骨架。

---

## 4. 后续计划

以下为 v6 方案中尚未落地的阶段，建议作为后续独立 PR 推进（与既有节奏一致：先做风险最低、
可独立验证的垂直切片）：

- **Phase B 增强（收尾）**：支持非默认 / 多参数构造函数的更精细签名映射；将实例化追踪
  扩展到局部变量声明与 `new Foo<T>()` 等表达式级使用点。
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

**风险提示**：显式实例化别名与工厂当前生成的仍是「骨架」——对应的 C++ 符号需用户显式
实例化 / 包装后才存在；复杂或类类型实参仍需用户结合 hicc 类型补全。模板能力默认关闭的
设计为上述后续阶段提供了安全的灰度通道：可在开关开启下先验证生成质量，再决定是否纳入
黄金基线。
