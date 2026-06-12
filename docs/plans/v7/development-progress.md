# v7 开发进展记录 — 总览

> 本文档跟踪 v7 方案（见 `automated-cpp2rust-ffi-v7.md`）的**目标、阶段、进展与后续计划**。
> 全程使用简体中文。每个阶段落地后可追加独立的 `development-progress-phase-*.md` 详述。

---

## 1. 开发目标（一句话）

把 v6 中由环境变量开关（`CPP2RUST_GEN_TEMPLATES` / `CPP2RUST_GEN_PROXY` /
`CPP2RUST_GEN_DYNAMIC_CAST` / `CPP2RUST_GEN_SMOKE`）控制、默认关闭的高级能力
**毕业为默认行为**，删除全部开关，并补齐覆盖全部特性的行为级冒烟测试——
在不改变 `init` + `merge` 使用方式的前提下，让默认产物即覆盖全特性、符合 hicc 与 Rust 最佳实践。

---

## 2. 硬约束

1. `init` / `merge` 命令、参数、`.cpp2rust/<feature>/` 目录结构完全不变。
2. 不新增、不依赖任何环境变量开关来影响生成结果。
3. 代码库中不再保留 `CPP2RUST_GEN_*` 字样。
4. 冒烟测试默认生成、覆盖全部特性、行为级断言、符合 Rust 测试最佳实践。

---

## 3. 阶段进展表

| 阶段 | 内容 | 状态 | 备注 |
|------|------|------|------|
| Phase 1 | 移除 `CPP2RUST_GEN_SMOKE`；冒烟默认幂等生成 | ✅ 已完成 | 删除 `smoke_enabled()`/`GEN_SMOKE_ENV` 及单测；`init` 无条件生成（文件级幂等） |
| Phase 2 | 移除 `CPP2RUST_GEN_DYNAMIC_CAST`；`@dynamic_cast` 默认输出 | ✅ 已完成 | 活动绑定；无示例触发，默认产物不变 |
| Phase 3 | 移除 `CPP2RUST_GEN_PROXY`；`@make_proxy` 默认输出 | ✅ 已完成 | 活动绑定；无示例触发，默认产物不变 |
| Phase 4 | 移除 `CPP2RUST_GEN_TEMPLATES`；模板骨架/别名/工厂默认输出 + 重做黄金 | ✅ 已完成 | 模板骨架改**注释形式**（保证可编译，过 L6 gen-verify）；024 黄金 `lib_scaffold.rs` 更新 |
| Phase 5 | 冒烟全特性覆盖：48/48 示例具备行为级 `tests/smoke.rs` | ✅ 已完成 | 34 个示例迁移为 lib.rs+main.rs+smoke.rs 结构；详见 `development-progress-phase-5.md` |
| Phase 6 | 代码去冗余：删除 `env_switch_enabled` 等遗留、emit_* 去 `enabled` 形参、清理注释 | ✅ 已完成 | 生成路径收敛为「IR 非空即输出」单路径 |
| Phase 7 | CI：l-smoke 扩全量、gen-verify 覆盖三类高级能力、门禁校验无 `CPP2RUST_GEN_*` | ✅ 已完成 | l-smoke 自动发现全部 smoke.rs；添加 CPP2RUST_GEN_* 门禁 |
| Phase 8 | 文档：INTRODUCTION / hicc.md / README / CHANGELOG / DEVELOPMENT 对齐 v7 | ✅ 已完成 | 全部移除开关表述，改「默认生成」 |

状态图例：⬜ 待开始　🚧 进行中　✅ 已完成

> **设计要点（实现期补充）**：模板类 / 模板函数 / 实例化别名 / 构造工厂因「未实例化的模板没有
> 可链接符号、泛型 `<T>` 不可直接编译」，默认以**注释骨架**形式输出（带 `cpp2rust-todo[TMPL]`
> 指引），既保证工具默认产物可通过 L6 gen-verify 编译，又指引用户补全后取消注释；而
> `@make_proxy` / `@dynamic_cast` 使用 hicc 内建指令、对接具体类型，默认输出为可编译的活动绑定。
> 实测开启全部能力后，48 个示例中仅 024_template_function 的默认产物发生变化（其余示例的
> proxy/dynamic_cast/template IR 为空），故黄金重做范围极小。

---

## 4. 受影响的关键文件（预估）

- 生成器：`src/generator/hicc_codegen.rs`、`src/generator/smoke_test_gen.rs`、`src/generator/project_generator.rs`
- 命令：`src/commands/init.rs`
- IR / 提取器：`src/ffi_model.rs`、`src/extractor/{template_spec,proxy_spec,dynamic_cast_spec,mod}.rs`、`src/ast_parser/{mod,collector}.rs`
- 测试：`tests/{template_gen_tests,proxy_gen_tests,dynamic_cast_gen_tests,l1_golden_tests,gen_verify_e2e_test}.rs`、各 `examples/*/rust_hicc/tests/smoke.rs`
- CI：`.github/workflows/ci.yml`
- 文档：`docs/INTRODUCTION.md`、`docs/references/hicc.md`、`README.md`、`CHANGELOG.md`、`DEVELOPMENT.md`

---

## 5. 质量验证

v7 全部 8 个阶段完成后，进行了深度质量验证（2026-06-12），详见 `quality-verification-report.md`。

### 验证结论

- **48/48** 示例全部具备行为级冒烟测试（219 个测试、401+ 个断言）
- **266** 个 lib 单元测试全部通过
- **0** 个 `CPP2RUST_GEN_*` 残留
- CI master 全绿

### 质量验证修复

| 示例 | 修复内容 |
|------|---------|
| 012_class_volatile | 新增 8 个精确值行为断言（status_reg / data_reg） |
| 014_inheritance_multiple | 新增 `smoke_derived_base2_value` 测试 |
| 015_virtual_basic | 新增 Shape::area() 精确断言 |
| 017_virtual_override | 新增 area() 精确断言 + 多态测试 |

## 6. 后续计划

v7 全部 8 个阶段已完成。质量验证报告见 `quality-verification-report.md` §6。

### 完成总结

| 阶段 | 内容 | 状态 | 完成日期 |
|------|------|------|---------|
| Phase 1 | 移除 `CPP2RUST_GEN_SMOKE`；冒烟默认幂等生成 | ✅ | PR #152 前 |
| Phase 2 | 移除 `CPP2RUST_GEN_DYNAMIC_CAST`；`@dynamic_cast` 默认输出 | ✅ | PR #152 前 |
| Phase 3 | 移除 `CPP2RUST_GEN_PROXY`；`@make_proxy` 默认输出 | ✅ | PR #152 前 |
| Phase 4 | 移除 `CPP2RUST_GEN_TEMPLATES`；模板骨架默认输出 | ✅ | PR #152 前 |
| Phase 5 | 冒烟全特性覆盖：48/48 示例具备行为级 `tests/smoke.rs` | ✅ | 2026-06-12 |
| Phase 6 | 代码去冗余：删除 `env_switch_enabled` 等遗留 | ✅ | PR #152 前 |
| Phase 7 | CI：l-smoke 扩全量、gen-verify、门禁校验 | ✅ | 2026-06-12 |
| Phase 8 | 文档：INTRODUCTION / hicc.md / README / CHANGELOG / DEVELOPMENT 对齐 v7 | ✅ | PR #152 前 |
| 质量验证 | 深度审计 48 示例，修复 4 个冒烟测试质量问题 | ✅ | 2026-06-12 |

详细进展见 `development-progress-phase-5.md` 和 `quality-verification-report.md`。
