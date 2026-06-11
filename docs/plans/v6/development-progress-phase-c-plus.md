# v6 开发进展记录 — Phase C（续）：`@dynamic_cast` 下行转换绑定骨架

> 本文档记录 v6 方案（见 `automated-cpp2rust-ffi-v6.md`）中 **Phase C（续，高级映射）**
> 阶段的开发目标、详细方案、详细进展与后续计划。全程使用简体中文。
>
> 前序进展：
> - `development-progress.md`（Phase A/B：模板类 / 模板函数泛型骨架）
> - `development-progress-phase-b-plus.md`（Phase B 增强：模板实例化别名）
> - `development-progress-phase-b-plus2.md`（Phase B 增强（续）：实例化追踪扩展 + 构造工厂骨架）
> - `development-progress-phase-b-plus3.md`（Phase B 增强（再续）：显式实例化追踪）
> - `development-progress-phase-b-plus4.md`（Phase B 增强（收尾）：局部变量声明实例化追踪）
> - `development-progress-phase-c.md`（Phase C：`@make_proxy` 代理工厂骨架）
>
> 已合并的相关 PR：[#137](https://github.com/LuuuXXX/cpp2rust-demo/pull/137)、
> [#139](https://github.com/LuuuXXX/cpp2rust-demo/pull/139)、
> [#140](https://github.com/LuuuXXX/cpp2rust-demo/pull/140)、
> [#141](https://github.com/LuuuXXX/cpp2rust-demo/pull/141)、
> [#142](https://github.com/LuuuXXX/cpp2rust-demo/pull/142)、
> [#143](https://github.com/LuuuXXX/cpp2rust-demo/pull/143)、
> [#144](https://github.com/LuuuXXX/cpp2rust-demo/pull/144)。

---

## 1. 开发目标

v6 方案 §7 的 **Phase C（高级映射）** 共含四项，其中三项已在前序 PR 落地：

1. 抽象类 → `#[interface]` Trait —— **已落地**（`class_spec.rs` 的 `is_interface` 判定，参与默认产物）；
2. 私有析构 → `destroy = "..."` 属性 —— **已落地**（`ClassSpec.destroy_fn` / 生成器）；
3. 可选 `@make_proxy` 工厂 —— **已落地**（PR #144，`proxy_spec.rs`，受 `CPP2RUST_GEN_PROXY` 控制）；
4. **RTTI 场景 → `@dynamic_cast` 绑定 —— 本阶段实现**。

因此本阶段聚焦 `development-progress-phase-c.md`「后续计划」中列为**首项**的
**Phase C（续）：`@dynamic_cast` 下行转换绑定骨架**：

> 针对具体的 `(多态基类, 派生类)` 类型对，生成 hicc `@dynamic_cast` 下行转换骨架
> （形如 `#[cpp(func = "const Bar* @dynamic_cast<const Bar*>(const Foo*)")]`），
> 替代 v5 的整数枚举绕过方案（对应方案 §3.2 示例 023 typeid_rtti），
> 见 `references/hicc/examples/dynamic_cast`。

具体目标：

1. **识别下行转换目标**：在当前编译单元中识别「继承自某个**多态基类**（自身或祖先含
   虚函数 / 虚析构）的派生类」，对每个直接继承关系派生一个 `基类 → 派生类` 的下行转换。
2. **生成下行转换骨架**：在 `import_lib!` 中输出
   `const Derived* @dynamic_cast<const Derived*>(const Base*)`，Rust 侧以多态基类裸指针
   为入参、返回派生类裸指针（转换失败返回空指针，调用方判空）。
3. **硬约束（来自 v6 方案 §1.3 / §9）**：
   - `init` / `merge` 命令、参数、`.cpp2rust/<feature>/` 目录结构完全不变；
   - **默认产物逐字节不变** —— 本阶段能力由**新增**环境变量
     `CPP2RUST_GEN_DYNAMIC_CAST` 控制，默认关闭。

---

## 2. 详细方案

### 2.1 新增独立环境变量开关，保证默认产物不变

下行转换骨架的输出由**新增**开关 `CPP2RUST_GEN_DYNAMIC_CAST` 控制
（`generator::hicc_codegen::dynamic_cast_enabled`，常量名 `GEN_DYNAMIC_CAST_ENV`）：

- 默认关闭，仅当取值为 `1` / `true` / `yes` / `on`（忽略大小写）时启用；
- 与 `CPP2RUST_GEN_TEMPLATES`、`CPP2RUST_GEN_PROXY` 相互独立，互不影响；
- 关闭时生成器不输出任何 `@dynamic_cast` 相关内容，所有既有 L1 黄金 / L2 / L3 / L4 / L5
  基线均不受影响。

提取器侧（`extractor::dynamic_cast_spec`）**始终**构建规格（开销极小），是否输出由生成器
统一裁决，与 Phase B / Phase C 的既有节奏一致。

### 2.2 提取器：`src/extractor/dynamic_cast_spec.rs`

`build_dynamic_casts(ast)` 由 `CppAst` 构建 `Vec<DynamicCastSpec>`：

- 仅纳入来自当前编译单元（`is_from_current_file`）的派生类；
- 遍历每个类的**直接基类**，若基类在当前单元已知且为「多态类」，派生一个
  `基类 → 派生类` 的下行转换；
- **多态判定**（`is_polymorphic`）：类自身含任一 `is_virtual` / `is_pure_virtual` 方法
  （包括虚析构），或其任一（递归）基类为多态。这与 C++ 中 `dynamic_cast` 要求源类型为
  多态类型的约束一致——非多态类型上的 `dynamic_cast` 无法编译，故不予生成。

### 2.3 IR：`src/ffi_model.rs`

新增 `DynamicCastSpec`：

- `rust_name`：Rust 函数名（如 `dynamic_cast_foo_to_bar`，由 `to_snake_case` 派生）；
- `src_class` / `dst_class`：源（多态基类）/ 目标（派生类）类型名；
- `cpp_sig`：C++ 转换签名（如 `const Bar* @dynamic_cast<const Bar*>(const Foo*)`）。

`FfiSpec` 新增 `dynamic_casts: Vec<DynamicCastSpec>` 字段。

### 2.4 生成器：`src/generator/hicc_codegen.rs`

- `emit_dynamic_cast` 在 `import_lib!` 块内输出下行转换骨架，形如：

  ```rust
  // cpp2rust-todo[DCAST]: @dynamic_cast 下行转换骨架 —— 多态基类 Foo 向下转换为派生类 Bar；
  // 转换失败返回空指针，调用方需判空（is_null）。RTTI 要求源类型为多态类型（含虚函数）。
  #[cpp(func = "const Bar* @dynamic_cast<const Bar*>(const Foo*)")]
  pub unsafe fn dynamic_cast_foo_to_bar(src: *const Foo) -> *const Bar;
  ```

- `import_lib!` 块的「是否生成」判定补充 `has_dynamic_casts` 条件，使仅有下行转换骨架
  （且开关开启）的单元也能正确输出块；关闭时该条件恒为 `false`，不影响产物。

### 2.5 测试

- 单元测试（`dynamic_cast_spec.rs`）：验证多态基类派生下行转换、非多态基类跳过、
  多态性沿基类链传递、未知基类跳过。
- 集成测试（`tests/dynamic_cast_gen_tests.rs`）：对含多态基类 `Foo`/派生类 `Bar` 与
  非多态类 `Plain`/`PlainChild` 的 C++ 源码，验证：
  - **默认关闭**时不输出任何 `@dynamic_cast` 骨架与 `cpp2rust-todo[DCAST]` 占位；
  - **开启开关**时输出正确的 `Foo → Bar` 下行转换骨架，且非多态的 `Plain` 派生类不被生成。
  因开关为进程级环境变量，断言集中在单个 `#[test]` 中串行执行，避免并发竞态。

---

## 3. 详细进展

| 阶段 | 状态 | 说明 |
|------|------|------|
| IR：`DynamicCastSpec` + `FfiSpec.dynamic_casts` | ✅ 已完成 | `src/ffi_model.rs` |
| 提取器：`dynamic_cast_spec::build_dynamic_casts` | ✅ 已完成 | 多态基类 → 派生类下行转换，含多态性递归判定 |
| 生成器：`CPP2RUST_GEN_DYNAMIC_CAST` 开关 + `emit_dynamic_cast` | ✅ 已完成 | 默认关闭，受开关控制 |
| 接入：`extractor/mod.rs` | ✅ 已完成 | 始终构建规格，输出由生成器裁决 |
| 单元测试 + `dynamic_cast_gen_tests` | ✅ 已完成 | 263 lib 单测 + 集成测试全绿 |
| 回归验证（L1 黄金 / lib） | ✅ 已完成 | 52 L1 黄金 + 263 lib 单测全绿，默认产物逐字节不变 |
| 文档（目标 / 方案 / 进展 / 后续） | ✅ 已完成 | 本文档 |

**回归验证结论**：开关默认关闭时，L1 黄金（52）与 lib 单测（263）全部通过，确认默认
产物与改动前逐字节一致；开启 `CPP2RUST_GEN_DYNAMIC_CAST` 后下行转换骨架按预期输出。

> 说明：本仓库当前在较新版 clippy（rust 1.95.0）下存在 5 处既有告警
> （`capture.rs`、`commands/init.rs`、`extractor/mod.rs` 中与本阶段无关的行），
> 经 `git stash` 对照确认为**改动前已存在**，不在本阶段任务范围内；本阶段新增代码
> 不引入任何新的 clippy 告警。

---

## 4. 后续计划

以下为 v6 方案中尚未落地的阶段，建议作为后续独立 PR 推进（与既有节奏一致：先做风险最低、
可独立验证的垂直切片）：

- **Phase C（收尾）**：私有析构 `destroy = "..."` 在更多场景下的自动检测增强；
  `@dynamic_cast` 跨层 / 交叉转换（cross-cast）与「引用形式」返回（`&Derived`）的可选支持。
- **Phase E（examples 改造）**：将 023 typeid_rtti 从「整数枚举绕过」升级为「`@dynamic_cast`
  下行转换」，024/025/026/027 + 虚函数 / STL 选定示例从「手写包装降级」升级为「原生
  hicc 模板 / 接口 / RTTI 映射」，并为每个改造示例补充 `tests/smoke.rs`；同步更新各示例
  README。涉及 `main.rs` → `lib.rs` + `main.rs` 的结构调整与 L1 黄金提取目标变更，需分批、
  逐示例验证，风险较高。
- **Phase F（测试 / CI）**：L1 黄金适配模板骨架 / 别名 / 工厂 / 代理 / 下行转换片段；
  新增 `smoke` job 与 `gen-verify` 端到端 job（`init` → 生成目录 `cargo test`）。
- **Phase G（剩余文档）**：随 examples 改造同步更新各示例 README 的「模板 / 接口 / RTTI
  映射 + 冒烟测试」说明；`docs/references/hicc.md` 补充 `@dynamic_cast` 速查。

**风险提示**：下行转换当前生成的仍是「骨架」——`@dynamic_cast` 在转换失败时返回空指针，
Rust 侧返回裸指针需调用方自行判空；跨多层 / 交叉转换、含命名空间限定名的复杂类型仍需
用户结合实际类型确认。下行转换能力默认关闭的设计为上述后续阶段提供了安全的灰度通道：
可在开关开启下先验证生成质量，再决定是否纳入黄金基线。
