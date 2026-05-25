# Feature 级开发设计文档

## 报告基本信息

| 字段 | 内容 |
|------|------|
| feature ID | `FEAT-005` |
| feature 名称 | `pointer-path` |
| 所属 component | `rapidjson-rs` (`COMP-001`) |
| 所属 crate | `rapidjson-rs` |
| 文档版本 | `0.1.0-draft` |
| 设计日期 | `2026-05-14` |
| 架构师 | `TBD` |
| 文档状态 | `草稿` |
| component 设计追溯 | [`rapidjson-rs-dev-design.md` 第 3.6、4.1 pointer 模块](../rapidjson-rs-dev-design.md#4-module-划分) |
| 是否商用代码 | `是` |
| 依赖使用策略 | `core/std-only`（禁止 crates.io 第三方依赖） |

---

## 快速导航

- [1. feature 概述](#1-feature-概述)
- [2. 功能需求](#2-功能需求)
- [3. 接口设计](#3-接口设计)
- [4. 数据结构](#4-数据结构)
- [5. 实现要点](#5-实现要点)
- [变更历史](#变更历史)

---

## 1. feature 概述

### 1.0 商用代码与依赖使用约束

| 字段 | 内容 |
|------|------|
| 是否商用代码 | `是` |
| 允许依赖范围 | `core/std-only`（禁止 crates.io 第三方依赖） |
| crates.io 使用策略 | Pointer 实现不得依赖第三方路径解析/正则/JSON 库，所有路径解析与操作逻辑由本 feature 内自行实现。 |
| 对当前 feature 技术选型的影响 | JSON Pointer 实现需直接基于 DOM 与内部字符串/数组 API，不使用额外第三方工具；错误表示依赖 core-infra 的错误类型。 |

### 1.1 feature 职责

**一句话描述:**
- `pointer-path` 实现 RFC 6901 JSON Pointer 功能，为 DOM 提供基于路径的访问/修改/创建/删除/交换等操作能力，以及 Pointer/URI Fragment 的解析与序列化。

**详细职责:**
- 解析 JSON Pointer 字符串为内部路径表示，支持 `~0`/`~1` 转义等规则；
- 根据 Pointer 路径在 DOM 中获取目标值（Get）或在不存在时返回错误；
- 根据 Pointer 路径在 DOM 中设置目标值（Set），必要时创建中间节点；
- 根据 Pointer 路径删除目标值（Erase）；
- 根据 Pointer 路径交换值（Swap）；
- 支持 URI Fragment（`#` 前缀 + 百分号编码）形式 Pointer 的解析与序列化；
- 提供 Pointer 字符串化与结构化表示之间的转换。

**不在职责范围内:**
- 不直接参与 JSON 文本解析/生成，仅操作 DOM 结构；
- 不实现 Schema 校验逻辑，虽然 Schema 可能使用 Pointer 作为内部引用机制，但验证逻辑属于 schema feature。

### 1.2 项目中的位置

- 所属 crate：`rapidjson-rs`。
- 预期 module 路径：`rapidjson_rs::pointer`。

### 1.3 重构策略

**重构策略:** `完全 Rust 化 + 行为对齐 C++`。

- 按照 `pointer.h` 中的接口和 `pointertest.cpp` 用例重建 Pointer 功能；
- 内部路径结构尽量保持简洁（如使用 `Vec<String>` 或轻量 token 结构），同时确保行为与 C++ 实现一致；
- 优先实现基本指向、创建/删除、错误报告，再扩展到 URI Fragment 和复杂场景。

**C/C++ 代码对应部分:**

| 源文件 | 范围 | 复杂度 |
|-------|-----|-------|
| `rapidjson_legacy/include/rapidjson/pointer.h` | JSON Pointer 类型与操作接口（Get/Set/Create/Swap/Erase 等） | 高 |

**相关测试：**
- `rapidjson_legacy/test/unittest/pointertest.cpp`。

---

## 2. 功能需求

### 2.1 功能清单

| 需求 ID | 功能名称 | 优先级 | 复杂度 | component 开发设计文档追溯 | 状态 |
|---------|----------|--------|--------|-----------------------------|------|
| `PTR-REQ-PARSE-001` | Pointer 字符串解析 | P0 | 中 | [需求文档 2.5 REQ-JP-01/07/08/09](../../requirements/requirements.md#2-5-json-pointerrfc-6901) | 待开发 |
| `PTR-REQ-GET-001` | Pointer Get 操作 | P0 | 中 | [REQ-JP-01/04](../../requirements/requirements.md#2-5-json-pointerrfc-6901) | 待开发 |
| `PTR-REQ-SET-001` | Pointer Set/Create 操作 | P0 | 高 | [REQ-JP-02/03](../../requirements/requirements.md#2-5-json-pointerrfc-6901) | 待开发 |
| `PTR-REQ-SWAP-001` | Pointer Swap 操作 | P1 | 中 | [REQ-JP-05](../../requirements/requirements.md#2-5-json-pointerrfc-6901) | 待开发 |
| `PTR-REQ-ERASE-001` | Pointer Erase 操作 | P1 | 中 | [REQ-JP-06](../../requirements/requirements.md#2-5-json-pointerrfc-6901) | 待开发 |
| `PTR-REQ-ERROR-001` | Pointer 错误报告 | P0 | 中 | [REQ-JP-10](../../requirements/requirements.md#2-5-json-pointerrfc-6901) | 待开发 |

### 2.2 功能详细规格

#### PTR-REQ-PARSE-001：Pointer 字符串解析

**输入:**
- Pointer 字符串，如 `/foo/0/bar~1baz` 或 URI Fragment 形式 `#/foo/0` 等。

**输出:**
- 内部路径表示（例如 token 列表），每个 token 表示对象 key 或数组下标。内部结构本设计中仅描述为“token 序列”，具体类型在实现阶段确定。

**处理逻辑要点:**
1. 按 RFC 6901 规则解析：
   - 以 `/` 分割 token；
   - 处理 `~0` → `~`、`~1` → `/` 转义；
   - 对 URI Fragment 形式，去掉前导 `#` 并进行必要的百分号解码。
2. 禁止空 token 产生不合法路径（除根路径特殊情况外）。

**异常场景:**
- 遇到非法转义或无效 URI 编码时，返回 Pointer 特定错误，包含错误位置。

#### PTR-REQ-GET-001：Pointer Get 操作

**输入:**
- DOM 根节点（`Document` 或 `Value` 引用）与解析后的 Pointer 路径。

**输出:**
- 目标 `Value` 的引用（只读或可变，具体接口在实现阶段细化），或错误（路径不存在/类型不匹配等）。

**处理逻辑要点:**
1. 按 token 顺序遍历 DOM：
   - 对对象，按 key 查找成员；
   - 对数组，将 token 解析为 index 并访问元素；
2. 如任一中间节点不存在或类型不匹配，返回错误。

#### PTR-REQ-SET-001：Pointer Set/Create 操作

**输入:**
- DOM 根节点、Pointer 路径、要设置的值；
  - 对于 Create，允许中间节点不存在；

**输出:**
- 修改后的 DOM；
  - 可能返回旧值或状态指示（参考 C++ 行为）。

**处理逻辑要点:**
1. 对于 create 模式：在路径不存在时自动创建中间对象/数组；
2. 对于 set 模式：当目标存在时替换值，不存在时行为需与 C++ 实现对齐（一般视为错误或 create）。

#### PTR-REQ-SWAP-001：Pointer Swap 操作

**输入/输出:**
- 两个 Pointer 路径与 DOM；交换两个位置的值。

**处理逻辑要点:**
1. 两个路径都必须有效；
2. 使用 DOM 提供的 swap 接口，无需额外复制。

#### PTR-REQ-ERASE-001：Pointer Erase 操作

**输入:**
- DOM 与 Pointer 路径；

**输出:**
- 删除目标节点后的 DOM 状态。

**处理逻辑要点:**
1. 对象中删除成员，数组中删除指定位置元素；
2. 不自动清理空上层结构（行为与 C++ 保持一致）。

#### PTR-REQ-ERROR-001：Pointer 错误报告

**输入/输出:**
- Pointer 操作中的错误情况。

**处理逻辑要点:**
1. 返回包含错误码与偏移信息的错误对象，与 component 级错误模型兼容；
2. 区分解析错误与访问错误，便于上层诊断。

---

## 3. 接口设计

### 3.1 外部接口（extern "C"）

本 feature 不直接定义 extern "C" 接口，Pointer 相关 FFI 将由 FFI component 统一设计。本节**无特殊设计**。

### 3.2 内部接口（Rust API 概要）

```rust
pub struct Pointer {
    // 内部 token 列表等（具体实现阶段细化）
}

impl Pointer {
    pub fn parse(text: &str) -> Result<Pointer, crate::error::Error> {
        // 仅示意
        unimplemented!()
    }

    pub fn get<'a>(&self, root: &'a crate::dom::Value) -> Result<&'a crate::dom::Value, crate::error::Error> {
        unimplemented!()
    }

    pub fn set(&self, root: &mut crate::dom::Value, value: crate::dom::Value) -> Result<(), crate::error::Error> {
        unimplemented!()
    }
}
```

> 上述接口仅用于说明方向，具体类型与错误细节将在实现阶段细化。

### 3.3 与其他 feature 的接口关系

**使用其他 feature 的接口:**

| feature ID | 接口名称 | 用途 | feature 开发设计文档追溯 |
|------------|----------|------|--------------------------|
| `FEAT-003` (`dom-core`) | `Document`/`Value` 类型与对象/数组 API | 按路径遍历和修改 DOM | [dom-core-dev-design.md](../FEAT-003/dom-core-dev-design.md) |
| `FEAT-001` (`core-infra`) | 错误类型 `Error` | 在解析与操作失败时统一返回错误 | [core-infra-dev-design.md](../FEAT-001/core-infra-dev-design.md) |

**暴露给其他 feature 的接口（宏观）：**

| 接口名称 | 接口类型 | 暴露给的 feature | 用途 | 说明 |
|----------|----------|------------------|------|------|
| `Pointer` 类型与相关 API | public | `schema-validate`, 上层应用 | 通过路径访问/修改 DOM | Schema 校验中使用 Pointer 访问节点 |

---

## 4. 数据结构

本 feature 主要依赖指向 DOM 的路径 token 列表，内部实现形式可为：

- `Vec<Token>`，其中 `Token` 为对象 key 或数组 index 的枚举；
- 或更轻量的结构（如 slice 视图），具体方案由实现阶段性能分析决定。本节**无特殊设计**。

---

## 5. 实现要点

实现要点包括：

- 按 RFC 6901 与 C++ 行为实现解析与转义规则；
- 在路径操作中优先使用 DOM 提供的 API，避免重复实现对象/数组逻辑；
- 在错误场景中返回精确的错误类型和位置，以便上层记录与调试。

详细算法和伪代码留待实现阶段，本节**无特殊设计**。

---

## 变更历史

| 版本 | 日期 | 变更内容 | 变更人 |
|------|------|----------|--------|
| `0.1.0-draft` | `2026-05-14` | 初始化 FEAT-005 `pointer-path` feature 级开发设计文档。 | `TBD` |
