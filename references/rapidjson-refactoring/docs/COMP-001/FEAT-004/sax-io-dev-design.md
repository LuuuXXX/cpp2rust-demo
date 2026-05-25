# Feature 级开发设计文档

## 报告基本信息

| 字段 | 内容 |
|------|------|
| feature ID | `FEAT-004` |
| feature 名称 | `sax-io` |
| 所属 component | `rapidjson-rs` (`COMP-001`) |
| 所属 crate | `rapidjson-rs` |
| 文档版本 | `0.1.0-draft` |
| 设计日期 | `2026-05-14` |
| 架构师 | `TBD` |
| 文档状态 | `草稿` |
| component 设计追溯 | [`rapidjson-rs-dev-design.md` 第 2.1、3.6、4.1 sax 模块](../rapidjson-rs-dev-design.md#4-module-划分) |
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
| crates.io 使用策略 | SAX/Writer 不得依赖第三方 JSON/流/事件框架（如 `serde`、`tokio`），所有解析/生成逻辑在 `rapidjson-rs` 内实现。 |
| 对当前 feature 技术选型的影响 | 流式解析与生成逻辑必须直接使用 `core-infra` 提供的流抽象和编码工具；不引入 async 运行时或第三方 IO 抽象。 |

### 1.1 feature 职责

**一句话描述:**
- `sax-io` 提供基于事件的 JSON 解析与生成能力，包括 `Reader`/`Writer` 核心管线、Handler 接口、迭代式解析、多文档流与宽松语法支持，为 DOM 构建与纯 SAX 场景提供高性能 IO 通道。

**详细职责:**
- 提供 `Reader` 类型，从输入流读取 JSON 文本，并按照 Handler 接口定义发送 SAX 事件（Null/Bool/Number/String/Key/StartObject/EndObject/StartArray/EndArray 等）。
- 提供 `Writer` 和 `PrettyWriter` 类型，根据调用顺序输出 JSON 文本，支持紧凑输出与格式化输出。
- 提供 `BaseReaderHandler` 等默认 Handler 实现，为用户简化实现负担。
- 支持迭代式解析（Iterative Parse）与多文档流解析，满足需求文档 2.1/2.4 中相关要求。
- 支持需求文档规定的宽松语法选项（注释、尾逗号、NaN/Infinity 等），在配置允许时接受这些扩展语法。

**不在职责范围内:**
- 不负责 DOM 内部结构的构建（由 DOM feature 负责），但必须为 DOM 提供 Handler 风格的构建接入点；
- 不处理 Schema 校验逻辑，仅在 SAX 事件层提供可能的 hook；
- 不负责文件/网络 IO 细节，输入输出依赖 `core-infra` 流抽象。

### 1.2 项目中的位置

- 所属 crate：`rapidjson-rs`。
- 预期 module 路径：`rapidjson_rs::sax`（或类似命名）。

### 1.3 重构策略

**重构策略:** `渐进式重构 + 行为对齐 C++`。

- 起步阶段，将 C++ Reader/Writer 行为映射为 Rust 类型与接口，不考虑全部宽松语法选项；
- 随着镜像测试与孪生测试覆盖扩大，逐步引入宽松语法、迭代式解析、多文档流等高级特性；
- 解析与生成算法直接用 Rust 实现，避免通过 FFI 调用 C++ 解析器，以便在 Rust 侧统一错误模型与内存安全。

**技术选型:**
- 利用 `core-infra` 的 `InputStream`/`OutputStream` trait 组合输入/输出；
- 利用 `encoding-unicode` 提供的解码/编码接口处理多编码输入；
- 事件分发通过 trait/泛型完成，不使用动态分发以减少开销，但需平衡灵活性。

**C/C++ 代码对应部分:**

| 源文件 | 范围 | 复杂度 |
|-------|-----|-------|
| `rapidjson_legacy/include/rapidjson/reader.h` | SAX 解析管线（Reader + Handler 接口 + 迭代式解析等） | 高 |
| `rapidjson_legacy/include/rapidjson/writer.h` | SAX 生成管线（Writer + PrettyWriter 等） | 中/高 |
| `rapidjson_legacy/include/rapidjson/istreamwrapper.h`/`ostreamwrapper.h`/`cursorstreamwrapper.h` | 流封装与游标行为 | 中 |

**相关测试：**
- `rapidjson_legacy/test/unittest/readertest.cpp`
- `rapidjson_legacy/test/unittest/writertest.cpp`
- `rapidjson_legacy/test/unittest/cursorstreamwrappertest.cpp`
- `rapidjson_legacy/test/unittest/jsoncheckertest.cpp`
- `rapidjson_legacy/test/perftest/rapidjsontest.cpp` 中 SAX/Writer 相关性能测试。

---

## 2. 功能需求

### 2.1 功能清单

| 需求 ID | 功能名称 | 优先级 | 复杂度 | component 开发设计文档追溯 | 状态 |
|---------|----------|--------|--------|-----------------------------|------|
| `SAX-REQ-READ-001` | SAX 解析（Reader + Handler） | P0 | 高 | [FR-003, 2.4 SAX 事件处理](../rapidjson-rs-dev-design.md#1-关键需求) | 待开发 |
| `SAX-REQ-WRITE-001` | SAX 生成（Writer + PrettyWriter） | P0 | 高 | [FR-003, FR-G-01/02/03](../rapidjson-rs-dev-design.md#1-关键需求) | 待开发 |
| `SAX-REQ-ITER-001` | 迭代式解析与多文档流 | P1 | 高 | [REQ-S-06, REQ-P-12](../../requirements/requirements.md#2-4-sax-事件处理) | 待开发 |
| `SAX-REQ-LOOSE-001` | 宽松语法（注释/尾逗号/NaN/Infinity） | P1 | 中 | [REQ-P-08/09/10/17, REQ-S-07](../../requirements/requirements.md#2-1-json-解析parsing) | 待开发 |
| `SAX-REQ-HANDLER-001` | 默认 Handler 与错误处理策略 | P0 | 中 | [REQ-S-05](../../requirements/requirements.md#2-4-sax-事件处理) | 待开发 |

### 2.2 功能详细规格

#### SAX-REQ-READ-001：SAX 解析（Reader + Handler）

**输入:**
- `InputStream` 实例（来自 `core-infra`），以及 Handler 实现；
- 编码与解析配置（宽松语法开关、是否逐 token 等）。

**输出:**
- Handler 收到的 SAX 事件序列；如遇解析错误，返回 `Error` 或通过 Handler 终止解析并携带错误信息。

**处理逻辑要点:**
1. 使用 `encoding-unicode` 解码字节流为码点；
2. 按 JSON 语法（或宽松扩展）识别 token 并驱动事件回调；
3. 提供统一入口（如 `parse(stream, handler, config)`）。

#### SAX-REQ-WRITE-001：SAX 生成（Writer + PrettyWriter）

**输入:**
- `OutputStream` 实例（来自 `core-infra`），调用方依序调用 `Null`/`Bool`/`Int`/`String`/`StartObject` 等接口；
- 格式化配置（缩进风格、换行策略等）。

**输出:**
- 写入到输出流的 JSON 文本；遇错误时返回统一 `Error`。

**处理逻辑要点:**
1. 维护内部状态机确保输出结构合法；
2. 正确处理 NaN/Infinity/宽松数字输出选项；
3. 与 C++ Writer 行为一致（参见 `writertest.cpp`）。

#### SAX-REQ-ITER-001：迭代式解析与多文档流

**输入:**
- 流和 Handler，调用方希望在多次调用中逐渐解析输入（如流式网络读取）。

**输出:**
- 每次迭代的状态（继续/完成/错误），以及已消费的输入位置；

**处理逻辑要点:**
1. 内部维护解析状态，支持暂停与继续；
2. 支持在同一流中解析多个 JSON 文档（StopWhenDone）。

#### SAX-REQ-LOOSE-001：宽松语法

**输入/输出:**
- 在解析/生成接口中增加配置选项，启用或禁用注释、尾逗号、NaN/Infinity 等特性；

**处理逻辑要点:**
1. 默认关闭宽松语法，与标准一致；
2. 开启后需与 C++ 行为精确对齐，参考 `readertest.cpp` 中评论相关用例。

#### SAX-REQ-HANDLER-001：默认 Handler

**输入/输出:**
- 提供一个默认 Handler 基类/实现，用户可继承/组合以实现定制逻辑；

**处理逻辑要点:**
1. 默认实现所有接口为“空操作”，方便用户只重载关心的事件；
2. 提供终止解析的机制（如 Handler 返回 false 触发 early stop）。

---

## 3. 接口设计

### 3.1 外部接口（extern "C"）

本 feature 不直接定义 extern "C" 接口，SAX 层 FFI API 将由 FFI component 统一设计。本节**无特殊设计**。

### 3.2 内部接口（Rust API 概要）

```rust
pub trait Handler {
    fn null(&mut self) -> bool;
    fn bool_(&mut self, b: bool) -> bool;
    fn int(&mut self, i: i32) -> bool;
    fn uint(&mut self, u: u32) -> bool;
    fn int64(&mut self, i: i64) -> bool;
    fn uint64(&mut self, u: u64) -> bool;
    fn double(&mut self, d: f64) -> bool;
    fn string(&mut self, s: &str) -> bool;
    fn key(&mut self, k: &str) -> bool;
    fn start_object(&mut self) -> bool;
    fn end_object(&mut self, member_count: usize) -> bool;
    fn start_array(&mut self) -> bool;
    fn end_array(&mut self, element_count: usize) -> bool;
}

pub struct Reader {/* 内部状态，略 */}

impl Reader {
    pub fn parse<H: Handler>(
        &mut self,
        input: &mut dyn crate::stream::InputStream,
        handler: &mut H,
        config: ParseConfig,
    ) -> Result<(), crate::error::Error> {
        // 仅示意
        unimplemented!()
    }
}
```

> 以上仅为接口方向示例，具体字段与配置类型将在实现阶段细化。

### 3.3 与其他 feature 的接口关系

**使用其他 feature 的接口:**

| feature ID | 接口名称 | 用途 | feature 开发设计文档追溯 |
|------------|----------|------|--------------------------|
| `FEAT-001` (`core-infra`) | 流抽象 `InputStream`/`OutputStream`、错误类型 | 从输入流读取字节、向输出流写入 JSON 文本，错误统一通过 `Error` 返回 | [core-infra-dev-design.md](../FEAT-001/core-infra-dev-design.md) |
| `FEAT-002` (`encoding-unicode`) | 编码解码接口 | 在解析/生成阶段进行编码转换与验证 | [encoding-unicode-dev-design.md](../FEAT-002/encoding-unicode-dev-design.md) |

**暴露给其他 feature 的接口（宏观）：**

| 接口名称 | 接口类型 | 暴露给的 feature | 用途 | 说明 |
|----------|----------|------------------|------|------|
| `Handler` trait | public | `dom-core`, `schema-validate`, 上层应用 | 让调用方实现自定义处理逻辑（构建 DOM、统计信息等） | DOM feature 可实现一个 Handler 以构建 Document |
| `Reader`/`Writer` | public 类型 | 上层应用、DOM | 提供解析与生成管线入口 | 调用方控制输入/输出与 Handler |

---

## 4. 数据结构

本 feature 主要通过状态机与流接口组合实现，内部数据结构（如解析状态、栈等）将在实现阶段结合 C++ `reader.h` 细化。本节**无特殊设计**。

---

## 5. 实现要点

实现要点包括：

- 解析状态机设计：如何映射 C++ Reader 状态到 Rust，实现安全/可测的状态机；
- 错误处理路径：遇到语法错误或编码错误时，及时终止解析并返回统一错误；
- 性能优化：在后续 `simd-accel` feature 中引入空白跳过等加速点；
- 与 DOM 的集成：为 DOM 构建提供专用 Handler 实现，确保行为与 C++ `Document` 的 `Parse` 路径一致。

详细算法和伪代码在本阶段不展开。本节**无特殊设计**。

---

## 变更历史

| 版本 | 日期 | 变更内容 | 变更人 |
|------|------|----------|--------|
| `0.1.0-draft` | `2026-05-14` | 初始化 FEAT-004 `sax-io` feature 级开发设计文档。 | `TBD` |
