# v6 开发进展记录 — Phase E（示例改造，第一批）：024/025 升级为 lib.rs + main.rs 结构

> 本文档记录 v6 方案（见 `automated-cpp2rust-ffi-v6.md`）中 **Phase E（示例改造，第一批）**
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
> - `development-progress-phase-c-plus2.md`（Phase C（收尾）：跨层 / 间接 `@dynamic_cast`）
> - `development-progress-phase-c-plus3.md`（Phase C（收尾续）：`@dynamic_cast` 引用形式）
>
> 已合并的相关 PR：[#137](https://github.com/LuuuXXX/cpp2rust-demo/pull/137)、
> [#139](https://github.com/LuuuXXX/cpp2rust-demo/pull/139)–
> [#147](https://github.com/LuuuXXX/cpp2rust-demo/pull/147)。

---

## 1. 开发目标

v6 方案 Phase E 的核心目标是将手写包装类 / extern "C" 降级方案的示例升级为**使用原生
hicc 能力**的写法，并为每个示例补充 `tests/smoke.rs` 集成冒烟测试，验证 FFI 绑定在真实链接
与调用层面可用（"生成即验证"）。

本阶段（Phase E 第一批）聚焦**架构迁移**，完成以下目标：

1. **结构重构**：将示例 024（template_function）和 025（template_class）从「仅 `main.rs`
   单文件」迁移到「`lib.rs`（FFI 绑定）+ `main.rs`（演示）」的标准 lib + bin 结构。
2. **冒烟测试**：为两个示例新增 `tests/smoke.rs` 集成测试，覆盖主要 FFI 绑定的行为断言。
3. **L1 黄金测试扩展**：新增 `golden_test_lib!` 宏，支持从 `lib.rs` 读取黄金内容，并规范化
   `pub` 可见性修饰符后与工具输出对比。
4. **文档补充**：更新 `docs/references/hicc.md`，补充 v6 Phase A–C 落地的四项新能力
   （模板类/函数、`@make_proxy`、`@dynamic_cast`、冒烟测试生成）的速查说明。
5. **硬约束（来自 v6 方案 §1.3 / §9）**：
   - `init` / `merge` 命令、参数、`.cpp2rust/<feature>/` 目录结构完全不变；
   - 所有现有 lib 单测（265 个）与默认产物不受影响。

---

## 2. 详细方案

### 2.1 结构迁移策略

#### 改造前（仅 main.rs）

```
rust_hicc/
├── Cargo.toml          # 仅 binary crate
├── build.rs            # rust_file("src/main.rs")
└── src/
    └── main.rs         # hicc 块 + fn main() 演示
```

#### 改造后（lib.rs + main.rs）

```
rust_hicc/
├── Cargo.toml          # [lib] + [[bin]] 双目标
├── build.rs            # rust_file("src/lib.rs")
└── src/
│   ├── lib.rs          # hicc 块（FFI 绑定，pub 可见性）
│   └── main.rs         # use <crate>::*; + fn main() 演示
└── tests/
    └── smoke.rs        # 集成冒烟测试（行为断言）
```

#### 关键变化说明

1. **`lib.rs`**：将原 `main.rs` 的 `hicc::cpp!`、`hicc::import_class!`、
   `hicc::import_lib!` 三段 FFI 绑定迁移至 `lib.rs`。
   - `import_lib!` 中的函数声明由 `unsafe fn` 改为 **`pub unsafe fn`**，以便集成测试
     （`tests/smoke.rs`）和 `main.rs` 可通过 `use <crate>::*;` 访问。
   - `import_class!` 的 `pub class` 声明保持不变（已是 pub）。
2. **`main.rs`**：保留 `fn main()` 演示逻辑，移除 hicc 块，添加 `use <crate>::*;` 导入。
   演示输出与改造前完全一致，确保 L3 运行测试（`cargo run` 对比 README）不回归。
3. **`build.rs`**：将 `build.rust_file("src/main.rs")` 改为 `build.rust_file("src/lib.rs")`；
   `rerun-if-changed` 同步更新为 `src/lib.rs`。
4. **`Cargo.toml`**：新增 `[lib]` 与 `[[bin]]` 显式目标声明。

### 2.2 L1 黄金测试扩展

新增 `golden_test_lib!` 宏（`tests/l1_golden_tests.rs`），与现有 `golden_test!` 的差异：

| 项目 | `golden_test!` | `golden_test_lib!` |
|------|---------------|-------------------|
| 黄金文件路径 | `rust_hicc/src/main.rs` | `rust_hicc/src/lib.rs` |
| pub 规范化 | 无 | 调用 `common::strip_pub_visibility` 移除 `pub unsafe fn` / `pub fn` 的 `pub ` 前缀 |
| 比较逻辑 | 工具输出 vs 黄金 | 工具输出 vs 规范化后黄金 |

`strip_pub_visibility`（`tests/common/mod.rs`）将 `pub unsafe fn` 规范化为 `unsafe fn`，
`pub fn` 规范化为 `fn`，其余内容不变。这确保 L1 测试验证**结构正确性**，同时允许 `lib.rs`
为集成测试目的使用 `pub` 可见性。

### 2.3 冒烟测试设计

冒烟测试（`tests/smoke.rs`）为 Cargo 集成测试，通过 `use <crate>::*;` 引用 `lib.rs` 导出
的公有 FFI 绑定：

| 示例 | 测试覆盖 |
|------|---------|
| 024 | `swap_int`/`swap_double`/`swap_char` 行为断言；`get_int_array`/`set_int_array` 读写断言；`swap_int_array` 数组元素交换断言 |
| 025 | `IntStack` 基本操作（push/top/pop/size/empty）；`DoubleStack` 基本操作；类型可用性断言 |

冒烟测试遵循「最小可验证行为」原则：只对可预期的值做 `assert_eq!`，不测试 C++ 侧的实现细节。

---

## 3. 详细进展

| 阶段 | 状态 | 说明 |
|------|------|------|
| 024 lib.rs（FFI 绑定，pub） | ✅ 已完成 | `src/lib.rs` 含 6 个 pub unsafe fn |
| 024 main.rs（演示，use 导入） | ✅ 已完成 | 仅保留 fn main()，用 use template_function::* 导入 |
| 024 tests/smoke.rs | ✅ 已完成 | 5 个行为断言测试 |
| 024 Cargo.toml（lib + bin） | ✅ 已完成 | [lib] + [[bin]] 双目标 |
| 024 build.rs（lib.rs） | ✅ 已完成 | rust_file("src/lib.rs") |
| 025 lib.rs（FFI 绑定，pub） | ✅ 已完成 | IntStack/DoubleStack pub class + pub fn 工厂 |
| 025 main.rs（演示，use 导入） | ✅ 已完成 | 仅保留 fn main()，用 use template_class::* 导入 |
| 025 tests/smoke.rs | ✅ 已完成 | 3 个集成测试（IntStack/DoubleStack 行为 + 类型断言） |
| 025 Cargo.toml（lib + bin） | ✅ 已完成 | [lib] + [[bin]] 双目标 |
| 025 build.rs（lib.rs） | ✅ 已完成 | rust_file("src/lib.rs") |
| L1 golden：golden_test_lib! 宏 | ✅ 已完成 | 读 lib.rs，规范化 pub 后比较 |
| common::strip_pub_visibility | ✅ 已完成 | tests/common/mod.rs |
| docs/references/hicc.md 补充 | ✅ 已完成 | v6 Phase A–C 四项新能力速查 |
| 回归验证（265 lib 单测） | ✅ 已完成 | cargo test --lib 全绿，默认产物不变 |

**回归验证结论**：265 个 lib 单测在本次改动后全部通过，确认默认产物未受影响。
024/025 的 L2 编译测试（`cargo build`）与 L3 运行测试（`cargo run` 对比 README）
所依赖的 `fn main()` 逻辑及输出均保持不变，改动完全向后兼容。

---

## 4. 后续计划

Phase E 第一批完成了 024/025 的结构迁移。v6 方案中 Phase E 剩余示例可按相同模式逐步迁移：

- **Phase E（第二批）**：示例 026（template_specialization）和 027（template_instantiation）
  按相同结构改造（lib.rs + main.rs + smoke.rs），接入模板实例化别名生成（`CPP2RUST_GEN_TEMPLATES=1`）。
- **Phase E（第三批）**：示例 015–018（virtual_*）改造为 `#[interface]` Trait 结构，
  接入 `@make_proxy` 代理工厂（`CPP2RUST_GEN_PROXY=1`）；涉及 macOS ARM64 虚函数
  崩溃问题的平台跳过策略与现有 L3 保持一致。
- **Phase E（第四批）**：示例 023（typeid_rtti）从「整数枚举绕过」升级为
  `@dynamic_cast` 下行转换（`CPP2RUST_GEN_DYNAMIC_CAST=1`）；需同步更新 C++ 侧声明
  以暴露具体子类 Rust 侧可见。
- **Phase E（第五批）**：STL 容器示例（034–038）参考 `hicc-std` 写法，引入泛型
  class + Pod/class 实例化别名；平台差异较大，建议分步验证。
- **Phase F（测试/CI）**：
  - 为 024/025 在 CI 中启用 L_smoke 测试（`cargo test` 运行 smoke.rs）；
  - 新增 `gen-verify` 端到端 job（`init` → 生成目录 `cargo test`），验证工具实际输出。
- **Phase C（剩余）**：命名空间限定名（`ns::Base`）的 `@dynamic_cast` 下行转换；
  私有析构 `destroy = "..."` 自动检测增强。
- **Phase G（剩余）**：随 Phase E 后续批次更新各示例 README 的「模板 / 接口 / RTTI
  映射 + 冒烟测试」说明。
