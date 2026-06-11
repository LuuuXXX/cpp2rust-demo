# v6 开发进展记录 — Phase C（收尾续）：`@dynamic_cast` 引用形式（`&Derived`）下行转换骨架

> 本文档记录 v6 方案（见 `automated-cpp2rust-ffi-v6.md`）中 **Phase C（收尾续，高级映射）**
> 阶段的开发目标、详细方案、详细进展与后续计划。全程使用简体中文。
>
> 前序进展：
> - `development-progress.md`（Phase A/B：模板类 / 模板函数泛型骨架）
> - `development-progress-phase-b-plus.md`（Phase B 增强：模板实例化别名）
> - `development-progress-phase-b-plus2.md`（Phase B 增强（续）：实例化追踪扩展 + 构造工厂骨架）
> - `development-progress-phase-b-plus3.md`（Phase B 增强（再续）：显式实例化追踪）
> - `development-progress-phase-b-plus4.md`（Phase B 增强（收尾）：局部变量声明实例化追踪）
> - `development-progress-phase-c.md`（Phase C：`@make_proxy` 代理工厂骨架）
> - `development-progress-phase-c-plus.md`（Phase C（续）：`@dynamic_cast` 直接基类下行转换骨架）
> - `development-progress-phase-c-plus2.md`（Phase C（收尾）：跨层 / 间接 `@dynamic_cast` 下行转换骨架）
>
> 已合并的相关 PR：[#137](https://github.com/LuuuXXX/cpp2rust-demo/pull/137)、
> [#139](https://github.com/LuuuXXX/cpp2rust-demo/pull/139)、
> [#140](https://github.com/LuuuXXX/cpp2rust-demo/pull/140)、
> [#141](https://github.com/LuuuXXX/cpp2rust-demo/pull/141)、
> [#142](https://github.com/LuuuXXX/cpp2rust-demo/pull/142)、
> [#143](https://github.com/LuuuXXX/cpp2rust-demo/pull/143)、
> [#144](https://github.com/LuuuXXX/cpp2rust-demo/pull/144)、
> [#145](https://github.com/LuuuXXX/cpp2rust-demo/pull/145)、
> [#146](https://github.com/LuuuXXX/cpp2rust-demo/pull/146)。

---

## 1. 开发目标

`development-progress-phase-c-plus2.md`（PR #146）已落地**跨层 / 间接** `@dynamic_cast`
下行转换骨架，但所有骨架**仅以裸指针形式**输出（`*const Base -> *const Derived`，转换失败
返回空指针，调用方判空）。

本阶段实现 `development-progress-phase-c-plus2.md`「后续计划」中 **Phase C（剩余）** 列为
首项的能力：

> `@dynamic_cast` 的「引用形式」返回（`&Derived`）可选支持。

具体目标：

1. **引用形式骨架**：在既有裸指针形式之外，为每个下行转换额外派生一个**引用形式**函数
   （`&Src -> &Dst`，函数名在裸指针形式基础上追加 `_ref` 后缀）。两种形式复用**同一指针型
   C++ 签名** `const Dst* @dynamic_cast<const Dst*>(const Src*)`——这与 hicc 官方
   `examples/dynamic_cast` 的写法一致：`as_bar(&self) -> *const Bar` 与
   `as_foo(&self) -> &Foo` 共用指针型 `@dynamic_cast` 签名，仅 Rust 侧返回类型不同。
2. **安全语义提示**：引用形式更符合 Rust 习惯，但**要求转换必定成功**——若转换失败
   （基类指针实际并非该派生类），hicc 会以空指针构造引用，属未定义行为。因此引用形式骨架
   附带专门的 `cpp2rust-todo[DCAST]` 提示：仅在调用方能确保类型成立时使用，否则改用裸指针
   形式并判空。
3. **硬约束（来自 v6 方案 §1.3 / §9）**：
   - `init` / `merge` 命令、参数、`.cpp2rust/<feature>/` 目录结构完全不变；
   - **默认产物逐字节不变** —— 引用形式仍由既有环境变量 `CPP2RUST_GEN_DYNAMIC_CAST`
     控制，默认关闭，**不新增任何开关**。

---

## 2. 详细方案

### 2.1 复用既有开关，保证默认产物不变

引用形式的输出仍由 PR #145 引入的 `CPP2RUST_GEN_DYNAMIC_CAST` 开关裁决
（`generator::hicc_codegen::dynamic_cast_enabled`）：

- 默认关闭，仅当取值为 `1` / `true` / `yes` / `on`（忽略大小写）时启用；
- 提取器侧（`extractor::dynamic_cast_spec`）**始终**构建规格（含引用形式函数名），是否输出由
  生成器统一裁决；
- 关闭时不输出任何 `@dynamic_cast` 相关内容（裸指针形式与引用形式皆不输出），所有既有
  L1 黄金 / L2 / L3 / L4 / L5 基线均不受影响。

### 2.2 IR：`src/ffi_model.rs`

`DynamicCastSpec` 新增字段 `ref_rust_name: String`，保存引用形式的 Rust 函数名（如
`dynamic_cast_foo_to_bar_ref`）。裸指针形式函数名 `rust_name`、源 / 目标类型、C++ 签名
`cpp_sig` 均保持不变——两种形式共用同一 `cpp_sig`。

### 2.3 提取器：`src/extractor/dynamic_cast_spec.rs`

`build_one(base, derived)` 在构造规格时，由裸指针形式名追加 `_ref` 派生引用形式名：

```rust
let rust_name = format!("dynamic_cast_{}_to_{}", to_snake_case(base), to_snake_case(derived));
let ref_rust_name = format!("{}_ref", rust_name);
```

跨层 / 去重逻辑（`collect_ancestors`、`is_polymorphic`、`HashSet` 去重）完全不变——引用形式
随裸指针形式一并派生，无需额外遍历。

### 2.4 生成器：`src/generator/hicc_codegen.rs`

`emit_dynamic_cast` 在输出裸指针形式之后，追加引用形式骨架（复用同一 `cpp_sig`）：

```rust
// cpp2rust-todo[DCAST]: @dynamic_cast 下行转换骨架 —— 多态基类 Foo 向下转换为派生类 Bar；
// 转换失败返回空指针，调用方需判空（is_null）。RTTI 要求源类型为多态类型（含虚函数）。
#[cpp(func = "const Bar* @dynamic_cast<const Bar*>(const Foo*)")]
pub unsafe fn dynamic_cast_foo_to_bar(src: *const Foo) -> *const Bar;
// cpp2rust-todo[DCAST]: 引用形式 —— 仅在转换必定成功时使用；否则请用上面的裸指针形式判空。
#[cpp(func = "const Bar* @dynamic_cast<const Bar*>(const Foo*)")]
pub unsafe fn dynamic_cast_foo_to_bar_ref(src: &Foo) -> &Bar;
```

### 2.5 测试

- 单元测试（`dynamic_cast_spec.rs`）：扩展 `derives_downcast_for_polymorphic_base`，
  断言 `ref_rust_name == "dynamic_cast_foo_to_bar_ref"`。
- 集成测试（`tests/dynamic_cast_gen_tests.rs`）：开启开关后断言生成引用形式
  `dynamic_cast_foo_to_bar_ref(src: &Foo) -> &Bar` 及跨层引用形式
  `dynamic_cast_foo_to_baz_ref(src: &Foo) -> &Baz`；默认关闭断言不变（引用形式名包含
  既有裸指针形式名子串，原有「不含 `dynamic_cast_foo_to_bar`」断言同时覆盖两者）。

---

## 3. 详细进展

| 阶段 | 状态 | 说明 |
|------|------|------|
| IR：`DynamicCastSpec.ref_rust_name` | ✅ 已完成 | `src/ffi_model.rs` |
| 提取器：`build_one` 派生引用形式名 | ✅ 已完成 | `src/extractor/dynamic_cast_spec.rs` |
| 生成器：`emit_dynamic_cast` 追加引用形式 | ✅ 已完成 | `src/generator/hicc_codegen.rs` |
| 单元测试（引用形式名断言） | ✅ 已完成 | 6 个 `dynamic_cast_spec` 单测全绿 |
| 集成测试 `dynamic_cast_gen_tests` 引用形式断言 | ✅ 已完成 | `--features full-test` 下全绿 |
| 回归验证（L1 黄金 / lib） | ✅ 已完成 | 52 L1 黄金 + 265 lib 单测全绿，默认产物逐字节不变 |
| 文档（DEVELOPMENT.md + 本文档） | ✅ 已完成 | 同步引用形式说明，新增本进展文档 |

**回归验证结论**：开关默认关闭时，L1 黄金（52）与 lib 单测（265）全部通过，确认默认
产物与改动前逐字节一致；开启 `CPP2RUST_GEN_DYNAMIC_CAST` 后裸指针形式与引用形式下行转换
骨架按预期一并输出。`cargo fmt --check` 通过；本阶段新增代码不引入新的 clippy 告警。

> 说明：本仓库当前在较新版 clippy 下存在数处**改动前已存在**的告警
> （`capture.rs`、`commands/init.rs`、`extractor/mod.rs`），与本阶段无关。

---

## 4. 后续计划

以下为 v6 方案中尚未落地的阶段，建议作为后续独立 PR 推进（与既有节奏一致：先做风险最低、
可独立验证的垂直切片）：

- **Phase C（剩余）**：含命名空间限定名（`ns::Base`）的复杂类型对下行转换（需 AST 层补齐
  命名空间路径信息）；私有析构 `destroy = "..."` 在更多场景下的自动检测增强。
- **Phase E（examples 改造）**：将 023 typeid_rtti 从「整数枚举绕过」升级为「`@dynamic_cast`
  下行转换」，024/025/026/027 + 虚函数 / STL 选定示例从「手写包装降级」升级为「原生
  hicc 模板 / 接口 / RTTI 映射」，并为每个改造示例补充 `tests/smoke.rs`；同步更新各示例
  README。涉及 `main.rs` → `lib.rs` + `main.rs` 的结构调整与 L1 黄金提取目标变更，需分批、
  逐示例验证，风险较高。
- **Phase F（测试 / CI）**：L1 黄金适配模板骨架 / 别名 / 工厂 / 代理 / 下行转换片段；
  新增 `smoke` job 与 `gen-verify` 端到端 job（`init` → 生成目录 `cargo test`）。
- **Phase G（剩余文档）**：随 examples 改造同步更新各示例 README 的「模板 / 接口 / RTTI
  映射 + 冒烟测试」说明；`docs/references/hicc.md` 补充 `@dynamic_cast` 速查。

**风险提示**：引用形式下行转换在转换失败时由空指针构造引用，属未定义行为，故仅作为「转换
必定成功」场景下的便捷写法，并附 `cpp2rust-todo[DCAST]` 提示用户审核；无法确保类型成立时
应改用裸指针形式判空。能力默认关闭的设计为上述后续阶段提供了安全的灰度通道。
