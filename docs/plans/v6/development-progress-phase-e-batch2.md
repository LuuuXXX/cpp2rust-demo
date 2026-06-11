# v6 开发进展记录 — Phase E（示例改造，第二批 & 第三批）

> 本文档记录 v6 方案（见 `automated-cpp2rust-ffi-v6.md`）中 **Phase E（示例改造，第二批 & 第三批）**
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
> - `development-progress-phase-e.md`（Phase E 第一批：024/025 升级为 lib.rs + main.rs 结构）

---

## 1. 开发目标

在 Phase E 第一批（024/025）完成的基础上，按照相同的「lib.rs + main.rs + tests/smoke.rs」
结构模式，本阶段（第二批 & 第三批）将以下示例完成结构迁移与冒烟测试补充：

**第二批（模板相关）**：
- **026_template_specialization**：模板偏特化示例（IntHolder / DoubleHolder / StringHolder）
- **027_template_instantiation**：模板显式实例化示例（IntMatrix / DoubleMatrix）

**第三批（虚函数相关）**：
- **015_virtual_basic**：基本虚函数示例（Shape / Circle）
- **016_virtual_pure**：纯虚函数示例（AbstractShape 抽象接口）
- **017_virtual_override**：虚函数 override 示例（Base / Derived）
- **018_virtual_diamond**：菱形继承示例（虚继承，A → B/C → D）

每个示例的迁移目标：
1. **结构重构**：从「仅 `main.rs` 单文件」迁移到「`lib.rs`（FFI 绑定，pub 方法）+ `main.rs`（演示，use 导入）」。
2. **冒烟测试**：新增 `tests/smoke.rs` 集成测试，覆盖主要 FFI 绑定的行为断言。
3. **L1 黄金测试**：将对应示例的测试宏从 `golden_test!` 更新为 `golden_test_lib!`，
   从 `lib.rs` 读取黄金内容并规范化 pub 可见性后比较。
4. **硬约束**：
   - `init` / `merge` 命令、参数、目录结构完全不变；
   - 265 个 lib 单测全部通过，默认产物不受影响。

---

## 2. 详细方案

与 Phase E 第一批相同，沿用「lib.rs + main.rs」结构迁移策略：

### 2.1 文件改造模式

| 文件 | 改造内容 |
|------|---------|
| `src/lib.rs`（新增） | 将原 `main.rs` 的 hicc 三段块（`cpp!`/`import_class!`/`import_lib!`）迁入；所有方法声明及工厂函数改为 `pub fn`/`pub unsafe fn` |
| `src/main.rs`（改造） | 移除 hicc 块，添加 `use <crate>::*;`（及需要时 `use hicc::AbiClass;`）；保留完整 `fn main()` 演示逻辑 |
| `tests/smoke.rs`（新增） | 行为断言集成测试，通过 `use <crate>::*;` 调用 `lib.rs` 公有绑定 |
| `Cargo.toml`（更新） | 添加 `[lib]` + `[[bin]]` 显式目标声明 |
| `build.rs`（更新） | `rust_file("src/main.rs")` → `rust_file("src/lib.rs")`；`rerun-if-changed` 同步更新 |

### 2.2 特殊处理

- **016_virtual_pure**：工厂函数返回 `*mut AbstractShape`（原始指针），`into_value()`/`into_unique()` 需要 `hicc::AbiClass` trait。因此 lib.rs 顶部保留 `use hicc::AbiClass;`，main.rs 同样补充 `use hicc::AbiClass;`，smoke.rs 也同样补充。
- **018_virtual_diamond**：`d_get_a_value` 参数为 `*mut D`，需 `AbiClass::as_mut_ptr()` 方法，lib.rs 和相关调用处需 `use hicc::AbiClass;`。
- **golden_test_lib! 兼容性**：`strip_pub_visibility` 函数（`tests/common/mod.rs`）将 `pub unsafe fn`/`pub fn` 规范化为不带 `pub` 的形式，与工具生成器的输出格式匹配；`use hicc::AbiClass;` 等非 hicc 块内容由 `extract_hicc_blocks` 过滤，不影响比较。

---

## 3. 详细进展

### 3.1 第二批（模板相关）

| 示例 | 文件 | 状态 |
|------|------|------|
| 026_template_specialization | `src/lib.rs` | ✅ 新增（3 个 class + 3 个工厂，全 pub） |
| 026_template_specialization | `src/main.rs` | ✅ 改造（use template_specialization::*） |
| 026_template_specialization | `tests/smoke.rs` | ✅ 新增（5 个行为断言测试） |
| 026_template_specialization | `Cargo.toml` | ✅ 更新（[lib] + [[bin]]） |
| 026_template_specialization | `build.rs` | ✅ 更新（lib.rs） |
| 027_template_instantiation | `src/lib.rs` | ✅ 新增（2 个 class + 2 个工厂，全 pub） |
| 027_template_instantiation | `src/main.rs` | ✅ 改造（use template_instantiation::*） |
| 027_template_instantiation | `tests/smoke.rs` | ✅ 新增（4 个行为断言测试） |
| 027_template_instantiation | `Cargo.toml` | ✅ 更新（[lib] + [[bin]]） |
| 027_template_instantiation | `build.rs` | ✅ 更新（lib.rs） |

