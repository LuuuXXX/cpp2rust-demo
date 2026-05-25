# Feature 级开发设计文档

## 报告基本信息

| 字段 | 内容 |
|------|------|
| feature ID | `FEAT-003` |
| feature 名称 | `dom-core` |
| 所属 component | `rapidjson-rs` (`COMP-001`) |
| 所属 crate | `rapidjson-rs` |
| 文档版本 | `0.1.0-draft` |
| 设计日期 | `2026-05-14` |
| 架构师 | `TBD` |
| 文档状态 | `草稿` |
| component 设计追溯 | [`rapidjson-rs-dev-design.md` 第 2.1、3.6、4.1 dom 模块](../rapidjson-rs-dev-design.md#4-module-划分) |
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
| crates.io 使用策略 | DOM 实现不得依赖任何第三方 JSON/集合/序列化/反序列化 crate（如 `serde_json`），所有 DOM 行为完全在 `rapidjson-rs` 内实现。 |
| 对当前 feature 技术选型的影响 | DOM 内部数据结构需基于 `core`/`std` 容器或自研结构实现；不引入第三方错误库/派发库，错误通过 core-infra 提供的类型返回。 |

**约束说明:**
- DOM 为核心对外能力，所有第三方依赖禁用策略对本 feature 约束最为严格，避免在对象表示层引入无法替换的外部库。

### 1.1 feature 职责

**一句话描述:**
- `dom-core` 提供 RapidJSON 风格的树形 JSON 表示与操作接口，包括 `Value` 与 `Document` 类型、类型查询、强类型数值 API、对象/数组操作、零拷贝字符串引用、深拷贝与比较等，为解析与生成提供内存中的中间表示。

**详细职责:**
- 提供 7 种 JSON 值类型：Null、Bool、Number、String、Array、Object，以及对应的类型查询 API。
- 提供强类型数值接口：`i32`/`u32`/`i64`/`u64`/`f64`，并保证与需求文档和 C++ 行为一致。
- 提供对象成员增删改查接口（按 key 访问、插入、删除、合并等）。
- 提供数组元素增删改查接口（按 index 访问、push/pop/erase range 等）。
- 支持字符串值，包括包含 `\0` 的字符串、零拷贝引用与深拷贝选项。
- 提供 `Document` 作为顶层容器，包含解析状态、Allocator 与根节点等信息。
- 提供 DOM 深拷贝与值比较能力，用于 Schema 校验与 Pointer 操作等高级功能。

**不在职责范围内:**
- 不直接实现 JSON 文本解析/生成逻辑（由 `sax-io` 或解析器模块承担），但 DOM 必须为这些模块提供足够的构建/更新接口。
- 不处理编码细节（由 `encoding-unicode` feature 负责），DOM 仅操作统一编码表示的字符串。

### 1.2 项目中的位置

- 所属 crate：`rapidjson-rs`。
- 预期 module 路径：`rapidjson_rs::dom`（子模块细分视实现需要决定）。

### 1.3 重构策略

**重构策略:** `渐进式重构 + 完全 Rust 化`。

- 先建立与 C++ `Value`/`Document` 行为等价的公开 API，并通过 gtest 镜像测试验证行为；
- 不通过 FFI 重用 C++ DOM，实现阶段直接用 Rust 编写数据结构与操作逻辑；
- 通过 `documenttest.cpp` 和 `valuetest.cpp` 中的大量用例，逐步补齐边界行为与历史问题场景。

**技术选型:**
- 使用枚举 + 结构体组合表示 `Value`，避免泛型过度复杂化；
- 使用 `core`/`std` 容器（如 `Vec`、`String`、`HashMap` 或自研 map 结构）实现对象/数组，具体结构在实现阶段由性能分析指导；
- 内存管理通过 FEAT-001 `core-infra` 的分配器接口实现。

**C/C++ 代码对应部分:**

| 源文件 | 范围 | 复杂度 |
|-------|-----|-------|
| `rapidjson_legacy/include/rapidjson/document.h` | `Value`/`Document` 定义与 DOM 操作接口 | 高 |

**相关测试（主要引用）：**
- `rapidjson_legacy/test/unittest/documenttest.cpp`
- `rapidjson_legacy/test/unittest/valuetest.cpp`

---

## 2. 功能需求

### 2.1 功能清单

| 需求 ID | 功能名称 | 优先级 | 复杂度 | component 开发设计文档追溯 | 状态 |
|---------|----------|--------|--------|-----------------------------|------|
| `DOM-REQ-TYPE-001` | 支持 7 种 JSON 值类型及类型查询 | P0 | 中 | [FR-002, 2.3 DOM 操作](../rapidjson-rs-dev-design.md#1-关键需求) | 待开发 |
| `DOM-REQ-NUM-001` | 强类型数值接口（i32/u32/i64/u64/f64） | P0 | 中 | [FR-002, 数值 API](../rapidjson-rs-dev-design.md#1-关键需求) | 待开发 |
| `DOM-REQ-OBJ-001` | 对象成员增删改查 | P0 | 高 | [FR-002, 对象操作](../rapidjson-rs-dev-design.md#1-关键需求) | 待开发 |
| `DOM-REQ-ARR-001` | 数组元素增删改查和范围操作 | P0 | 高 | [FR-002, 数组操作](../rapidjson-rs-dev-design.md#1-关键需求) | 待开发 |
| `DOM-REQ-STR-001` | 字符串表示（含 `\0`、零拷贝引用） | P1 | 高 | [FR-002, FR-D-16, FR-D-17](../rapidjson-rs-dev-design.md#1-关键需求) | 待开发 |
| `DOM-REQ-DOC-001` | Document 顶层容器与解析状态 | P0 | 中 | [需求文档 UC-01/UC-02](../../requirements/requirements.md#uc-01解析-json-字符串到-dom) | 待开发 |
| `DOM-REQ-COPY-001` | 深拷贝与值比较 | P1 | 中 | [需求文档 2.3 REQ-D-14/18](../../requirements/requirements.md#2-3-dom-操作) | 待开发 |

### 2.2 功能详细规格

#### DOM-REQ-TYPE-001：支持 7 种 JSON 值类型及类型查询

**输入:**
- 用户创建或解析后得到的 `Value`/`Document` 对象。

**输出:**
- 类型查询接口结果（如 `is_null()`、`is_bool()`、`is_number()`、`is_string()`、`is_array()`、`is_object()`）。

**处理逻辑:**
1. 对每个 `Value` 存储内部类型标签；
2. 类型查询 API 只读访问标签，不触发昂贵操作；
3. 行为与 C++ 实现保持一致，包括所有边界类型（如空字符串、空数组、空对象）。

#### DOM-REQ-NUM-001：强类型数值接口

**输入:**
- 数值类型 `Value` 实例；用户通过解析或构造接口创建。

**输出:**
- 精确的整数/浮点视图（`i32`/`u32`/`i64`/`u64`/`f64`），以及相关判断接口（如 `is_lossless_double()`）。

**处理逻辑要点:**
1. 内部表示可采用统一数值结构或区分整数/浮点存储；
2. 访问接口应遵循 C++ 行为：当类型无法无损转换时，返回失败或特定错误，而非静默截断；
3. 利用 `encoding-unicode` 和 `core-infra` 内部大整数/浮点工具，确保转换与比较精度。

#### DOM-REQ-OBJ-001：对象成员增删改查

**输入:**
- 对象类型 `Value` 和键值对信息（键名、值）。

**输出:**
- 更新后的对象 `Value`，或查询/删除操作的结果。

**处理逻辑要点:**
1. 提供按 key 查询、插入、更新、删除的接口；
2. 保持与 C++ RapidJSON 在 key 冲突、插入顺序等方面的行为一致（参考 `valuetest.cpp`）；
3. 使用 `core-infra` Allocator 管理对象成员内存；
4. 字符串 key 应支持零拷贝引用及深拷贝两种模式。

#### DOM-REQ-ARR-001：数组元素增删改查和范围操作

**输入:**
- 数组类型 `Value` 与 index/范围参数。

**输出:**
- 修改后的数组或被移除元素。

**处理逻辑要点:**
1. 提供索引访问、插入、删除、范围删除等接口；
2. 行为应与 C++ `Value::PushBack`/`Erase` 等 API 保持一致；
3. 使用 `core-infra` Allocator 分配数组存储。

#### DOM-REQ-STR-001：字符串表示

**输入:**
- 字符串值（含 `\0` 或其他特殊字符），以及内存所有权策略（拷贝/引用）。

**输出:**
- DOM 中的字符串 `Value`；
- 在需要时提供底层指针与长度访问接口供 Pointer/Schema 等模块使用。

**处理逻辑要点:**
1. 支持包含 `\0` 的字符串，采用显式长度表示；
2. 零拷贝引用模式依赖调用方保证源缓冲区生命周期足够长；
3. 深拷贝模式将字符串数据复制到由 Allocator 管理的缓冲区中。

#### DOM-REQ-DOC-001：Document 顶层容器与解析状态

**输入:**
- 解析/构造逻辑传入的根节点与解析元信息（偏移、错误状态等）。

**输出:**
- 完整 `Document` 对象，包含根 `Value` 与关联的 Allocator、解析错误信息等。

**处理逻辑要点:**
1. `Document` 管理 DOM 生命周期及其关联内存；
2. 提供查询解析错误状态、重置文档等接口；
3. 与 C++ `Document` 类型在公共行为上保持一致。

#### DOM-REQ-COPY-001：深拷贝与值比较

**输入:**
- 源 `Value`/`Document` 与目标分配器/上下文；

**输出:**
- 深拷贝的 `Value`/`Document`，或比较结果布尔值。

**处理逻辑要点:**
1. 深拷贝应递归复制所有子节点，以避免共享可变结构导致数据竞争；
2. 比较应按照 C++ 实现定义的语义（包括数组/对象顺序等），参照 `valuetest.cpp`；
3. 注意避免递归过深导致栈溢出，可使用迭代算法或限制深度。

---

## 3. 接口设计

### 3.1 外部接口（extern "C"）

本 feature 不直接定义 extern "C" 接口，DOM 相关 FFI API 将由 FFI component 统一设计。本节**无特殊设计**。

### 3.2 内部接口（Rust API 概要）

> 以下为面向 Rust 调用方的接口方向示例，具体签名在实现阶段细化。本节不含函数体。

```rust
pub enum Value {
    Null,
    Bool(bool),
    Number(Number),
    String(StringValue),
    Array(ArrayValue),
    Object(ObjectValue),
}

pub struct Document {
    // 根节点与分配器等元数据（具体字段实现阶段确定）
}

impl Value {
    pub fn is_null(&self) -> bool { /* ... */ }
    pub fn is_bool(&self) -> bool { /* ... */ }
    // 其他类型查询与访问接口...
}
```

> 注：以上代码为接口形态示例，实际文档不包含实现细节。

### 3.3 与其他 feature 的接口关系

**使用其他 feature 的接口:**

| feature ID | 接口名称 | 用途 | feature 开发设计文档追溯 |
|------------|----------|------|--------------------------|
| `FEAT-001` (`core-infra`) | `Allocator`、错误类型 `Error` | 为 DOM 节点分配内存、在操作失败时返回统一错误类型 | [core-infra-dev-design.md 3.2 小节](../FEAT-001/core-infra-dev-design.md) |
| `FEAT-002` (`encoding-unicode`) | 编码工具接口 | 在字符串构造/序列化过程中进行编码转换与验证 | [encoding-unicode-dev-design.md](../FEAT-002/encoding-unicode-dev-design.md) |

**暴露给其他 feature 的接口（宏观）：**

| 接口名称 | 接口类型 | 暴露给的 feature | 用途 | 函数/类型签名（示意） |
|----------|----------|------------------|------|-------------------|
| `Document`/`Value` | public 类型 | `sax-io`, `pointer-path`, `schema-validate` | DOM 表示与操作 | `pub struct Document`, `pub enum Value` |
| 对象访问接口 | public | `pointer-path`, `schema-validate` | 根据 key/path 获取或修改节点 | `pub fn get(&self, key: &str) -> Option<&Value>` 等 |
| 数组访问接口 | public | `pointer-path`, `schema-validate` | 根据 index 操作数组 | `pub fn at(&self, index: usize) -> Option<&Value>` 等 |

**接口一致性说明:**
- DOM 对外接口需稳定，便于 FFI 与其他 feature 使用；
- 值比较与深拷贝接口应与 C++ 行为一致，避免出现语义偏差影响 Schema/Pointers 等功能。

---

## 4. 数据结构

### 4.1 类型定义（概念级）

本 feature 的核心类型为 `Value` 与 `Document`，本节给出概念级结构示意。具体字段与内存布局将在实现阶段根据性能与安全要求细化。

```rust
pub enum Number {
    I32(i32),
    U32(u32),
    I64(i64),
    U64(u64),
    F64(f64),
}

pub struct StringValue {
    // 指向底层字符串缓冲的指针/索引 + 长度 + 所有权标记
}

pub struct ArrayValue {
    // 使用 Vec-like 结构存放元素，底层由 Allocator 管理
}

pub struct ObjectValue {
    // 使用键值结构存放成员，可为 Vec<(StringValue, Value)> 或自研 map
}
```

### 4.2 数据结构设计

| 字段/变量 | 类型 | 用途 |
|-----------|------|------|
| `Number::*` | 各整数/浮点类型 | 精确存储数值，避免无谓转换损失 |
| `StringValue.len` | `usize` | 字符串长度，支持包含 `\0` 的字符串 |
| `StringValue.owned` | `bool` 或枚举 | 标记是否由 DOM 拥有底层缓冲 |
| `ArrayValue.items` | 内部元素容器 | 存放数组元素的顺序集合 |
| `ObjectValue.members` | 内部键值集合 | 存放对象成员，键为 `StringValue` |

### 4.3 内存布局

内存布局由 Allocator 控制，DOM 类型本身不定义特定的 C 对齐约束。本节**无特殊设计**。

---

## 5. 实现要点

### 5.1 关键算法

本 feature 关键实现点包括：

- 数值存储与访问策略：如何映射 C++ 中的数值表示到 Rust；
- 对象/数组操作复杂度控制（如插入/删除开销）；
- 深拷贝与比较算法的效率与递归深度控制。

具体算法细节将在实现阶段结合 C++ 代码与 gtest 行为进行设计，本节**不展开伪代码**。

### 5.2 错误处理

DOM 操作错误（如索引越界、键不存在等）应通过返回 `Option` 或 `Result` 表达，不使用 panic 作为跨模块错误机制。本节**无特殊设计**。

### 5.3 性能优化

性能优化策略将在实现阶段结合基准测试确定，本节**无特殊设计**。

### 5.4 内存管理

DOM 不自行管理底层内存，仅通过 `core-infra` 的 Allocator 实例执行分配与释放。本节**无特殊设计**。

### 5.5 并发安全

DOM 类型不保证内部并发修改安全；多线程使用应由调用方维护适当同步。本节**无特殊设计**。

---

## 变更历史

| 版本 | 日期 | 变更内容 | 变更人 |
|------|------|----------|--------|
| `0.1.0-draft` | `2026-05-14` | 初始化 FEAT-003 `dom-core` feature 级开发设计文档。 | `TBD` |
