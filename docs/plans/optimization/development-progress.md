# 优化方案开发进展记录

> 跟踪 `optimization-plan.md` 各阶段的落地进展。每个阶段落地后在此追加条目。
> 全程使用简体中文。

---

## 进展表

| 阶段 | 内容 | 状态 | 落地 | 备注 |
|------|------|------|------|------|
| A | 仓库瘦身与子模块化 | ✅ 已完成 | PR #157 | c2rust-demo / hicc-usages 子模块化；移除约 4M vendored + 2 个 `.rar`；`docs/plans/v5` → `docs/archive/v5` |
| E | 文档与导航 | ✅ 已完成 | PR #157 | README L0/L1 分级 + hicc-usages 对账；DEVELOPMENT 示例矩阵关联标准答案集 |
| —  | 优化方案文档沉淀 | ✅ 已完成 | 本次 | 新增 `docs/plans/optimization/`（方案 + 进展），README 文档导航引用 |
| B | L0/L1 shim 分级 + 重生成 L1 golden | ⬜ 待开始 | — | 高风险，需跨平台 golden 回归 |
| C | 导出符号冒烟测试全覆盖 | ⬜ 待开始 | — | 需各库可链接产物 |
| D | 每库独立 CI + 扩充真实项目矩阵 | ⬜ 待开始 | — | 依赖 CI 基础设施 |
| A 收尾 | `references/hicc` 子模块化、`examples/` 去重 | ⬜ 阻塞中 | — | gitcode.com 网络受限 |

状态图例：⬜ 待开始　🚧 进行中　✅ 已完成

---

## 阶段 A / E（PR #157）

落地优化方案中**最低风险、可本地验证**的首个增量（仓库瘦身 + 文档），
不触碰 FFI 生成逻辑与测试依赖项，确保现有 CI 行为不变。

- 阶段 A：`references/c2rust-demo` 移除 vendored 副本及 `.rar` 二进制改为子模块；
  新增 `references/hicc-usages` 子模块；`references/hicc` 因 gitcode 网络受限暂保留 vendored
  并在 `.gitmodules` 注释记录预期来源；`docs/plans/v5` 归档至 `docs/archive/v5`。
- 阶段 E：README 强化为 L0/L1 shim 分级并引用 hicc-usages 表1/表2；特性矩阵与仓库结构更新；
  DEVELOPMENT 示例矩阵关联标准答案集。

影响评估：无 Rust 源码改动；CI 各 job 不自动拉取子模块，新增子模块不改变现有 CI 行为。

---

## 优化方案文档沉淀（本次）

此前优化方案仅存在于 PR #157 描述中，仓库内无统一跟踪入口。本次将完整方案沉淀为
`docs/plans/optimization/optimization-plan.md`（阶段 A–E 路线图、硬约束、决策点）与本进展文档，
并在 README 文档导航处引用，便于后续阶段 B/C/D 按既定排序推进与跟踪。