### 3.2 第三批（虚函数相关）

| 示例 | 文件 | 状态 |
|------|------|------|
| 015_virtual_basic | `src/lib.rs` | ✅ 新增（Shape/Circle，全 pub 方法） |
| 015_virtual_basic | `src/main.rs` | ✅ 改造（use virtual_basic::*） |
| 015_virtual_basic | `tests/smoke.rs` | ✅ 新增（4 个断言：半径/面积/名称/shape_new） |
| 015_virtual_basic | `Cargo.toml` | ✅ 更新（[lib] + [[bin]]） |
| 015_virtual_basic | `build.rs` | ✅ 更新（lib.rs） |
| 016_virtual_pure | `src/lib.rs` | ✅ 新增（AbstractShape + 工厂，含 AbiClass） |
| 016_virtual_pure | `src/main.rs` | ✅ 改造（use hicc::AbiClass + use virtual_pure::*） |
| 016_virtual_pure | `tests/smoke.rs` | ✅ 新增（3 个断言：圆面积/矩形面积/getName） |
| 016_virtual_pure | `Cargo.toml` | ✅ 更新（[lib] + [[bin]]） |
| 016_virtual_pure | `build.rs` | ✅ 更新（lib.rs） |
| 017_virtual_override | `src/lib.rs` | ✅ 新增（Base/Derived，全 pub 方法） |
| 017_virtual_override | `src/main.rs` | ✅ 改造（use virtual_override::*） |
| 017_virtual_override | `tests/smoke.rs` | ✅ 新增（3 个断言：Derived 值/area/base_create） |
| 017_virtual_override | `Cargo.toml` | ✅ 更新（[lib] + [[bin]]） |
| 017_virtual_override | `build.rs` | ✅ 更新（lib.rs） |
| 018_virtual_diamond | `src/lib.rs` | ✅ 新增（D 类 + 工厂 + d_get_a_value，AbiClass） |
| 018_virtual_diamond | `src/main.rs` | ✅ 改造（use hicc::AbiClass + use virtual_diamond::*） |
| 018_virtual_diamond | `tests/smoke.rs` | ✅ 新增（2 个断言：D 值 + compute） |
| 018_virtual_diamond | `Cargo.toml` | ✅ 更新（[lib] + [[bin]]） |
| 018_virtual_diamond | `build.rs` | ✅ 更新（lib.rs） |

### 3.3 L1 黄金测试更新

| 示例 | 变更 | 状态 |
|------|------|------|
| 026_template_specialization | `golden_test!` → `golden_test_lib!` | ✅ |
| 027_template_instantiation | `golden_test!` → `golden_test_lib!` | ✅ |
| 015_virtual_basic | `golden_test!` → `golden_test_lib!` | ✅ |
| 016_virtual_pure | `golden_test!` → `golden_test_lib!` | ✅ |
| 017_virtual_override | `golden_test!` → `golden_test_lib!` | ✅ |
| 018_virtual_diamond | `golden_test!` → `golden_test_lib!` | ✅ |

### 3.4 回归验证

| 验证项 | 状态 | 说明 |
|--------|------|------|
| `cargo test --lib`（265 个 lib 单测） | ✅ 通过 | 全部通过，默认产物不变 |
| `cargo check`（主 crate） | ✅ 通过 | 无编译错误 |

---

## 4. 后续计划

本阶段完成了 026/027（模板）与 015-018（虚函数）共 6 个示例的 lib.rs + main.rs + smoke.rs 结构改造。
v6 方案中 Phase E 剩余工作：

- **Phase E（第四批）**：示例 023（typeid_rtti）从「整数枚举绕过」升级为 `@dynamic_cast` 下行转换
  （`CPP2RUST_GEN_DYNAMIC_CAST=1`）；需同步更新 C++ 侧声明以暴露具体子类为 Rust 可见。
- **Phase E（第五批）**：STL 容器示例（034–038）参考 `hicc-std` 写法，引入泛型
  class + Pod/class 实例化别名；平台差异较大，建议分步验证。
- **Phase F（测试 / CI）**：
  - 为改造示例在 CI 中启用 L_smoke 测试（`cargo test` 运行 smoke.rs）；
  - 新增 `gen-verify` 端到端 job（`init` → 生成目录 `cargo test`），验证工具实际输出。
- **Phase G（剩余文档）**：
  - `docs/INTRODUCTION.md` 补充模板类/模板函数/接口映射章节；
  - `README.md` 更新模板类/模板函数/虚函数的支持状态（从「⚠️ 包装降级」改为「✅ 原生 hicc 映射」）；
  - 各示例 README 说明「冒烟测试」运行方式。
