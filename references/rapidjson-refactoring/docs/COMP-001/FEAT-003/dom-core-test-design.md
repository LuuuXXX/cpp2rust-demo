# Feature 级测试设计文档

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
| component 设计追溯 | [`rapidjson-rs-test-design.md`](../rapidjson-rs-test-design.md) |
| 对应开发文档 | [`dom-core-dev-design.md` 开发设计文档](./dom-core-dev-design.md) |
| 是否商用代码 | `是` |
| 依赖使用策略 | `core/std-only`（禁止 crates.io 第三方依赖） |

---

## 目录

- [1. 测试概述](#1-测试概述)
- [2. 基线层](#2-基线层)
- [3. 镜像测试层](#3-镜像测试层)
- [4. 孪生测试层](#4-孪生测试层)

---

## 1. 测试概述

### 1.0 商用代码与测试依赖约束

| 字段 | 内容 |
|------|------|
| 是否商用代码 | `是` |
| 允许依赖范围 | `core/std-only`（禁止 crates.io 第三方依赖） |
| crates.io 使用策略 | DOM 测试仅使用 gtest 与 `cargo test`，不引入第三方 Rust 测试/属性测试/快照测试框架。 |
| 对当前 feature 技术选型的影响 | DOM 行为验证应完全基于 legacy gtest 与 Rust 自测，不依赖其他 JSON/序列化库进行“交叉验证”，避免引入额外依赖。 |

### 1.1 测试目标

**本 feature 测试重点:**
- 验证 `Value`/`Document` 的值类型表示与访问接口行为与 C++ RapidJSON 等价，包括所有类型/边界条件。
- 验证对象/数组增删改查、深拷贝和值比较行为，与 `documenttest.cpp`/`valuetest.cpp` 中的覆盖一致。
- 验证 DOM 层在解析错误和内存错误情况下的状态不变性（如 `Document.UnchangedOnParseError` 行为）。

| 范围类别 | 内容说明 | 备注 |
|----------|----------|------|
| 包含范围 | DOM 值模型与 Document 行为（类型、数值、对象、数组、字符串、深拷贝、比较等） | 对应需求文档 2.3 DOM 相关需求 |
| 排除范围 | 具体解析/生成行为（Reader/Writer、SAX 事件） | 由 SAX/解析器 feature 测试承担 |

### 1.2 测试工具链

| 工具 | 用途 | 版本 |
|------|------|------|
| C/C++ gtest | 执行 `documenttest.cpp`、`valuetest.cpp` 中 DOM 相关测试 | 与 legacy RapidJSON 项目一致 |
| C/C++ 构建系统（如 CMake） | 构建 legacy DOM 实现与相关测试 | 同 legacy 项目 |
| `cargo test` | 执行 Rust DOM 镜像与孪生测试 | 与 workspace Rust 版本一致 |
| Python 3 | 如需，可编写辅助脚本生成结构化比较报告 | 可选 |

### 1.3 测试环境

与 component 级测试环境一致：使用同一编译器、平台与构建配置。当前 feature 不增加额外环境要求。本节**无特殊设计**。

### 1.4 三层测试防护网摘要

| 层级 | 层级名称 | 当前 feature 覆盖说明 | 关联章节 |
|------|----------|-------------------|----------|
| L0 | 基线层 | 使用 `documenttest.cpp` 与 `valuetest.cpp` 冻结 C++ DOM 行为，作为 DOM 行为基线。 | [2](#2-基线层) |
| L1 | 镜像测试层 | 为上述 gtest 编写 Rust 镜像测试，通过 FFI 调用 C++ DOM 实现。 | [3](#3-镜像测试层) |
| L2 | 孪生测试层 | 基于 `rapidjson-rs` DOM 实现编写纯 Rust 测试，与镜像测试并跑，验证行为一致。 | [4](#4-孪生测试层) |

测试断言来源遵循 component 级约束：`legacy_test` 为主，必要时结合 `spec`/`invariant`/`metamorphic`，例如深拷贝与比较的等价性质。

---

## 2. 基线层

### 2.1 基线层结构

```text
rapidjson-refactoring/
├── rapidjson_legacy/
│   └── test/unittest/
│       ├── documenttest.cpp                      # Document 行为测试
│       └── valuetest.cpp                         # Value 行为测试
│
├── inventory/
│   └── dom-core.legacy_tests.json                # DOM 相关 Legacy 测试资产清单
│
├── baseline/
│   └── dom-core.golden_samples.jsonl             # DOM 行为黄金样本（结构化输入/输出）
│
└── reports/
    ├── dom-core.legacy.junit.xml                 # Legacy DOM 测试执行结果
    └── dom-core.legacy.coverage.xml              # Legacy DOM 覆盖率报告
```

### 2.2 基线测试清单

| 测试 ID | file | oracle 来源 |
|---------|------|-------------|
| Document.AcceptWriter | test/unittest/documenttest.cpp:361 | legacy_test |
| Document.AssertAcceptInvalidNameType | test/unittest/documenttest.cpp:390 | legacy_test |
| Document.CrtAllocator | test/unittest/valuetest.cpp:1703 | legacy_test |
| Document.Parse | test/unittest/documenttest.cpp:120 | legacy_test |
| Document.Parse_Encoding | test/unittest/documenttest.cpp:174 | legacy_test |
| Document.ParseStream_AutoUTFInputStream | test/unittest/documenttest.cpp:256 | legacy_test |
| Document.ParseStream_EncodedInputStream | test/unittest/documenttest.cpp:215 | legacy_test |
| Document.Swap | test/unittest/documenttest.cpp:293 | legacy_test |
| Document.UnchangedOnParseError | test/unittest/documenttest.cpp:127 | legacy_test |
| Document.UserBuffer | test/unittest/documenttest.cpp:372 | legacy_test |
| Document.UTF16_Document | test/unittest/documenttest.cpp:402 | legacy_test |
| DocumentMove/0.MoveAssignment | test/unittest/documenttest.cpp:563 | legacy_test |
| DocumentMove/0.MoveAssignmentParseError | test/unittest/documenttest.cpp:599 | legacy_test |
| DocumentMove/0.MoveConstructor | test/unittest/documenttest.cpp:466 | legacy_test |
| DocumentMove/0.MoveConstructorParseError | test/unittest/documenttest.cpp:500 | legacy_test |
| DocumentMove/1.MoveAssignment | test/unittest/documenttest.cpp:563 | legacy_test |
| DocumentMove/1.MoveAssignmentParseError | test/unittest/documenttest.cpp:599 | legacy_test |
| DocumentMove/1.MoveConstructor | test/unittest/documenttest.cpp:466 | legacy_test |
| DocumentMove/1.MoveConstructorParseError | test/unittest/documenttest.cpp:500 | legacy_test |
| Value.AcceptTerminationByHandler | test/unittest/valuetest.cpp:1761 | legacy_test |
| Value.AllocateShortString | test/unittest/valuetest.cpp:1725 | legacy_test |
| Value.Array | test/unittest/valuetest.cpp:1080 | legacy_test |
| Value.ArrayHelper | test/unittest/valuetest.cpp:1134 | legacy_test |
| Value.ArrayHelperRangeFor | test/unittest/valuetest.cpp:1196 | legacy_test |
| Value.AssignmentOperator | test/unittest/valuetest.cpp:121 | legacy_test |
| Value.BigNestedArray | test/unittest/valuetest.cpp:1627 | legacy_test |
| Value.BigNestedObject | test/unittest/valuetest.cpp:1648 | legacy_test |
| Value.CopyFrom | test/unittest/valuetest.cpp:283 | legacy_test |
| Value.DefaultConstructor | test/unittest/valuetest.cpp:38 | legacy_test |
| Value.Double | test/unittest/valuetest.cpp:628 | legacy_test |
| Value.EqualtoOperator | test/unittest/valuetest.cpp:180 | legacy_test |
| Value.EraseMember_String | test/unittest/valuetest.cpp:1609 | legacy_test |
| Value.False | test/unittest/valuetest.cpp:358 | legacy_test |
| Value.Float | test/unittest/valuetest.cpp:660 | legacy_test |
| Value.Int | test/unittest/valuetest.cpp:384 | legacy_test |
| Value.Int64 | test/unittest/valuetest.cpp:512 | legacy_test |
| Value.IsLosslessDouble | test/unittest/valuetest.cpp:697 | legacy_test |
| Value.IsLosslessFloat | test/unittest/valuetest.cpp:722 | legacy_test |
| Value.MergeDuplicateKey | test/unittest/valuetest.cpp:1830 | legacy_test |
| Value.MoveConstructor | test/unittest/valuetest.cpp:96 | legacy_test |
| Value.Null | test/unittest/valuetest.cpp:304 | legacy_test |
| Value.Object | test/unittest/valuetest.cpp:1493 | legacy_test |
| Value.ObjectHelper | test/unittest/valuetest.cpp:1515 | legacy_test |
| Value.ObjectHelperRangeFor | test/unittest/valuetest.cpp:1571 | legacy_test |
| Value.RemoveLastElement | test/unittest/valuetest.cpp:1690 | legacy_test |
| Value.SetStringNull | test/unittest/valuetest.cpp:882 | legacy_test |
| Value.Size | test/unittest/valuetest.cpp:26 | legacy_test |
| Value.Sorting | test/unittest/valuetest.cpp:1786 | legacy_test |
| Value.SSOMemoryOverlapTest | test/unittest/valuetest.cpp:1860 | legacy_test |
| Value.String | test/unittest/valuetest.cpp:732 | legacy_test |
| Value.Swap | test/unittest/valuetest.cpp:288 | legacy_test |
| Value.True | test/unittest/valuetest.cpp:327 | legacy_test |
| Value.Uint | test/unittest/valuetest.cpp:455 | legacy_test |
| Value.Uint64 | test/unittest/valuetest.cpp:576 | legacy_test |## 3. 镜像测试层

### 3.1 镜像测试层结构

```text
rapidjson-refactoring/
├── rapidjson_refactoring_sys/
│   └── dom_ffi/                                   # DOM 相关 FFI 适配代码（规划）
│
├── migrations/
│   └── dom-core.legacy_to_mirror.json             # gtest -> Rust 镜像测试映射表
│
├── reports/
│   └── dom-core.mirror.junit.xml                  # 镜像层执行结果
│
└── rapidjson-rs/
    └── tests/
        └── mirrors/
            └── dom_core_mirror.rs                 # DOM 的 Rust 镜像测试源文件
```

### 3.2 镜像测试清单

| 测试 ID | 涉及 C/C++ 数据结构/接口 | 镜像测试 ID（Rust Test 路径） | 备注 |
|---------|-------------------------|------------------------------|------|
| Document.AcceptWriter | test/unittest/documenttest.cpp:361 | dom_core_mirror::Document_AcceptWriter | |
| Document.AssertAcceptInvalidNameType | test/unittest/documenttest.cpp:390 | dom_core_mirror::Document_AssertAcceptInvalidNameType | |
| Document.CrtAllocator | test/unittest/valuetest.cpp:1703 | dom_core_mirror::Document_CrtAllocator | |
| Document.Parse | test/unittest/documenttest.cpp:120 | dom_core_mirror::Document_Parse | |
| Document.Parse_Encoding | test/unittest/documenttest.cpp:174 | dom_core_mirror::Document_Parse_Encoding | |
| Document.ParseStream_AutoUTFInputStream | test/unittest/documenttest.cpp:256 | dom_core_mirror::Document_ParseStream_AutoUTFInputStream | |
| Document.ParseStream_EncodedInputStream | test/unittest/documenttest.cpp:215 | dom_core_mirror::Document_ParseStream_EncodedInputStream | |
| Document.Swap | test/unittest/documenttest.cpp:293 | dom_core_mirror::Document_Swap | |
| Document.UnchangedOnParseError | test/unittest/documenttest.cpp:127 | dom_core_mirror::Document_UnchangedOnParseError | |
| Document.UserBuffer | test/unittest/documenttest.cpp:372 | dom_core_mirror::Document_UserBuffer | |
| Document.UTF16_Document | test/unittest/documenttest.cpp:402 | dom_core_mirror::Document_UTF16_Document | |
| DocumentMove/0.MoveAssignment | test/unittest/documenttest.cpp:563 | dom_core_mirror::DocumentMove_0_MoveAssignment | |
| DocumentMove/0.MoveAssignmentParseError | test/unittest/documenttest.cpp:599 | dom_core_mirror::DocumentMove_0_MoveAssignmentParseError | |
| DocumentMove/0.MoveConstructor | test/unittest/documenttest.cpp:466 | dom_core_mirror::DocumentMove_0_MoveConstructor | |
| DocumentMove/0.MoveConstructorParseError | test/unittest/documenttest.cpp:500 | dom_core_mirror::DocumentMove_0_MoveConstructorParseError | |
| DocumentMove/1.MoveAssignment | test/unittest/documenttest.cpp:563 | dom_core_mirror::DocumentMove_1_MoveAssignment | |
| DocumentMove/1.MoveAssignmentParseError | test/unittest/documenttest.cpp:599 | dom_core_mirror::DocumentMove_1_MoveAssignmentParseError | |
| DocumentMove/1.MoveConstructor | test/unittest/documenttest.cpp:466 | dom_core_mirror::DocumentMove_1_MoveConstructor | |
| DocumentMove/1.MoveConstructorParseError | test/unittest/documenttest.cpp:500 | dom_core_mirror::DocumentMove_1_MoveConstructorParseError | |
| Value.AcceptTerminationByHandler | test/unittest/valuetest.cpp:1761 | dom_core_mirror::Value_AcceptTerminationByHandler | |
| Value.AllocateShortString | test/unittest/valuetest.cpp:1725 | dom_core_mirror::Value_AllocateShortString | |
| Value.Array | test/unittest/valuetest.cpp:1080 | dom_core_mirror::Value_Array | |
| Value.ArrayHelper | test/unittest/valuetest.cpp:1134 | dom_core_mirror::Value_ArrayHelper | |
| Value.ArrayHelperRangeFor | test/unittest/valuetest.cpp:1196 | dom_core_mirror::Value_ArrayHelperRangeFor | |
| Value.AssignmentOperator | test/unittest/valuetest.cpp:121 | dom_core_mirror::Value_AssignmentOperator | |
| Value.BigNestedArray | test/unittest/valuetest.cpp:1627 | dom_core_mirror::Value_BigNestedArray | |
| Value.BigNestedObject | test/unittest/valuetest.cpp:1648 | dom_core_mirror::Value_BigNestedObject | |
| Value.CopyFrom | test/unittest/valuetest.cpp:283 | dom_core_mirror::Value_CopyFrom | |
| Value.DefaultConstructor | test/unittest/valuetest.cpp:38 | dom_core_mirror::Value_DefaultConstructor | |
| Value.Double | test/unittest/valuetest.cpp:628 | dom_core_mirror::Value_Double | |
| Value.EqualtoOperator | test/unittest/valuetest.cpp:180 | dom_core_mirror::Value_EqualtoOperator | |
| Value.EraseMember_String | test/unittest/valuetest.cpp:1609 | dom_core_mirror::Value_EraseMember_String | |
| Value.False | test/unittest/valuetest.cpp:358 | dom_core_mirror::Value_False | |
| Value.Float | test/unittest/valuetest.cpp:660 | dom_core_mirror::Value_Float | |
| Value.Int | test/unittest/valuetest.cpp:384 | dom_core_mirror::Value_Int | |
| Value.Int64 | test/unittest/valuetest.cpp:512 | dom_core_mirror::Value_Int64 | |
| Value.IsLosslessDouble | test/unittest/valuetest.cpp:697 | dom_core_mirror::Value_IsLosslessDouble | |
| Value.IsLosslessFloat | test/unittest/valuetest.cpp:722 | dom_core_mirror::Value_IsLosslessFloat | |
| Value.MergeDuplicateKey | test/unittest/valuetest.cpp:1830 | dom_core_mirror::Value_MergeDuplicateKey | |
| Value.MoveConstructor | test/unittest/valuetest.cpp:96 | dom_core_mirror::Value_MoveConstructor | |
| Value.Null | test/unittest/valuetest.cpp:304 | dom_core_mirror::Value_Null | |
| Value.Object | test/unittest/valuetest.cpp:1493 | dom_core_mirror::Value_Object | |
| Value.ObjectHelper | test/unittest/valuetest.cpp:1515 | dom_core_mirror::Value_ObjectHelper | |
| Value.ObjectHelperRangeFor | test/unittest/valuetest.cpp:1571 | dom_core_mirror::Value_ObjectHelperRangeFor | |
| Value.RemoveLastElement | test/unittest/valuetest.cpp:1690 | dom_core_mirror::Value_RemoveLastElement | |
| Value.SetStringNull | test/unittest/valuetest.cpp:882 | dom_core_mirror::Value_SetStringNull | |
| Value.Size | test/unittest/valuetest.cpp:26 | dom_core_mirror::Value_Size | |
| Value.Sorting | test/unittest/valuetest.cpp:1786 | dom_core_mirror::Value_Sorting | |
| Value.SSOMemoryOverlapTest | test/unittest/valuetest.cpp:1860 | dom_core_mirror::Value_SSOMemoryOverlapTest | |
| Value.String | test/unittest/valuetest.cpp:732 | dom_core_mirror::Value_String | |
| Value.Swap | test/unittest/valuetest.cpp:288 | dom_core_mirror::Value_Swap | |
| Value.True | test/unittest/valuetest.cpp:327 | dom_core_mirror::Value_True | |
| Value.Uint | test/unittest/valuetest.cpp:455 | dom_core_mirror::Value_Uint | |
| Value.Uint64 | test/unittest/valuetest.cpp:576 | dom_core_mirror::Value_Uint64 | |#### 3.3 迁移策略建议

| 测试 ID | 进入 L1 优先级建议 | 原因 |
|---------|--------------------|------|
| `Value.*` | high | 直接定义 DOM 行为的根基，应优先构建镜像测试。 |
| `Document.Parse` | high | 最常用的 DOM 构造入口，影响多数用例。 |
| `Document.UnchangedOnParseError` | high | 确保错误场景下 DOM 不被污染，是安全性关键。 |
| `DocumentMove/*` | medium | 移动语义重要但对多数业务路径非关键，可在核心行为稳定后迁移。 |

### 3.4 镜像测试层交付件

| 交付件 | 说明 |
|--------|------|
| `reports/dom-core.mirror.junit.xml` | 镜像层执行结果。 |
| `migrations/dom-core.legacy_to_mirror.json` | gtest -> Rust 镜像测试映射表。 |
| `rapidjson-rs/tests/mirrors/dom_core_mirror.rs` | DOM 镜像测试源文件。 |

---

## 4. 孪生测试层

### 4.1 孪生测试层结构

```text
rapidjson-refactoring/
├── reports/
│   ├── dom-core.rust.junit.xml                    # 孪生层执行结果（Rust DOM 实现）
│   └── dom-core.parity.json                       # 镜像 vs 孪生 一致性报告
│
├── migrations/
│   └── dom-core.mirror_to_rust.json               # 镜像测试 -> 孪生测试映射表
│
└── rapidjson-rs/
    └── tests/
        └── rust/
            └── dom_core.rs                        # DOM 的 Rust 原生测试源文件
```

### 4.2 孪生测试清单

| 测试 ID | 镜像测试 ID | 涉及 Rust 数据结构/接口 | 原生测试 ID（Test 路径） | 是否验收 | 备注 |
|---------|-------------|------------------------|---------------------------|----------|------|
| Document.AcceptWriter | dom_core_mirror::Document_AcceptWriter | dom_core | dom_core::Document_AcceptWriter_rust | | |
| Document.AssertAcceptInvalidNameType | dom_core_mirror::Document_AssertAcceptInvalidNameType | dom_core | dom_core::Document_AssertAcceptInvalidNameType_rust | | |
| Document.CrtAllocator | dom_core_mirror::Document_CrtAllocator | dom_core | dom_core::Document_CrtAllocator_rust | | |
| Document.Parse | dom_core_mirror::Document_Parse | dom_core | dom_core::Document_Parse_rust | | |
| Document.Parse_Encoding | dom_core_mirror::Document_Parse_Encoding | dom_core | dom_core::Document_Parse_Encoding_rust | | |
| Document.ParseStream_AutoUTFInputStream | dom_core_mirror::Document_ParseStream_AutoUTFInputStream | dom_core | dom_core::Document_ParseStream_AutoUTFInputStream_rust | | |
| Document.ParseStream_EncodedInputStream | dom_core_mirror::Document_ParseStream_EncodedInputStream | dom_core | dom_core::Document_ParseStream_EncodedInputStream_rust | | |
| Document.Swap | dom_core_mirror::Document_Swap | dom_core | dom_core::Document_Swap_rust | | |
| Document.UnchangedOnParseError | dom_core_mirror::Document_UnchangedOnParseError | dom_core | dom_core::Document_UnchangedOnParseError_rust | | |
| Document.UserBuffer | dom_core_mirror::Document_UserBuffer | dom_core | dom_core::Document_UserBuffer_rust | | |
| Document.UTF16_Document | dom_core_mirror::Document_UTF16_Document | dom_core | dom_core::Document_UTF16_Document_rust | | |
| DocumentMove/0.MoveAssignment | dom_core_mirror::DocumentMove_0_MoveAssignment | dom_core | dom_core::DocumentMove_0_MoveAssignment_rust | | |
| DocumentMove/0.MoveAssignmentParseError | dom_core_mirror::DocumentMove_0_MoveAssignmentParseError | dom_core | dom_core::DocumentMove_0_MoveAssignmentParseError_rust | | |
| DocumentMove/0.MoveConstructor | dom_core_mirror::DocumentMove_0_MoveConstructor | dom_core | dom_core::DocumentMove_0_MoveConstructor_rust | | |
| DocumentMove/0.MoveConstructorParseError | dom_core_mirror::DocumentMove_0_MoveConstructorParseError | dom_core | dom_core::DocumentMove_0_MoveConstructorParseError_rust | | |
| DocumentMove/1.MoveAssignment | dom_core_mirror::DocumentMove_1_MoveAssignment | dom_core | dom_core::DocumentMove_1_MoveAssignment_rust | | |
| DocumentMove/1.MoveAssignmentParseError | dom_core_mirror::DocumentMove_1_MoveAssignmentParseError | dom_core | dom_core::DocumentMove_1_MoveAssignmentParseError_rust | | |
| DocumentMove/1.MoveConstructor | dom_core_mirror::DocumentMove_1_MoveConstructor | dom_core | dom_core::DocumentMove_1_MoveConstructor_rust | | |
| DocumentMove/1.MoveConstructorParseError | dom_core_mirror::DocumentMove_1_MoveConstructorParseError | dom_core | dom_core::DocumentMove_1_MoveConstructorParseError_rust | | |
| Value.AcceptTerminationByHandler | dom_core_mirror::Value_AcceptTerminationByHandler | dom_core | dom_core::Value_AcceptTerminationByHandler_rust | | |
| Value.AllocateShortString | dom_core_mirror::Value_AllocateShortString | dom_core | dom_core::Value_AllocateShortString_rust | | |
| Value.Array | dom_core_mirror::Value_Array | dom_core | dom_core::Value_Array_rust | | |
| Value.ArrayHelper | dom_core_mirror::Value_ArrayHelper | dom_core | dom_core::Value_ArrayHelper_rust | | |
| Value.ArrayHelperRangeFor | dom_core_mirror::Value_ArrayHelperRangeFor | dom_core | dom_core::Value_ArrayHelperRangeFor_rust | | |
| Value.AssignmentOperator | dom_core_mirror::Value_AssignmentOperator | dom_core | dom_core::Value_AssignmentOperator_rust | | |
| Value.BigNestedArray | dom_core_mirror::Value_BigNestedArray | dom_core | dom_core::Value_BigNestedArray_rust | | |
| Value.BigNestedObject | dom_core_mirror::Value_BigNestedObject | dom_core | dom_core::Value_BigNestedObject_rust | | |
| Value.CopyFrom | dom_core_mirror::Value_CopyFrom | dom_core | dom_core::Value_CopyFrom_rust | | |
| Value.DefaultConstructor | dom_core_mirror::Value_DefaultConstructor | dom_core | dom_core::Value_DefaultConstructor_rust | | |
| Value.Double | dom_core_mirror::Value_Double | dom_core | dom_core::Value_Double_rust | | |
| Value.EqualtoOperator | dom_core_mirror::Value_EqualtoOperator | dom_core | dom_core::Value_EqualtoOperator_rust | | |
| Value.EraseMember_String | dom_core_mirror::Value_EraseMember_String | dom_core | dom_core::Value_EraseMember_String_rust | | |
| Value.False | dom_core_mirror::Value_False | dom_core | dom_core::Value_False_rust | | |
| Value.Float | dom_core_mirror::Value_Float | dom_core | dom_core::Value_Float_rust | | |
| Value.Int | dom_core_mirror::Value_Int | dom_core | dom_core::Value_Int_rust | | |
| Value.Int64 | dom_core_mirror::Value_Int64 | dom_core | dom_core::Value_Int64_rust | | |
| Value.IsLosslessDouble | dom_core_mirror::Value_IsLosslessDouble | dom_core | dom_core::Value_IsLosslessDouble_rust | | |
| Value.IsLosslessFloat | dom_core_mirror::Value_IsLosslessFloat | dom_core | dom_core::Value_IsLosslessFloat_rust | | |
| Value.MergeDuplicateKey | dom_core_mirror::Value_MergeDuplicateKey | dom_core | dom_core::Value_MergeDuplicateKey_rust | | |
| Value.MoveConstructor | dom_core_mirror::Value_MoveConstructor | dom_core | dom_core::Value_MoveConstructor_rust | | |
| Value.Null | dom_core_mirror::Value_Null | dom_core | dom_core::Value_Null_rust | | |
| Value.Object | dom_core_mirror::Value_Object | dom_core | dom_core::Value_Object_rust | | |
| Value.ObjectHelper | dom_core_mirror::Value_ObjectHelper | dom_core | dom_core::Value_ObjectHelper_rust | | |
| Value.ObjectHelperRangeFor | dom_core_mirror::Value_ObjectHelperRangeFor | dom_core | dom_core::Value_ObjectHelperRangeFor_rust | | |
| Value.RemoveLastElement | dom_core_mirror::Value_RemoveLastElement | dom_core | dom_core::Value_RemoveLastElement_rust | | |
| Value.SetStringNull | dom_core_mirror::Value_SetStringNull | dom_core | dom_core::Value_SetStringNull_rust | | |
| Value.Size | dom_core_mirror::Value_Size | dom_core | dom_core::Value_Size_rust | | |
| Value.Sorting | dom_core_mirror::Value_Sorting | dom_core | dom_core::Value_Sorting_rust | | |
| Value.SSOMemoryOverlapTest | dom_core_mirror::Value_SSOMemoryOverlapTest | dom_core | dom_core::Value_SSOMemoryOverlapTest_rust | | |
| Value.String | dom_core_mirror::Value_String | dom_core | dom_core::Value_String_rust | | |
| Value.Swap | dom_core_mirror::Value_Swap | dom_core | dom_core::Value_Swap_rust | | |
| Value.True | dom_core_mirror::Value_True | dom_core | dom_core::Value_True_rust | | |
| Value.Uint | dom_core_mirror::Value_Uint | dom_core | dom_core::Value_Uint_rust | | |
| Value.Uint64 | dom_core_mirror::Value_Uint64 | dom_core | dom_core::Value_Uint64_rust | | |#### 4.3 迁移策略建议

| 测试 ID | 进入 L2 优先级建议 | 原因 |
|---------|--------------------|------|
| `Value.*` | high | DOM 值行为核心，应尽早在 Rust 实现上完成孪生验证。 |
| `Document.Parse` | high | 解析入口，覆盖多数常见用例。 |
| `Document.UnchangedOnParseError` | high | 错误场景安全关键。 |
| `Document.Swap`/`DocumentMove/*` | medium | 语义重要，但主要影响所有权与性能，不直接影响解析结果正确性。 |

### 4.4 孪生测试层交付件

| 交付件 | 说明 |
|--------|------|
| `reports/dom-core.rust.junit.xml` | 孪生层执行结果。 |
| `reports/dom-core.parity.json` | 镜像 vs 孪生 一致性报告。 |
| `migrations/dom-core.mirror_to_rust.json` | 镜像测试 -> 孪生测试映射表。 |
| `rapidjson-rs/tests/rust/dom_core.rs` | DOM 孪生测试源文件。 |

---

## 变更历史

| 版本 | 日期 | 变更内容 | 变更人 |
|------|------|----------|--------|
| `0.1.0-draft` | `2026-05-14` | 初始化 FEAT-003 `dom-core` feature 级测试设计文档。 | `TBD` |
