# Feature 级测试设计文档

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
| component 设计追溯 | [`rapidjson-rs-test-design.md`](../rapidjson-rs-test-design.md) |
| 对应开发文档 | [`pointer-path-dev-design.md` 开发设计文档](./pointer-path-dev-design.md) |
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
| crates.io 使用策略 | Pointer 测试不引入任何第三方 Rust 测试/属性测试框架，全部基于 gtest 与 `cargo test`。 |
| 对当前 feature 技术选型的影响 | Pointer 行为验证只能依赖 DOM 与自身的 Rust 实现，禁止通过其他 JSON 库或高阶测试框架作为“oracle”。 |

### 1.1 测试目标

**本 feature 测试重点:**
- 验证 JSON Pointer 解析、访问、创建、删除、交换等行为与 C++ 实现等价，包括所有边界与错误场景；
- 验证 URI Fragment 形式 Pointer 的解析与序列化；
- 验证错误报告是否包含正确的错误类型与偏移信息。

| 范围类别 | 内容说明 | 备注 |
|----------|----------|------|
| 包含范围 | 所有 JSON Pointer 功能（解析、Get/Set/Create/Erase/Swap、URI Fragment、错误报告） | 对应需求文档 2.5 JSON Pointer |
| 排除范围 | DOM 本身的行为（由 dom-core feature 测试），仅通过 Pointer 访问 DOM |

### 1.2 测试工具链

| 工具 | 用途 | 版本 |
|------|------|------|
| C/C++ gtest | 执行 `pointertest.cpp` 中 Pointer 相关测试 | 与 legacy RapidJSON 项目一致 |
| C/C++ 构建系统（如 CMake） | 构建 legacy Pointer 实现与测试 | 同 legacy 项目 |
| `cargo test` | 执行 Rust Pointer 镜像与孪生测试 | 与 workspace Rust 版本一致 |
| Python 3 | 如需，可编写辅助脚本生成 Pointer 测试用例或 diff 报告 | 可选 |

### 1.3 测试环境

与 component 级测试环境保持一致，本 feature 无额外环境约束。本节**无特殊设计**。

### 1.4 三层测试防护网摘要

