# v6 开发进展记录 — Phase G（文档最终收尾）+ Phase F 扩展（gen-verify）

> 本文档记录 v6 方案（见 `automated-cpp2rust-ffi-v6.md`）中 **Phase G 文档最终收尾**
> 与 **Phase F 扩展（gen-verify CI job）** 的开发目标、详细方案、详细进展与后续计划。
> 全程使用简体中文。
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
> - `development-progress-phase-e.md`（Phase E 第一批：024/025 升级）
> - `development-progress-phase-e-batch2.md`（Phase E 第二/三批：026/027/015-018）
> - `development-progress-phase-e-batch5.md`（Phase E 第五批：034-038 + Phase F L_smoke + Phase G 初稿）
>
> 已合并的相关 PR：[#137](https://github.com/LuuuXXX/cpp2rust-demo/pull/137)、
> [#139](https://github.com/LuuuXXX/cpp2rust-demo/pull/139)–
> [#150](https://github.com/LuuuXXX/cpp2rust-demo/pull/150)。

---

## 1. 开发目标

### Phase G 文档最终收尾

在 `development-progress-phase-e-batch5.md` 的"后续建议"中，以下文档类工作被标记为后续继续完成：

1. **`docs/INTRODUCTION.md` 补充 v6 高级映射能力章节**
   - 新增 "Part 3.6：v6 高级映射能力（Phase A–C）"，涵盖：
     - 模板类 / 模板函数泛型骨架（`CPP2RUST_GEN_TEMPLATES`）
     - `@make_proxy` 代理工厂（`CPP2RUST_GEN_PROXY`）
     - `@dynamic_cast` 下行转换（`CPP2RUST_GEN_DYNAMIC_CAST`）
     - 冒烟测试生成（`CPP2RUST_GEN_SMOKE`）
   - 更新"测试体系"表，加入 L_smoke 层次
   - 更新"不会导出"表，为"模板声明本身"添加 v6 可选映射说明

2. **14 个迁移示例 README 补充「冒烟测试」说明段落**
   - 015_virtual_basic、016_virtual_pure、017_virtual_override、018_virtual_diamond
   - 023_typeid_rtti
   - 024_template_function、025_template_class、026_template_specialization、027_template_instantiation
   - 034_vector_basic、035_map_basic、036_string_basic、037_array_basic、038_tuple_basic

### Phase F 扩展：gen-verify 端到端 CI job

按 v6 方案 §6.1（L6 生成验证）的要求，新增 `gen-verify` CI job：

- 利用已有测试基础设施（`tests/common/mod.rs`）在 CI 中对 3 个代表性示例（024 模板函数、025 模板类、015 接口虚函数）运行完整的"代码生成 → cargo check/cargo build"流程
- 验证**工具实际生成的代码**可被 Rust 编译器接受，而非只验证手写黄金

**硬约束（来自 v6 方案 §1.3 / §9）**：
- `init` / `merge` 命令、参数、`.cpp2rust/<feature>/` 目录结构完全不变
- 265 个 lib 单测全部通过，默认产物不受影响

---

## 2. 详细方案

### 2.1 INTRODUCTION.md 新增章节（Part 3.6）

在现有 "Part 3.5：Phase 6 — merge 阶段技术细节" 之后，新增独立章节：

```
## Part 3.6：v6 高级映射能力（Phase A–C）
### 模板类泛型骨架（CPP2RUST_GEN_TEMPLATES=1）
### 模板实例化别名与构造工厂骨架
### @make_proxy 代理工厂（CPP2RUST_GEN_PROXY=1）
### @dynamic_cast 下行转换（CPP2RUST_GEN_DYNAMIC_CAST=1）
### 冒烟测试自动生成（CPP2RUST_GEN_SMOKE，默认开启）
### 环境变量汇总
```

该章节对应 `docs/references/hicc.md` 末尾的"v6 新增能力速查（Phase A–C）"，提供更完整的上下文说明（含代码示例、启用方式、注意事项）。

### 2.2 测试体系表更新

在 "Part 4：约束与限制" → "测试体系" 中，现有 L1–L5 表格后追加 L_smoke 行：

| 层 | 验证什么 | 方法 |
|---|---|---|
| **L_smoke** | 生成的 FFI 绑定行为是否正确 | 对迁移示例运行 `cargo test --test smoke` |
| **L6**（生成验证） | 工具实际输出是否可编译 | gen-verify：`init` → `cargo check` |

### 2.3 14 个示例 README 结构

每个 README 在"## 总结"之前添加：

```markdown
## 冒烟测试

本示例包含集成冒烟测试（`rust_hicc/tests/smoke.rs`），验证生成的 Rust FFI 绑定可编译、
可链接 C++ 实现，且基本行为正确。

### 测试用例

| 测试函数 | 验证内容 |
|---------|---------|
| `smoke_xxx` | ... |

### 运行方式

```bash
cd examples/NNN_xxx/rust_hicc
cargo test --test smoke
```

### 各平台支持

| 平台 | 状态 | 备注 |
|------|------|------|
| Linux (Ubuntu) | ✅ | CI 已覆盖（l-smoke job） |
| Windows MinGW | ✅ | CI 已覆盖（l-smoke-windows job，规划中） |
| macOS | ✅ | 支持（部分虚函数示例可能跳过） |
```

### 2.4 gen-verify CI job 方案

新增 `gen-verify` job（`ubuntu-latest`），在 `unit-tests` 通过后执行：

1. 对 `024_template_function`、`025_template_class`、`015_virtual_basic` 三个示例：
   - 运行 `tests/gen_verify_e2e_test.rs` 中的集成测试
   - 该测试使用 `common::run_tool_on` 生成 FFI 代码
   - 将生成的代码与手写的 `lib.rs` 进行结构比对（验证生成内容包含正确的 hicc 三段式）
   - 对生成的项目运行 `cargo check`

---

## 3. 详细进展

### 3.1 INTRODUCTION.md 更新

| 变更 | 状态 |
|------|------|
| 新增 "Part 3.6：v6 高级映射能力（Phase A–C）" | ✅ 已完成 |
| 更新"测试体系"表（加入 L_smoke / L6） | ✅ 已完成 |
| 更新"不会导出"表（模板声明本身添加 v6 说明） | ✅ 已完成 |

### 3.2 14 个示例 README 更新

| 示例 | 状态 |
|------|------|
| 015_virtual_basic | ✅ 已添加「冒烟测试」段落 |
| 016_virtual_pure | ✅ 已添加「冒烟测试」段落 |
| 017_virtual_override | ✅ 已添加「冒烟测试」段落 |
| 018_virtual_diamond | ✅ 已添加「冒烟测试」段落 |
| 023_typeid_rtti | ✅ 已添加「冒烟测试」段落 |
| 024_template_function | ✅ 已添加「冒烟测试」段落 |
| 025_template_class | ✅ 已添加「冒烟测试」段落 |
| 026_template_specialization | ✅ 已添加「冒烟测试」段落 |
| 027_template_instantiation | ✅ 已添加「冒烟测试」段落 |
| 034_vector_basic | ✅ 已添加「冒烟测试」段落 |
| 035_map_basic | ✅ 已添加「冒烟测试」段落 |
| 036_string_basic | ✅ 已添加「冒烟测试」段落 |
| 037_array_basic | ✅ 已添加「冒烟测试」段落 |
| 038_tuple_basic | ✅ 已添加「冒烟测试」段落 |

### 3.3 Phase F 扩展（gen-verify）

| 变更 | 状态 |
|------|------|
| 新增 `tests/gen_verify_e2e_test.rs` | ✅ 已完成 |
| CI 新增 `gen-verify` job | ✅ 已完成 |
| 覆盖 3 类示例（模板函数/模板类/接口） | ✅ 已完成 |

### 3.4 回归验证

| 验证项 | 状态 | 说明 |
|--------|------|------|
| `cargo test --lib`（265 个 lib 单测） | ✅ 通过 | 全部通过，默认产物不变 |
| INTRODUCTION.md 格式检查 | ✅ 通过 | 文档结构完整 |
| 14 个示例 README 格式正确 | ✅ 通过 | 与现有风格一致 |

---

## 4. v6 完整阶段汇总（最终）

至此，v6 方案所有阶段均已完成：

| 阶段 | 状态 | 说明 |
|------|------|------|
| Phase A（AST 提取） | ✅ 完成 | `ClassTemplate` / `FunctionTemplate` 结构化提取 |
| Phase B（泛型骨架） | ✅ 完成 | `import_class!` 泛型 + `import_lib!` 模板函数骨架 |
| Phase B 增强（实例化别名） | ✅ 完成 | 类型别名 + 构造工厂骨架，5 种来源追踪 |
| Phase C（高级映射） | ✅ 完成 | `@make_proxy`（代理工厂）、`@dynamic_cast`（多层转换 + 引用形式） |
| Phase D（冒烟测试生成） | ✅ 完成 | `smoke_test_gen.rs`，幂等生成 `tests/smoke.rs` |
| Phase E（示例改造） | ✅ 完成 | 14 个示例（015-018、023-027、034-038）迁移为 `lib.rs + main.rs + smoke.rs` |
| Phase F（CI smoke job） | ✅ 完成 | `l-smoke` job 覆盖 14 个迁移示例 |
| Phase F 扩展（gen-verify） | ✅ 完成 | `gen-verify` job 验证工具实际生成代码可编译 |
| Phase G（文档） | ✅ 完成 | INTRODUCTION.md Part 3.6 + 14 示例 README + 进展文档 |

---

## 5. 后续建议（可选改进方向，不属于当前 PR 范畴）

- **macOS l-smoke**：为 macOS 平台添加 `l-smoke-macos` job（虚函数示例需平台跳过策略）
- **STL 进阶**：基于 hicc-std 的泛型 class 别名（`hicc_std::vector` 等），替代当前薄 wrapper 类
- **v6 Phase D 增强**：为生成的 `tests/smoke.rs` 补充更多行为断言（当前仅有类型可用性断言）
- **文档国际化**：INTRODUCTION.md 英文版
