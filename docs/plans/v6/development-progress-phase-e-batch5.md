# v6 开发进展记录 — Phase E（示例改造，第五批）+ Phase F（CI smoke job）+ Phase G（文档收尾）

> 本文档记录 v6 方案（见 `automated-cpp2rust-ffi-v6.md`）中 **Phase E（示例改造，第五批）**、
> **Phase F（CI 新增 L_smoke job）** 与 **Phase G（文档收尾）** 阶段的开发目标、详细方案、
> 详细进展与后续计划。全程使用简体中文。
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
> - `development-progress-phase-e-batch2.md`（Phase E 第二批 & 第三批：026/027/015-018）
>
> 已合并的相关 PR：[#137](https://github.com/LuuuXXX/cpp2rust-demo/pull/137)、
> [#139](https://github.com/LuuuXXX/cpp2rust-demo/pull/139)–
> [#149](https://github.com/LuuuXXX/cpp2rust-demo/pull/149)。

---

## 1. 开发目标

### Phase E 第五批（STL 容器示例）

在 Phase E 第一批（024/025）、第二批（026/027）、第三批（015-018）、第四批（023）完成的基础上，
将 v6 方案 §3.2 中余下的 **STL 容器示例**（034–038）按相同的「`lib.rs` + `main.rs` + `tests/smoke.rs`」
结构完成迁移：

- **034_vector_basic**：`std::vector<T>` 薄 wrapper（`IntVector` / `StringVector`）
- **035_map_basic**：`std::map<K,V>` 薄 wrapper（`StringIntMap` / `IntStringMap`）
- **036_string_basic**：`std::string` 薄 wrapper（`String`）
- **037_array_basic**：`std::array<T,N>` 薄 wrapper（`IntArray5` / `DoubleArray3` / `StringArray4`）
- **038_tuple_basic**：`std::tuple<T...>` 薄 wrapper（`Tuple2` / `Tuple3` / `Tuple4`）

每个示例迁移目标：
1. 新增 `src/lib.rs`（hicc 三段 FFI 绑定；类方法与工厂函数加 `pub` 修饰，供集成测试访问）
2. 改造 `src/main.rs`（移除 hicc 块，添加 `use <crate>::*;`）
3. 新增 `tests/smoke.rs`（行为断言集成测试）
4. 更新 `Cargo.toml`（添加 `[lib]` + `[[bin]]` 目标声明）
5. 更新 `build.rs`（`rust_file("src/main.rs")` → `rust_file("src/lib.rs")`）
6. 更新 `tests/l1_golden_tests.rs`（`golden_test!` → `golden_test_lib!`）

### Phase F（CI 新增 L_smoke job）

在现有 L1–L5 基础上，新增 **`l-smoke`** job：
- 对包含 `tests/smoke.rs` 的 14 个已迁移示例（015-018、023-027、034-038）运行 `cargo test --test smoke`
- 依赖 `l2-compile`，与 `l3-run` 并行执行

### Phase G（文档收尾）

- 更新 `README.md` 特性矩阵中 034-038 的说明，标注已有 `lib.rs + tests/smoke.rs` 结构与冒烟测试
- 更新 `README.md` L_smoke 冒烟测试命令列表，补充 034-038
- 新增本进展文档

**硬约束（来自 v6 方案 §1.3 / §9）**：
- `init` / `merge` 命令、参数、`.cpp2rust/<feature>/` 目录结构完全不变
- 265 个 lib 单测全部通过，默认产物不受影响

---

## 2. 详细方案

### 2.1 文件改造模式（与前序批次一致）

与 Phase E 前几批相同，沿用「lib.rs + main.rs」结构迁移策略：

| 文件 | 改造内容 |
|------|---------|
| `src/lib.rs`（新增） | 将原 `main.rs` 的 hicc 三段块迁入；`import_class!` 内方法加 `pub fn`；`import_lib!` 工厂函数加 `pub fn` / `pub unsafe fn` |
| `src/main.rs`（改造） | 移除 hicc 块，添加 `use <crate>::*;`；保留完整 `fn main()` 演示逻辑 |
| `tests/smoke.rs`（新增） | 行为断言集成测试，通过 `use <crate>::*;` 调用 `lib.rs` 公有绑定 |
| `Cargo.toml`（更新） | 添加 `[lib]` + `[[bin]]` 显式目标声明 |
| `build.rs`（更新） | `rust_file("src/main.rs")` → `rust_file("src/lib.rs")`；`rerun-if-changed` 同步更新 |

### 2.2 特殊处理

- **035_map_basic**：`insert`/`get`/`set`/`erase` 方法参数含 `*const i8`（C 字符串），smoke
  测试使用 `CString::new(...).unwrap()` 绑定生命周期，避免悬空指针。
- **036_string_basic**：`string_new_from` / `string_new_from_len` 已标记为 `unsafe fn`（含
  原始指针参数），smoke 测试在 `unsafe {}` 块内调用并通过 `CStr::from_ptr` 验证字符串内容。
- **038_tuple_basic**：工厂函数含 `*const i8` 参数，均标记 `pub unsafe fn`；smoke 测试同上
  在 `unsafe {}` 块内调用。

### 2.3 L1 黄金测试适配

与前几批一致：将 `golden_test!(test_034_vector_basic, ...)` 等改为 `golden_test_lib!(...)` 宏，
从 `lib.rs` 读取黄金内容，经 `strip_pub_visibility` 规范化后与工具输出比较。

### 2.4 L_smoke CI job

新增 `l-smoke` job（`ubuntu-latest`），在 `l2-compile` 通过后对下列 14 个示例依次执行
`cargo test --manifest-path ... --test smoke`：

```
015_virtual_basic, 016_virtual_pure, 017_virtual_override, 018_virtual_diamond,
023_typeid_rtti, 024_template_function, 025_template_class,
026_template_specialization, 027_template_instantiation,
034_vector_basic, 035_map_basic, 036_string_basic, 037_array_basic, 038_tuple_basic
```

使用共享 `CARGO_TARGET_DIR`（`examples-target`）缓存编译产物，加速构建。

---

## 3. 详细进展

### 3.1 Phase E 第五批（034-038 STL 示例迁移）

| 示例 | 文件 | 状态 |
|------|------|------|
| 034_vector_basic | `src/lib.rs` | ✅ 新增（IntVector/StringVector pub class + pub 方法 + pub 工厂） |
| 034_vector_basic | `src/main.rs` | ✅ 改造（use vector_basic::*） |
| 034_vector_basic | `tests/smoke.rs` | ✅ 新增（5 个行为断言测试） |
| 034_vector_basic | `Cargo.toml` | ✅ 更新（[lib] + [[bin]]） |
| 034_vector_basic | `build.rs` | ✅ 更新（lib.rs） |
| 035_map_basic | `src/lib.rs` | ✅ 新增（StringIntMap/IntStringMap pub class + pub 方法 + pub 工厂） |
| 035_map_basic | `src/main.rs` | ✅ 改造（use map_basic::*） |
| 035_map_basic | `tests/smoke.rs` | ✅ 新增（6 个行为断言测试） |
| 035_map_basic | `Cargo.toml` | ✅ 更新（[lib] + [[bin]]） |
| 035_map_basic | `build.rs` | ✅ 更新（lib.rs） |
| 036_string_basic | `src/lib.rs` | ✅ 新增（String pub class + pub 方法 + pub unsafe 工厂） |
| 036_string_basic | `src/main.rs` | ✅ 改造（use string_basic::*） |
| 036_string_basic | `tests/smoke.rs` | ✅ 新增（5 个行为断言测试，含 CStr 验证） |
| 036_string_basic | `Cargo.toml` | ✅ 更新（[lib] + [[bin]]） |
| 036_string_basic | `build.rs` | ✅ 更新（lib.rs） |
| 037_array_basic | `src/lib.rs` | ✅ 新增（IntArray5/DoubleArray3/StringArray4 pub class + pub 方法 + pub 工厂） |
| 037_array_basic | `src/main.rs` | ✅ 改造（use array_basic::*） |
| 037_array_basic | `tests/smoke.rs` | ✅ 新增（6 个行为断言测试） |
| 037_array_basic | `Cargo.toml` | ✅ 更新（[lib] + [[bin]]） |
| 037_array_basic | `build.rs` | ✅ 更新（lib.rs） |
| 038_tuple_basic | `src/lib.rs` | ✅ 新增（Tuple2/3/4 pub class + pub 方法 + pub unsafe 工厂） |
| 038_tuple_basic | `src/main.rs` | ✅ 改造（use tuple_basic::*） |
| 038_tuple_basic | `tests/smoke.rs` | ✅ 新增（3 个行为断言测试） |
| 038_tuple_basic | `Cargo.toml` | ✅ 更新（[lib] + [[bin]]） |
| 038_tuple_basic | `build.rs` | ✅ 更新（lib.rs） |

### 3.2 L1 黄金测试更新

| 示例 | 变更 | 状态 |
|------|------|------|
| 034_vector_basic | `golden_test!` → `golden_test_lib!` | ✅ |
| 035_map_basic | `golden_test!` → `golden_test_lib!` | ✅ |
| 036_string_basic | `golden_test!` → `golden_test_lib!` | ✅ |
| 037_array_basic | `golden_test!` → `golden_test_lib!` | ✅ |
| 038_tuple_basic | `golden_test!` → `golden_test_lib!` | ✅ |

### 3.3 Phase F（CI L_smoke job）

| 变更 | 状态 |
|------|------|
| 新增 `l-smoke` job（ubuntu-latest） | ✅ 已完成 |
| 覆盖 14 个迁移示例（015-018、023-027、034-038） | ✅ |
| 依赖 `l2-compile`，与 `l3-run` 并行 | ✅ |

### 3.4 Phase G（文档收尾）

| 文档 | 变更 | 状态 |
|------|------|------|
| `README.md` | 特性矩阵中 034-038 标注已有 `lib.rs + tests/smoke.rs` 与冒烟测试 | ✅ |
| `README.md` | L_smoke 命令列表补充 034-038 | ✅ |
| `docs/plans/v6/development-progress-phase-e-batch5.md` | 本进展文档 | ✅ |

### 3.5 回归验证

| 验证项 | 状态 | 说明 |
|--------|------|------|
| `cargo test --lib`（265 个 lib 单测） | ✅ 通过 | 全部通过，默认产物不变 |
| `cargo check` 034-038（所有编译目标） | ✅ 通过 | lib + bin 双目标均无错误 |

**回归验证结论**：265 个 lib 单测在本次改动后全部通过，确认默认产物未受影响。
034-038 的 `lib.rs`（新增）与改造后的 `main.rs` 均编译无错误，结构迁移完全向后兼容。

---

## 4. v6 完整进展汇总

至此，v6 方案中的主要阶段均已完成：

| 阶段 | 状态 | 说明 |
|------|------|------|
| Phase A（AST 提取） | ✅ 完成 | `ClassTemplate` / `FunctionTemplate` 结构化提取 |
| Phase B（泛型骨架） | ✅ 完成 | `import_class!` 泛型 + `import_lib!` 模板函数骨架 |
| Phase B 增强（实例化别名） | ✅ 完成 | 类型别名 + 构造工厂骨架，5 种来源追踪 |
| Phase C（高级映射） | ✅ 完成 | `@make_proxy`（代理工厂）、`@dynamic_cast`（多层转换 + 引用形式） |
| Phase D（冒烟测试生成） | ✅ 完成 | `smoke_test_gen.rs`，幂等生成 `tests/smoke.rs`，受 `CPP2RUST_GEN_SMOKE` 控制 |
| Phase E（示例改造） | ✅ 完成 | 15 个示例（015-018、023-027、034-038）迁移为 `lib.rs + main.rs + smoke.rs` |
| Phase F（CI smoke job） | ✅ 完成 | `l-smoke` job 覆盖 14 个迁移示例，接入 CI 门禁 |
| Phase G（文档） | ✅ 完成 | README、特性矩阵、各阶段进展文档全部更新 |

## 5. 后续建议（可选改进方向）

以下为 v6 完成后可考虑的后续优化方向，不属于当前 PR 范畴：

- **Phase F 扩展**：为 macOS 平台添加 `l-smoke-macos` job（虚函数相关示例需平台跳过策略）
- **Phase F 扩展**：`gen-verify` 端到端 job（`init` → 生成目录 `cargo test`），验证**工具实际输出**
- **Phase G 扩展**：`docs/INTRODUCTION.md` 补充模板类 / 模板函数 / 接口映射的完整章节
- **示例 README**：各迁移示例的 README 补充「冒烟测试」说明段落
- **STL 进阶**：基于 hicc-std 的泛型 class 别名（`hicc_std::vector` 等），替代当前薄 wrapper 类
