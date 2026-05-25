# Feature 级开发设计文档

## 报告基本信息

| 字段 | 内容 |
|------|------|
| feature ID | `FEAT-006` |
| feature 名称 | `schema-validate` |
| 所属 component | `rapidjson-rs` (`COMP-001`) |
| 所属 crate | `rapidjson-rs` |
| 文档版本 | `0.1.0-draft` |
| 设计日期 | `2026-05-14` |
| 架构师 | `TBD` |
| 文档状态 | `草稿` |
| component 设计追溯 | [`rapidjson-rs-dev-design.md` 第 2.3、3.6、4.1 schema 模块](../rapidjson-rs-dev-design.md#4-module-划分) |
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
| crates.io 使用策略 | Schema 校验实现不得依赖第三方 JSON Schema 库或 URI 解析库，所有逻辑在 `rapidjson-rs` 内实现；正则与 URI 解析需复用已有 internal/regex 与 uri 工具。 |
| 对当前 feature 技术选型的影响 | JSON Schema 支持必须基于 dom-core、sax-io、pointer-path 与 core-infra/encoding-unicode 的组合，不引入任何外部验证框架。 |

### 1.1 feature 职责

**一句话描述:**
- `schema-validate` 实现 JSON Schema Draft v4 校验能力，包括 Schema 编译到内部表示、DOM/SAX 校验、远程引用支持、错误报告和 Swagger/OpenAPI 扩展支持。

**详细职责:**
- 将 JSON Schema 文档编译成内部 `SchemaDocument` 表示；
- 支持对 DOM（Document/Value）进行 Schema 校验（Accept 模式）以及对 SAX 解析过程进行同步校验；
- 支持在 Writer 序列化过程中进行校验（可选）；
- 支持远程 Schema 引用，通过用户提供的 Provider 加载；
- 提供详细的错误报告，包括违反的关键字、实例路径、Schema 路径等；
- 支持基本的 URI 解析与合并，用于处理 `$ref` 等；
- 支持 Swagger v2 与 OpenAPI v3.0.x Schema 的兼容扩展（在行为上与 C++ 实现相同）。

**不在职责范围内:**
- 不承担 JSON 文本解析/生成工作，仅基于已有 DOM/SAX/Writer 接口进行校验；
- 不支持超出 Draft v4 范围的 Schema 特性，除非需求文档明确要求并在后续版本设计中扩展。

### 1.2 项目中的位置

- 所属 crate：`rapidjson-rs`。
- 预期 module 路径：`rapidjson_rs::schema`。

### 1.3 重构策略

**重构策略:** `高风险模块，渐进式重构`。

- 先实现核心 Draft v4 功能子集（如 type/enum/properties/items/required 等），并通过 schematest 关键用例验证；
- 随后分阶段引入远程引用、复杂组合关键字（allOf/anyOf/oneOf/not）、Swagger/OpenAPI 扩展等；
- 通过镜像/孪生测试与结构化错误报告对比，逐渐收敛行为差异；
- 性能方面，在功能稳定后再通过 perftest schematest 进行优化。

**C/C++ 代码对应部分:**

| 源文件 | 范围 | 复杂度 |
|-------|-----|-------|
| `rapidjson_legacy/include/rapidjson/schema.h` | SchemaDocument、SchemaValidator、SchemaValidatingReader/Writer 等 | 高 |
| `rapidjson_legacy/include/rapidjson/uri.h` | URI 解析与合并 | 中 |

**相关测试：**
- `rapidjson_legacy/test/unittest/schematest.cpp`
- `rapidjson_legacy/test/perftest/schematest.cpp`

---

## 2. 功能需求

### 2.1 功能清单

| 需求 ID | 功能名称 | 优先级 | 复杂度 | component 开发设计文档追溯 | 状态 |
|---------|----------|--------|--------|-----------------------------|------|
| `SCH-REQ-COMPILE-001` | 编译 JSON Schema 为 SchemaDocument | P0 | 高 | [REQ-JS-01](../../requirements/requirements.md#2-6-json-schema) | 待开发 |
| `SCH-REQ-VALIDATE-DOM-001` | DOM 校验（Accept 模式） | P0 | 高 | [REQ-JS-02](../../requirements/requirements.md#2-6-json-schema) | 待开发 |
| `SCH-REQ-VALIDATE-SAX-001` | SAX 校验 | P1 | 高 | [REQ-JS-03](../../requirements/requirements.md#2-6-json-schema) | 待开发 |
| `SCH-REQ-VALIDATE-WRITER-001` | Writer 序列化时校验 | P2 | 中 | [REQ-JS-04](../../requirements/requirements.md#2-6-json-schema) | 待开发 |
| `SCH-REQ-REMOTE-001` | 远程 Schema 引用支持 | P2 | 高 | [REQ-JS-05](../../requirements/requirements.md#2-6-json-schema) | 待开发 |
| `SCH-REQ-ERROR-001` | 校验错误报告 | P0 | 高 | [REQ-JS-06](../../requirements/requirements.md#2-6-json-schema) | 待开发 |
| `SCH-REQ-REGEX-001` | 内建正则引擎支持 pattern/patternProperties | P1 | 中 | [REQ-JS-07](../../requirements/requirements.md#2-6-json-schema) | 待开发 |
| `SCH-REQ-URI-001` | URI 解析与合并 | P2 | 中 | [REQ-JS-08](../../requirements/requirements.md#2-6-json-schema) | 待开发 |
| `SCH-REQ-OAS-001` | Swagger v2/OpenAPI v3 Schema 支持 | P3 | 中 | [REQ-JS-09](../../requirements/requirements.md#2-6-json-schema) | 待开发 |

### 2.2 功能详细规格

#### SCH-REQ-COMPILE-001：编译 JSON Schema 为 SchemaDocument

**输入:**
- DOM 表示的 Schema 文档（`Document` 或 `Value`）；
- 编译配置（是否允许某些扩展等）。

**输出:**
- 内部 `SchemaDocument` 表示，与 C++ 实现功能相当；
  - 包含结构化的关键字信息、类型约束、范围约束、组合关键字等。

**处理逻辑要点:**
1. 解析 Schema DOM，按 JSON Schema Draft v4 规则构建内部结构；
2. 支持 `$ref`，与 URI 模块配合解析并合并引用；
3. 对错误 Schema（不合法结构、未知 draft/version 等）返回明确错误。

#### SCH-REQ-VALIDATE-DOM-001：DOM 校验（Accept 模式）

**输入:**
- 目标 DOM（`Value`/`Document`）和预编译 SchemaDocument；

**输出:**
- 校验通过/失败结果，以及在失败时的详细错误信息（参见 SCH-REQ-ERROR-001）。

**处理逻辑要点:**
1. 按 Schema 关键字逐步检查 DOM；
2. 支持多种类型约束、枚举、范围、模式、组合关键字等；
3. 行为与 C++ SchemaValidator 保持一致，参照 `schematest.cpp` 中大量测试用例。

#### SCH-REQ-VALIDATE-SAX-001：SAX 校验

**输入:**
- SchemaDocument 与 SAX 解析过程产生的事件序列；

**输出:**
- 与 DOM 校验相同的结果与错误信息，但在解析过程中同步进行。

**处理逻辑要点:**
1. 与 `sax-io` feature 集成，通过 Handler 或中间层进行校验；
2. C++ 参考实现为 SchemaValidatingReader，本 feature 将设计类似结构。

#### SCH-REQ-VALIDATE-WRITER-001：Writer 序列化时校验

**输入/输出:**
- 在 Writer 输出事件流时，对输出内容进行 Schema 校验；

**处理逻辑要点:**
1. 与 `sax-io` Writer 集成，监听事件并反馈校验结果；
2. 与 C++ 实现行为保持一致，视需求文档优先级决定实现深度。

#### SCH-REQ-REMOTE-001：远程 Schema 引用支持

**输入:**
- Schema 中带有 `$ref` 的字段以及用户提供的 Provider；

**输出:**
- 可解析远程引用并构建完整 SchemaDocument 的行为；

**处理逻辑要点:**
1. 使用 URI 模块解析/合并 URI；
2. 通过 Provider 回调获取远程 Schema 文档并编译；
3. 对于找不到的引用返回错误，并在错误报告中记录路径与 URI。

#### SCH-REQ-ERROR-001：校验错误报告

**输入/输出:**
- 校验过程中产生的错误；

**处理逻辑要点:**
1. 返回包括违反关键字、实例路径、Schema 路径、消息等信息的错误对象；
2. 行为与 C++ 错误报告保持一致，参照 `schematest.cpp` 中相关测试（如 `SchemaValidator.*`）。

#### SCH-REQ-REGEX-001：内建正则引擎支持

**输入/输出:**
- 在处理 `pattern`/`patternProperties` 等关键字时使用 internal 正则引擎；

**处理逻辑要点:**
1. 复用 core-infra 的 `internal::regex` 实现；
2. 不引入第三方正则库；
3. 行为与 C++ 相同（包括错误场景）。

#### SCH-REQ-URI-001：URI 解析与合并

**输入/输出:**
- 在 `$id`/`$ref` 等字段处理过程使用 URI 工具；

**处理逻辑要点:**
1. 基于 `uri.h` 重写 URI 解析与合并逻辑；
2. 用于处理本地与远程 Schema 引用。

#### SCH-REQ-OAS-001：Swagger/OpenAPI Schema 支持

**输入/输出:**
- 当 Schema 中包含 Swagger v2/OpenAPI v3.0.x 特定结构时，按 C++ 扩展规则处理；

**处理逻辑要点:**
1. 与 C++ 行为一致，作为扩展支持；
2. 如有行为差异，应在文档中记录。

---

## 3. 接口设计

### 3.1 外部接口（extern "C"）

本 feature 不直接提供 extern "C" 接口，Schema FFI 由 FFI component 设计。本节**无特殊设计**。

### 3.2 内部接口（Rust API 概要）

```rust
pub struct SchemaDocument {
    // 内部结构，具体实现阶段细化
}

pub struct SchemaValidator<'a> {
    // 内部状态
    _phantom: core::marker::PhantomData<&'a ()>,
}

impl SchemaDocument {
    pub fn compile(schema: &crate::dom::Value) -> Result<SchemaDocument, crate::error::Error> {
        unimplemented!()
    }
}

impl<'a> SchemaValidator<'a> {
    pub fn validate_dom(
        &mut self,
        instance: &crate::dom::Value,
    ) -> Result<(), crate::error::Error> {
        unimplemented!()
    }
}
```

> 接口仅作方向示意，具体类型/生命周期与错误类型将在实现阶段细化。

### 3.3 与其他 feature 的接口关系

**使用其他 feature 的接口:**

| feature ID | 接口名称 | 用途 | feature 开发设计文档追溯 |
|------------|----------|------|--------------------------|
| `FEAT-003` (`dom-core`) | `Document`/`Value` | Schema 校验中的实例表示 | [dom-core-dev-design.md](../FEAT-003/dom-core-dev-design.md) |
| `FEAT-004` (`sax-io`) | Reader/Writer + Handler | SAX 校验与 Writer 校验 | [sax-io-dev-design.md](../FEAT-004/sax-io-dev-design.md) |
| `FEAT-005` (`pointer-path`) | Pointer API | 处理 `$ref` 等 Schema 引用 | [pointer-path-dev-design.md](../FEAT-005/pointer-path-dev-design.md) |
| `FEAT-001` (`core-infra`) | 错误类型、regex/uri 工具 | 错误报告与正则/URI 处理 | [core-infra-dev-design.md](../FEAT-001/core-infra-dev-design.md) |

**暴露给其他 feature 的接口（宏观）：**

| 接口名称 | 接口类型 | 暴露给的 feature | 用途 | 说明 |
|----------|----------|------------------|------|------|
| `SchemaDocument` | public | 上层应用 | 管理编译后的 Schema 表示 | 供用户/应用保存 Schema |
| `SchemaValidator` | public | 上层应用 | 执行 Schema 校验 | 供 DOM/SAX/Writer 场景使用 |

---

## 4. 数据结构

本 feature 内部数据结构（表示 Schema 树、关键字、约束等）较复杂，具体设计将在实现阶段结合 C++ 实现与测试用例详细确定。本节**无特殊设计**。

---

## 5. 实现要点

实现要点包括：

- 关键字处理顺序与 C++ 行为对齐；
- 错误报告结构（包含关键字、实例路径、Schema 路径）；
- 与 DOM/SAX/Pointers 的集成；
- 性能优化与短路策略（仅在必要时进行）。

详细算法与伪代码将在实现阶段设计，本节**无特殊设计**。

---

## 变更历史

| 版本 | 日期 | 变更内容 | 变更人 |
|------|------|----------|--------|
| `0.1.0-draft` | `2026-05-14` | 初始化 FEAT-006 `schema-validate` feature 级开发设计文档。 | `TBD` |
