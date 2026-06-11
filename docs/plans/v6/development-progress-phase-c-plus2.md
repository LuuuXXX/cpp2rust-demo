# v6 开发进展记录 — Phase C（收尾）：跨层 `@dynamic_cast` 下行转换骨架

> 本文档记录 v6 方案（见 `automated-cpp2rust-ffi-v6.md`）中 **Phase C（收尾，高级映射）**
> 阶段的开发目标、详细方案、详细进展与后续计划。全程使用简体中文。
>
> 前序进展：
> - `development-progress.md`（Phase A/B：模板类 / 模板函数泛型骨架）
> - `development-progress-phase-b-plus.md`（Phase B 增强：模板实例化别名）
> - `development-progress-phase-b-plus2.md`（Phase B 增强（续）：实例化追踪扩展 + 构造工厂骨架）
> - `development-progress-phase-b-plus3.md`（Phase B 增强（再续）：显式实例化追踪）
> - `development-progress-phase-b-plus4.md`（Phase B 增强（收尾）：局部变量声明实例化追踪）
> - `development-progress-phase-c.md`（Phase C：`@make_proxy` 代理工厂骨架）
> - `development-progress-phase-c-plus.md`（Phase C（续）：`@dynamic_cast` 下行转换骨架）
>
> 已合并的相关 PR：[#137](https://github.com/LuuuXXX/cpp2rust-demo/pull/137)、
> [#139](https://github.com/LuuuXXX/cpp2rust-demo/pull/139)、
> [#140](https://github.com/LuuuXXX/cpp2rust-demo/pull/140)、
> [#141](https://github.com/LuuuXXX/cpp2rust-demo/pull/141)、
> [#142](https://github.com/LuuuXXX/cpp2rust-demo/pull/142)、
> [#143](https://github.com/LuuuXXX/cpp2rust-demo/pull/143)、
> [#144](https://github.com/LuuuXXX/cpp2rust-demo/pull/144)、
> [#145](https://github.com/LuuuXXX/cpp2rust-demo/pull/145)。

---

## 1. 开发目标

`development-progress-phase-c-plus.md`（PR #145）已落地 `@dynamic_cast` 下行转换骨架，
但**仅覆盖直接基类**：对每个 `(直接多态基类, 派生类)` 关系派生一个下行转换。

本阶段实现该文档「后续计划」中 **Phase C（收尾）** 列为首项的能力：

> `@dynamic_cast` **跨层 / 间接（cross-layer）下行转换**支持。

具体目标：

1. **跨层下行转换**：除直接基类外，遍历派生类的**递归祖先链**，为每个**多态祖先**
   （直接或间接）派生一个 `祖先 → 派生类` 的下行转换骨架。例如 `Foo <- Bar <- Baz`
   除已有的 `Foo → Bar`、`Bar → Baz` 外，额外派生跨层的 `Foo → Baz`。这与 C++ 允许
   `dynamic_cast` 跨任意继承层级向下转换的语义一致。
2. **去重**：菱形 / 重复继承下，同一 `(src, dst)` 仅产出一个骨架。
3. **硬约束（来自 v6 方案 §1.3 / §9）**：
   - `init` / `merge` 命令、参数、`.cpp2rust/<feature>/` 目录结构完全不变；
   - **默认产物逐字节不变** —— 本阶段能力仍由既有环境变量
     `CPP2RUST_GEN_DYNAMIC_CAST` 控制，默认关闭，**不新增任何开关**。

---

## 2. 详细方案

### 2.1 复用既有开关，保证默认产物不变

跨层下行转换的输出仍由 PR #145 引入的 `CPP2RUST_GEN_DYNAMIC_CAST` 开关裁决
（`generator::hicc_codegen::dynamic_cast_enabled`）：

- 默认关闭，仅当取值为 `1` / `true` / `yes` / `on`（忽略大小写）时启用；
- 提取器侧（`extractor::dynamic_cast_spec`）**始终**构建规格（含跨层），是否输出由生成器
  统一裁决；
- 关闭时不输出任何 `@dynamic_cast` 相关内容，所有既有 L1 黄金 / L2 / L3 / L4 / L5
  基线均不受影响。

### 2.2 提取器：`src/extractor/dynamic_cast_spec.rs`

`build_dynamic_casts(ast)` 的遍历策略由「仅直接基类」升级为「递归祖先链」：

- 新增 `collect_ancestors(ci, all_classes, out, visited)`：自顶向下递归收集派生类的
  全部祖先（直接基类 + 间接祖先），`visited` 集合防止循环 / 重复继承导致的无限递归与
  重复收集（应对菱形继承）。
- 对收集到的每个祖先，沿用既有的 `is_polymorphic` 判定（类自身或任一递归基类含虚函数 /
  虚析构 / 纯虚方法）；仅对**多态祖先**派生下行转换——非多态类型上的 `dynamic_cast`
  无法编译，故不予生成。
- 用 `HashSet<(String, String)>` 按 `(src, dst)` 去重，保证菱形 / 重复继承下不产出
  重复骨架。

派生单个骨架的 `build_one` 与生成器 `emit_dynamic_cast` 保持不变——跨层骨架与直接基类
骨架在形态上完全一致（仅 `(src, dst)` 类型对不同），形如：

```rust
// cpp2rust-todo[DCAST]: @dynamic_cast 下行转换骨架 —— 多态基类 Foo 向下转换为派生类 Baz；
// 转换失败返回空指针，调用方需判空（is_null）。RTTI 要求源类型为多态类型（含虚函数）。
#[cpp(func = "const Baz* @dynamic_cast<const Baz*>(const Foo*)")]
pub unsafe fn dynamic_cast_foo_to_baz(src: *const Foo) -> *const Baz;
```

### 2.3 测试

- 单元测试（`dynamic_cast_spec.rs`）：
  - `polymorphism_propagates_through_base_chain`：扩展断言，验证三层继承下额外派生
    跨层的 `Foo → Baz`；
  - 新增 `derives_cross_layer_downcast_skips_non_polymorphic_ancestor`：验证中间类继承
    多态祖先后自身也成为多态类，链上各 `(祖先, 派生)` 对均派生；
  - 新增 `dedups_cross_layer_downcasts`：菱形继承（`Base <- L, R`；`Diamond <- L, R`）下
    `Base → Diamond` 仅产出一个骨架。
- 集成测试（`tests/dynamic_cast_gen_tests.rs`）：源码新增三层继承 `Foo <- Bar <- Baz`，
  开启开关后断言生成跨层 `Foo → Baz` 与直接 `Bar → Baz` 下行转换；默认关闭断言不变。

---

## 3. 详细进展

| 阶段 | 状态 | 说明 |
|------|------|------|
| 提取器：`collect_ancestors` 递归祖先收集 | ✅ 已完成 | `src/extractor/dynamic_cast_spec.rs` |
| 提取器：`build_dynamic_casts` 跨层 + 去重 | ✅ 已完成 | 复用 `is_polymorphic`，`HashSet` 去重 |
| 单元测试（跨层 / 链传递 / 菱形去重） | ✅ 已完成 | 6 个单测（新增 2 + 扩展 1）全绿 |
| 集成测试 `dynamic_cast_gen_tests` 跨层断言 | ✅ 已完成 | `--features full-test` 下全绿 |
| 回归验证（L1 黄金 / lib） | ✅ 已完成 | 52 L1 黄金 + 265 lib 单测全绿，默认产物逐字节不变 |
| 文档（DEVELOPMENT.md + 本文档） | ✅ 已完成 | 同步 `@dynamic_cast` 说明，新增本进展文档 |

**回归验证结论**：开关默认关闭时，L1 黄金（52）与 lib 单测（265）全部通过，确认默认
产物与改动前逐字节一致；开启 `CPP2RUST_GEN_DYNAMIC_CAST` 后跨层下行转换骨架按预期输出。

> 说明：本仓库当前在较新版 clippy 下存在数处**改动前已存在**的告警
> （`capture.rs`、`commands/init.rs`、`extractor/mod.rs`），与本阶段无关；本阶段新增代码
> 不引入任何新的 clippy 告警，`cargo fmt --check` 通过。

---

## 4. 后续计划

以下为 v6 方案中尚未落地的阶段，建议作为后续独立 PR 推进（与既有节奏一致：先做风险最低、
可独立验证的垂直切片）：

- **Phase C（剩余）**：`@dynamic_cast` 的「引用形式」返回（`&Derived`）可选支持；
  含命名空间限定名（`ns::Base`）的复杂类型对的下行转换；私有析构 `destroy = "..."`
  在更多场景下的自动检测增强。
- **Phase E（examples 改造）**：将 023 typeid_rtti 从「整数枚举绕过」升级为「`@dynamic_cast`
  下行转换」，024/025/026/027 + 虚函数 / STL 选定示例从「手写包装降级」升级为「原生
  hicc 模板 / 接口 / RTTI 映射」，并为每个改造示例补充 `tests/smoke.rs`；同步更新各示例
  README。涉及 `main.rs` → `lib.rs` + `main.rs` 的结构调整与 L1 黄金提取目标变更，需分批、
  逐示例验证，风险较高。
- **Phase F（测试 / CI）**：L1 黄金适配模板骨架 / 别名 / 工厂 / 代理 / 下行转换片段；
  新增 `smoke` job 与 `gen-verify` 端到端 job（`init` → 生成目录 `cargo test`）。
- **Phase G（剩余文档）**：随 examples 改造同步更新各示例 README 的「模板 / 接口 / RTTI
  映射 + 冒烟测试」说明；`docs/references/hicc.md` 补充 `@dynamic_cast` 速查。

**风险提示**：跨层下行转换生成的仍是「骨架」——`@dynamic_cast` 在转换失败时返回空指针，
Rust 侧返回裸指针需调用方自行判空；交叉转换（cross-cast，同层非祖先类型间）、含命名空间
限定名的复杂类型仍需用户结合实际类型确认。能力默认关闭的设计为上述后续阶段提供了安全的
灰度通道。