| 层级 | 层级名称 | 当前 feature 覆盖说明 | 关联章节 |
|------|----------|-------------------|----------|
| L0 | 基线层 | 使用 `pointertest.cpp` 冻结 C++ Pointer 行为，涵盖解析、访问、错误路径等。 | [2](#2-基线层) |
| L1 | 镜像测试层 | 为上述 gtest 编写 Rust 镜像测试，通过 FFI 调用 C++ Pointer 实现。 | [3](#3-镜像测试层) |
| L2 | 孪生测试层 | 基于 `rapidjson-rs` Pointer 实现编写纯 Rust 测试，与镜像测试并跑验证行为一致。 | [4](#4-孪生测试层) |

---

## 2. 基线层

### 2.1 基线层结构

```text
rapidjson-refactoring/
├── rapidjson_legacy/
│   └── test/unittest/
│       └── pointertest.cpp                       # Pointer 行为测试
│
├── inventory/
│   └── pointer-path.legacy_tests.json            # Pointer 相关 Legacy 测试资产清单
│
├── baseline/
│   └── pointer-path.golden_samples.jsonl         # Pointer 行为黄金样本（路径 vs DOM 结果）
│
└── reports/
    ├── pointer-path.legacy.junit.xml             # Legacy Pointer 测试执行结果
    └── pointer-path.legacy.coverage.xml          # Legacy Pointer 覆盖率报告
```

### 2.2 基线测试清单

| 测试 ID | file | oracle 来源 |
|---------|------|-------------|
| Pointer.Ambiguity | test/unittest/pointertest.cpp:1562 | legacy_test |
| Pointer.Append | test/unittest/pointertest.cpp:567 | legacy_test |
| Pointer.Assignment | test/unittest/pointertest.cpp:497 | legacy_test |
| Pointer.ConstructorWithToken | test/unittest/pointertest.cpp:455 | legacy_test |
| Pointer.CopyConstructor | test/unittest/pointertest.cpp:466 | legacy_test |
| Pointer.Create | test/unittest/pointertest.cpp:613 | legacy_test |
| Pointer.CreateValueByPointer | test/unittest/pointertest.cpp:1011 | legacy_test |
| Pointer.CreateValueByPointer_NoAllocator | test/unittest/pointertest.cpp:1025 | legacy_test |
| Pointer.DefaultConstructor | test/unittest/pointertest.cpp:38 | legacy_test |
| Pointer.Equality | test/unittest/pointertest.cpp:597 | legacy_test |
| Pointer.Erase | test/unittest/pointertest.cpp:972 | legacy_test |
| Pointer.EraseValueByPointer_Pointer | test/unittest/pointertest.cpp:1532 | legacy_test |
| Pointer.EraseValueByPointer_String | test/unittest/pointertest.cpp:1547 | legacy_test |
| Pointer.Get | test/unittest/pointertest.cpp:699 | legacy_test |
| Pointer.GetUri | test/unittest/pointertest.cpp:663 | legacy_test |
| Pointer.GetValueByPointer | test/unittest/pointertest.cpp:1038 | legacy_test |
| Pointer.GetValueByPointerWithDefault_Pointer | test/unittest/pointertest.cpp:1071 | legacy_test |
| Pointer.GetValueByPointerWithDefault_Pointer_NoAllocator | test/unittest/pointertest.cpp:1177 | legacy_test |
| Pointer.GetValueByPointerWithDefault_String | test/unittest/pointertest.cpp:1124 | legacy_test |
| Pointer.GetValueByPointerWithDefault_String_NoAllocator | test/unittest/pointertest.cpp:1229 | legacy_test |
| Pointer.GetWithDefault | test/unittest/pointertest.cpp:731 | legacy_test |
| Pointer.GetWithDefault_NoAllocator | test/unittest/pointertest.cpp:784 | legacy_test |
| Pointer.Inequality | test/unittest/pointertest.cpp:605 | legacy_test |
| Pointer.Issue1899 | test/unittest/pointertest.cpp:1721 | legacy_test |
| Pointer.Issue483 | test/unittest/pointertest.cpp:1713 | legacy_test |
| Pointer.LessThan | test/unittest/pointertest.cpp:1613 | legacy_test |
| Pointer.Parse | test/unittest/pointertest.cpp:44 | legacy_test |
| Pointer.Parse_URIFragment | test/unittest/pointertest.cpp:193 | legacy_test |
| Pointer.ResolveOnArray | test/unittest/pointertest.cpp:1597 | legacy_test |
| Pointer.ResolveOnObject | test/unittest/pointertest.cpp:1581 | legacy_test |
| Pointer.Set | test/unittest/pointertest.cpp:836 | legacy_test |
| Pointer.Set_NoAllocator | test/unittest/pointertest.cpp:896 | legacy_test |
| Pointer.SetValueByPointer_Pointer | test/unittest/pointertest.cpp:1281 | legacy_test |
| Pointer.SetValueByPointer_Pointer_NoAllocator | test/unittest/pointertest.cpp:1395 | legacy_test |
| Pointer.SetValueByPointer_String | test/unittest/pointertest.cpp:1338 | legacy_test |
| Pointer.SetValueByPointer_String_NoAllocator | test/unittest/pointertest.cpp:1451 | legacy_test |
| Pointer.Stringify | test/unittest/pointertest.cpp:404 | legacy_test |
| Pointer.Swap | test/unittest/pointertest.cpp:537 | legacy_test |
| Pointer.Swap_Value | test/unittest/pointertest.cpp:955 | legacy_test |
| Pointer.Swap_Value_NoAllocator | test/unittest/pointertest.cpp:964 | legacy_test |
| Pointer.SwapValueByPointer | test/unittest/pointertest.cpp:1507 | legacy_test |
| Pointer.SwapValueByPointer_NoAllocator | test/unittest/pointertest.cpp:1520 | legacy_test |## 3. 镜像测试层

### 3.1 镜像测试层结构

```text
rapidjson-refactoring/
├── rapidjson_refactoring_sys/
│   └── pointer_ffi/                               # Pointer 相关 FFI 适配代码（规划）
│
├── migrations/
│   └── pointer-path.legacy_to_mirror.json         # gtest -> Rust 镜像测试映射表
│
├── reports/
│   └── pointer-path.mirror.junit.xml              # 镜像层执行结果
│
└── rapidjson-rs/
    └── tests/
        └── mirrors/
            └── pointer_path_mirror.rs             # Pointer 的 Rust 镜像测试源文件
```

### 3.2 镜像测试清单

| 测试 ID | 涉及 C/C++ 数据结构/接口 | 镜像测试 ID（Rust Test 路径） | 备注 |
|---------|-------------------------|------------------------------|------|
| Pointer.Ambiguity | test/unittest/pointertest.cpp:1562 | pointer_path_mirror::Pointer_Ambiguity | |
| Pointer.Append | test/unittest/pointertest.cpp:567 | pointer_path_mirror::Pointer_Append | |
| Pointer.Assignment | test/unittest/pointertest.cpp:497 | pointer_path_mirror::Pointer_Assignment | |
| Pointer.ConstructorWithToken | test/unittest/pointertest.cpp:455 | pointer_path_mirror::Pointer_ConstructorWithToken | |
| Pointer.CopyConstructor | test/unittest/pointertest.cpp:466 | pointer_path_mirror::Pointer_CopyConstructor | |
| Pointer.Create | test/unittest/pointertest.cpp:613 | pointer_path_mirror::Pointer_Create | |
| Pointer.CreateValueByPointer | test/unittest/pointertest.cpp:1011 | pointer_path_mirror::Pointer_CreateValueByPointer | |
| Pointer.CreateValueByPointer_NoAllocator | test/unittest/pointertest.cpp:1025 | pointer_path_mirror::Pointer_CreateValueByPointer_NoAllocator | |
| Pointer.DefaultConstructor | test/unittest/pointertest.cpp:38 | pointer_path_mirror::Pointer_DefaultConstructor | |
| Pointer.Equality | test/unittest/pointertest.cpp:597 | pointer_path_mirror::Pointer_Equality | |
| Pointer.Erase | test/unittest/pointertest.cpp:972 | pointer_path_mirror::Pointer_Erase | |
| Pointer.EraseValueByPointer_Pointer | test/unittest/pointertest.cpp:1532 | pointer_path_mirror::Pointer_EraseValueByPointer_Pointer | |
| Pointer.EraseValueByPointer_String | test/unittest/pointertest.cpp:1547 | pointer_path_mirror::Pointer_EraseValueByPointer_String | |
| Pointer.Get | test/unittest/pointertest.cpp:699 | pointer_path_mirror::Pointer_Get | |
| Pointer.GetUri | test/unittest/pointertest.cpp:663 | pointer_path_mirror::Pointer_GetUri | |
| Pointer.GetValueByPointer | test/unittest/pointertest.cpp:1038 | pointer_path_mirror::Pointer_GetValueByPointer | |
| Pointer.GetValueByPointerWithDefault_Pointer | test/unittest/pointertest.cpp:1071 | pointer_path_mirror::Pointer_GetValueByPointerWithDefault_Pointer | |
| Pointer.GetValueByPointerWithDefault_Pointer_NoAllocator | test/unittest/pointertest.cpp:1177 | pointer_path_mirror::Pointer_GetValueByPointerWithDefault_Pointer_NoAllocator | |
| Pointer.GetValueByPointerWithDefault_String | test/unittest/pointertest.cpp:1124 | pointer_path_mirror::Pointer_GetValueByPointerWithDefault_String | |
| Pointer.GetValueByPointerWithDefault_String_NoAllocator | test/unittest/pointertest.cpp:1229 | pointer_path_mirror::Pointer_GetValueByPointerWithDefault_String_NoAllocator | |
| Pointer.GetWithDefault | test/unittest/pointertest.cpp:731 | pointer_path_mirror::Pointer_GetWithDefault | |
| Pointer.GetWithDefault_NoAllocator | test/unittest/pointertest.cpp:784 | pointer_path_mirror::Pointer_GetWithDefault_NoAllocator | |
| Pointer.Inequality | test/unittest/pointertest.cpp:605 | pointer_path_mirror::Pointer_Inequality | |
| Pointer.Issue1899 | test/unittest/pointertest.cpp:1721 | pointer_path_mirror::Pointer_Issue1899 | |
| Pointer.Issue483 | test/unittest/pointertest.cpp:1713 | pointer_path_mirror::Pointer_Issue483 | |
| Pointer.LessThan | test/unittest/pointertest.cpp:1613 | pointer_path_mirror::Pointer_LessThan | |
| Pointer.Parse | test/unittest/pointertest.cpp:44 | pointer_path_mirror::Pointer_Parse | |
| Pointer.Parse_URIFragment | test/unittest/pointertest.cpp:193 | pointer_path_mirror::Pointer_Parse_URIFragment | |
| Pointer.ResolveOnArray | test/unittest/pointertest.cpp:1597 | pointer_path_mirror::Pointer_ResolveOnArray | |
| Pointer.ResolveOnObject | test/unittest/pointertest.cpp:1581 | pointer_path_mirror::Pointer_ResolveOnObject | |
| Pointer.Set | test/unittest/pointertest.cpp:836 | pointer_path_mirror::Pointer_Set | |
| Pointer.Set_NoAllocator | test/unittest/pointertest.cpp:896 | pointer_path_mirror::Pointer_Set_NoAllocator | |
| Pointer.SetValueByPointer_Pointer | test/unittest/pointertest.cpp:1281 | pointer_path_mirror::Pointer_SetValueByPointer_Pointer | |
| Pointer.SetValueByPointer_Pointer_NoAllocator | test/unittest/pointertest.cpp:1395 | pointer_path_mirror::Pointer_SetValueByPointer_Pointer_NoAllocator | |
| Pointer.SetValueByPointer_String | test/unittest/pointertest.cpp:1338 | pointer_path_mirror::Pointer_SetValueByPointer_String | |
| Pointer.SetValueByPointer_String_NoAllocator | test/unittest/pointertest.cpp:1451 | pointer_path_mirror::Pointer_SetValueByPointer_String_NoAllocator | |
| Pointer.Stringify | test/unittest/pointertest.cpp:404 | pointer_path_mirror::Pointer_Stringify | |
| Pointer.Swap | test/unittest/pointertest.cpp:537 | pointer_path_mirror::Pointer_Swap | |
| Pointer.Swap_Value | test/unittest/pointertest.cpp:955 | pointer_path_mirror::Pointer_Swap_Value | |
| Pointer.Swap_Value_NoAllocator | test/unittest/pointertest.cpp:964 | pointer_path_mirror::Pointer_Swap_Value_NoAllocator | |
| Pointer.SwapValueByPointer | test/unittest/pointertest.cpp:1507 | pointer_path_mirror::Pointer_SwapValueByPointer | |
| Pointer.SwapValueByPointer_NoAllocator | test/unittest/pointertest.cpp:1520 | pointer_path_mirror::Pointer_SwapValueByPointer_NoAllocator | |#### 3.3 迁移策略建议

| 测试 ID | 进入 L1 优先级建议 | 原因 |
|---------|--------------------|------|
| `Pointer.Parse*` | high | 所有 Pointer 行为的基础，需优先验证解析正确性。 |
| `Pointer.Create*`/`Get*`/`Set*`/`Erase*` | high | 代表核心使用场景，对上层功能影响大。 |
| `Pointer.Swap*` | medium | 重要但属于增强功能，可在核心行为稳定后迁移。 |
| `Pointer.Ambiguity`/`ResolveOn*`/`LessThan` | medium | 边界与排序行为，重要但优先级次于核心路径。 |

### 3.4 镜像测试层交付件

| 交付件 | 说明 |
|--------|------|
| `reports/pointer-path.mirror.junit.xml` | 镜像层执行结果。 |
| `migrations/pointer-path.legacy_to_mirror.json` | gtest -> Rust 镜像测试映射表。 |
| `rapidjson-rs/tests/mirrors/pointer_path_mirror.rs` | Pointer 镜像测试源文件。 |

---

## 4. 孪生测试层

### 4.1 孪生测试层结构

```text
rapidjson-refactoring/
├── reports/
│   ├── pointer-path.rust.junit.xml                # 孪生层执行结果（Rust Pointer 实现）
│   └── pointer-path.parity.json                   # 镜像 vs 孪生 一致性报告
│
├── migrations/
│   └── pointer-path.mirror_to_rust.json           # 镜像测试 -> 孪生测试映射表
│
└── rapidjson-rs/
    └── tests/
        └── rust/
            └── pointer_path.rs                    # Pointer 的 Rust 原生测试源文件
```

### 4.2 孪生测试清单

| 测试 ID | 镜像测试 ID | 涉及 Rust 数据结构/接口 | 原生测试 ID（Test 路径） | 是否验收 | 备注 |
|---------|-------------|------------------------|---------------------------|----------|------|
| Pointer.Ambiguity | pointer_path_mirror::Pointer_Ambiguity | pointer_path | pointer_path::Pointer_Ambiguity_rust | | |
| Pointer.Append | pointer_path_mirror::Pointer_Append | pointer_path | pointer_path::Pointer_Append_rust | | |
| Pointer.Assignment | pointer_path_mirror::Pointer_Assignment | pointer_path | pointer_path::Pointer_Assignment_rust | | |
| Pointer.ConstructorWithToken | pointer_path_mirror::Pointer_ConstructorWithToken | pointer_path | pointer_path::Pointer_ConstructorWithToken_rust | | |
| Pointer.CopyConstructor | pointer_path_mirror::Pointer_CopyConstructor | pointer_path | pointer_path::Pointer_CopyConstructor_rust | | |
| Pointer.Create | pointer_path_mirror::Pointer_Create | pointer_path | pointer_path::Pointer_Create_rust | | |
| Pointer.CreateValueByPointer | pointer_path_mirror::Pointer_CreateValueByPointer | pointer_path | pointer_path::Pointer_CreateValueByPointer_rust | | |
| Pointer.CreateValueByPointer_NoAllocator | pointer_path_mirror::Pointer_CreateValueByPointer_NoAllocator | pointer_path | pointer_path::Pointer_CreateValueByPointer_NoAllocator_rust | | |
| Pointer.DefaultConstructor | pointer_path_mirror::Pointer_DefaultConstructor | pointer_path | pointer_path::Pointer_DefaultConstructor_rust | | |
| Pointer.Equality | pointer_path_mirror::Pointer_Equality | pointer_path | pointer_path::Pointer_Equality_rust | | |
| Pointer.Erase | pointer_path_mirror::Pointer_Erase | pointer_path | pointer_path::Pointer_Erase_rust | | |
| Pointer.EraseValueByPointer_Pointer | pointer_path_mirror::Pointer_EraseValueByPointer_Pointer | pointer_path | pointer_path::Pointer_EraseValueByPointer_Pointer_rust | | |
| Pointer.EraseValueByPointer_String | pointer_path_mirror::Pointer_EraseValueByPointer_String | pointer_path | pointer_path::Pointer_EraseValueByPointer_String_rust | | |
| Pointer.Get | pointer_path_mirror::Pointer_Get | pointer_path | pointer_path::Pointer_Get_rust | | |
| Pointer.GetUri | pointer_path_mirror::Pointer_GetUri | pointer_path | pointer_path::Pointer_GetUri_rust | | |
| Pointer.GetValueByPointer | pointer_path_mirror::Pointer_GetValueByPointer | pointer_path | pointer_path::Pointer_GetValueByPointer_rust | | |
| Pointer.GetValueByPointerWithDefault_Pointer | pointer_path_mirror::Pointer_GetValueByPointerWithDefault_Pointer | pointer_path | pointer_path::Pointer_GetValueByPointerWithDefault_Pointer_rust | | |
| Pointer.GetValueByPointerWithDefault_Pointer_NoAllocator | pointer_path_mirror::Pointer_GetValueByPointerWithDefault_Pointer_NoAllocator | pointer_path | pointer_path::Pointer_GetValueByPointerWithDefault_Pointer_NoAllocator_rust | | |
| Pointer.GetValueByPointerWithDefault_String | pointer_path_mirror::Pointer_GetValueByPointerWithDefault_String | pointer_path | pointer_path::Pointer_GetValueByPointerWithDefault_String_rust | | |
| Pointer.GetValueByPointerWithDefault_String_NoAllocator | pointer_path_mirror::Pointer_GetValueByPointerWithDefault_String_NoAllocator | pointer_path | pointer_path::Pointer_GetValueByPointerWithDefault_String_NoAllocator_rust | | |
| Pointer.GetWithDefault | pointer_path_mirror::Pointer_GetWithDefault | pointer_path | pointer_path::Pointer_GetWithDefault_rust | | |
| Pointer.GetWithDefault_NoAllocator | pointer_path_mirror::Pointer_GetWithDefault_NoAllocator | pointer_path | pointer_path::Pointer_GetWithDefault_NoAllocator_rust | | |
| Pointer.Inequality | pointer_path_mirror::Pointer_Inequality | pointer_path | pointer_path::Pointer_Inequality_rust | | |
| Pointer.Issue1899 | pointer_path_mirror::Pointer_Issue1899 | pointer_path | pointer_path::Pointer_Issue1899_rust | | |
| Pointer.Issue483 | pointer_path_mirror::Pointer_Issue483 | pointer_path | pointer_path::Pointer_Issue483_rust | | |
| Pointer.LessThan | pointer_path_mirror::Pointer_LessThan | pointer_path | pointer_path::Pointer_LessThan_rust | | |
| Pointer.Parse | pointer_path_mirror::Pointer_Parse | pointer_path | pointer_path::Pointer_Parse_rust | | |
| Pointer.Parse_URIFragment | pointer_path_mirror::Pointer_Parse_URIFragment | pointer_path | pointer_path::Pointer_Parse_URIFragment_rust | | |
| Pointer.ResolveOnArray | pointer_path_mirror::Pointer_ResolveOnArray | pointer_path | pointer_path::Pointer_ResolveOnArray_rust | | |
| Pointer.ResolveOnObject | pointer_path_mirror::Pointer_ResolveOnObject | pointer_path | pointer_path::Pointer_ResolveOnObject_rust | | |
| Pointer.Set | pointer_path_mirror::Pointer_Set | pointer_path | pointer_path::Pointer_Set_rust | | |
| Pointer.Set_NoAllocator | pointer_path_mirror::Pointer_Set_NoAllocator | pointer_path | pointer_path::Pointer_Set_NoAllocator_rust | | |
| Pointer.SetValueByPointer_Pointer | pointer_path_mirror::Pointer_SetValueByPointer_Pointer | pointer_path | pointer_path::Pointer_SetValueByPointer_Pointer_rust | | |
| Pointer.SetValueByPointer_Pointer_NoAllocator | pointer_path_mirror::Pointer_SetValueByPointer_Pointer_NoAllocator | pointer_path | pointer_path::Pointer_SetValueByPointer_Pointer_NoAllocator_rust | | |
| Pointer.SetValueByPointer_String | pointer_path_mirror::Pointer_SetValueByPointer_String | pointer_path | pointer_path::Pointer_SetValueByPointer_String_rust | | |
| Pointer.SetValueByPointer_String_NoAllocator | pointer_path_mirror::Pointer_SetValueByPointer_String_NoAllocator | pointer_path | pointer_path::Pointer_SetValueByPointer_String_NoAllocator_rust | | |
| Pointer.Stringify | pointer_path_mirror::Pointer_Stringify | pointer_path | pointer_path::Pointer_Stringify_rust | | |
| Pointer.Swap | pointer_path_mirror::Pointer_Swap | pointer_path | pointer_path::Pointer_Swap_rust | | |
| Pointer.Swap_Value | pointer_path_mirror::Pointer_Swap_Value | pointer_path | pointer_path::Pointer_Swap_Value_rust | | |
| Pointer.Swap_Value_NoAllocator | pointer_path_mirror::Pointer_Swap_Value_NoAllocator | pointer_path | pointer_path::Pointer_Swap_Value_NoAllocator_rust | | |
| Pointer.SwapValueByPointer | pointer_path_mirror::Pointer_SwapValueByPointer | pointer_path | pointer_path::Pointer_SwapValueByPointer_rust | | |
| Pointer.SwapValueByPointer_NoAllocator | pointer_path_mirror::Pointer_SwapValueByPointer_NoAllocator | pointer_path | pointer_path::Pointer_SwapValueByPointer_NoAllocator_rust | | |#### 4.3 迁移策略建议

| 测试 ID | 进入 L2 优先级建议 | 原因 |
|---------|--------------------|------|
| `Pointer.Parse*` | high | 核心解析行为，所有操作的基础。 |
| `Pointer.Create*`/`Get*`/`Set*`/`Erase*` | high | Pointer 的主要使用场景。 |
| `Pointer.Swap*`/`Pointer.Ambiguity`/`ResolveOn*`/`LessThan` | medium | 边界和增强行为，可在核心路径稳定后迁移。 |

### 4.4 孪生测试层交付件

| 交付件 | 说明 |
|--------|------|
| `reports/pointer-path.rust.junit.xml` | 孪生层执行结果。 |
| `reports/pointer-path.parity.json` | 镜像 vs 孪生 一致性报告。 |
| `migrations/pointer-path.mirror_to_rust.json` | 镜像测试 -> 孪生测试映射表。 |
| `rapidjson-rs/tests/rust/pointer_path.rs` | Pointer 孪生测试源文件。 |

---

## 变更历史

| 版本 | 日期 | 变更内容 | 变更人 |
|------|------|----------|--------|
| `0.1.0-draft` | `2026-05-14` | 初始化 FEAT-005 `pointer-path` feature 级测试设计文档。 | `TBD` |
