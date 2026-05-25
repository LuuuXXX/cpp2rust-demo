# Feature 级测试设计文档

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
| component 设计追溯 | [`rapidjson-rs-test-design.md`](../rapidjson-rs-test-design.md) |
| 对应开发文档 | [`schema-validate-dev-design.md` 开发设计文档](./schema-validate-dev-design.md) |
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
| crates.io 使用策略 | Schema 测试不引入第三方 JSON Schema 测试工具/框架，全部基于 legacy schematest 和 Rust 自测。 |
| 对当前 feature 技术选型的影响 | 所有 Schema 校验逻辑由 `rapidjson-rs` 自身承担，测试只使用 gtest 和 `cargo test`，不依赖其他 Schema 工具作为“真值”。 |

### 1.1 测试目标

**本 feature 测试重点:**
- 验证 JSON Schema 编译与校验的行为与 C++ RapidJSON 实现等价，包括 DOM/SAX/Writer 三种模式；
- 验证错误报告内容（关键字、实例路径、Schema 路径等）与 C++ 行为一致；
- 验证远程引用、continue-on-errors、不同 draft/version 处理、Swagger/OpenAPI 扩展等高级功能行为。

| 范围类别 | 内容说明 | 备注 |
|----------|----------|------|
| 包含范围 | SchemaDocument 编译、SchemaValidator、SchemaValidatingReader/Writer 行为，以及所有 schematest 中覆盖的功能 | 对应需求文档 2.6 JSON Schema |
| 排除范围 | DOM/SAX/Pointer 的基本行为（由各自 feature 测试），Schema 测试仅在这些行为基础上进行组合验证 |  |

### 1.2 测试工具链

| 工具 | 用途 | 版本 |
|------|------|------|
| C/C++ gtest | 执行 `schematest.cpp` 中所有 Schema 相关测试 | 与 legacy RapidJSON 项目一致 |
| C/C++ 构建系统（如 CMake） | 构建 legacy Schema 实现与测试 | 同 legacy 项目 |
| `cargo test` | 执行 Rust Schema 镜像与孪生测试 | 与 workspace Rust 版本一致 |
| Python 3 | 可选，用于生成/转换 JSON Schema 测试用例 | 可选 |

### 1.3 测试环境

沿用 component 级测试环境，本 feature 不增加额外环境要求。本节**无特殊设计**。

### 1.4 三层测试防护网摘要

| 层级 | 层级名称 | 当前 feature 覆盖说明 | 关联章节 |
|------|----------|-------------------|----------|
| L0 | 基线层 | 使用 `schematest.cpp`（unittest + perftest）冻结 C++ Schema 行为，包括错误报告与性能特征。 | [2](#2-基线层) |
| L1 | 镜像测试层 | 为上述 gtest 编写 Rust 镜像测试，通过 FFI 调用 C++ Schema 实现。 | [3](#3-镜像测试层) |
| L2 | 孪生测试层 | 基于 `rapidjson-rs` Schema 实现编写纯 Rust 测试，与镜像测试并跑验证行为一致。 | [4](#4-孪生测试层) |

---

## 2. 基线层

### 2.1 基线层结构

```text
rapidjson-refactoring/
├── rapidjson_legacy/
│   └── test/
│       ├── unittest/
│       │   └── schematest.cpp                    # 单元测试：SchemaValidator, SchemaDocument 等
│       └── perftest/
│           └── schematest.cpp                    # 性能测试：Schema 相关性能与特性
│
├── inventory/
│   └── schema-validate.legacy_tests.json         # Schema 相关 Legacy 测试资产清单
│
├── baseline/
│   └── schema-validate.golden_samples.jsonl      # Schema 行为黄金样本（Schema + 实例 + 预期结果）
│
└── reports/
    ├── schema-validate.legacy.junit.xml          # Legacy Schema 测试执行结果
    └── schema-validate.legacy.coverage.xml       # Legacy Schema 覆盖率报告
```

### 2.2 基线测试清单

| 测试 ID | file | oracle 来源 |
|---------|------|-------------|
| Schema.Issue552 | test/unittest/schematest.cpp:2385 | legacy_test |
| Schema.Issue848 | test/unittest/schematest.cpp:2370 | legacy_test |
| Schema.TestSuite | test/perftest/schematest.cpp:198 | spec + c_output |
| SchemaValidatingReader.Invalid | test/unittest/schematest.cpp:2308 | legacy_test |
| SchemaValidatingReader.Simple | test/unittest/schematest.cpp:2293 | legacy_test |
| SchemaValidatingWriter.Simple | test/unittest/schematest.cpp:2337 | legacy_test |
| SchemaValidator.AllOf | test/unittest/schematest.cpp:278 | legacy_test |
| SchemaValidator.AllOf_Nested | test/unittest/schematest.cpp:1742 | legacy_test |
| SchemaValidator.AnyOf | test/unittest/schematest.cpp:311 | legacy_test |
| SchemaValidator.Array | test/unittest/schematest.cpp:1465 | legacy_test |
| SchemaValidator.Array_AdditionalItems | test/unittest/schematest.cpp:1544 | legacy_test |
| SchemaValidator.Array_ItemsList | test/unittest/schematest.cpp:1480 | legacy_test |
| SchemaValidator.Array_ItemsRange | test/unittest/schematest.cpp:1579 | legacy_test |
| SchemaValidator.Array_ItemsTuple | test/unittest/schematest.cpp:1501 | legacy_test |
| SchemaValidator.Array_UniqueItems | test/unittest/schematest.cpp:1606 | legacy_test |
| SchemaValidator.Boolean | test/unittest/schematest.cpp:1627 | legacy_test |
| SchemaValidator.ContinueOnErrors | test/unittest/schematest.cpp:2678 | legacy_test |
| SchemaValidator.ContinueOnErrors_AllOf | test/unittest/schematest.cpp:2799 | legacy_test |
| SchemaValidator.ContinueOnErrors_AnyOf | test/unittest/schematest.cpp:2825 | legacy_test |
| SchemaValidator.ContinueOnErrors_BadSimpleType | test/unittest/schematest.cpp:2950 | legacy_test |
| SchemaValidator.ContinueOnErrors_Enum | test/unittest/schematest.cpp:2875 | legacy_test |
| SchemaValidator.ContinueOnErrors_OneOf | test/unittest/schematest.cpp:2773 | legacy_test |
| SchemaValidator.ContinueOnErrors_RogueArray | test/unittest/schematest.cpp:2894 | legacy_test |
| SchemaValidator.ContinueOnErrors_RogueObject | test/unittest/schematest.cpp:2913 | legacy_test |
| SchemaValidator.ContinueOnErrors_RogueString | test/unittest/schematest.cpp:2928 | legacy_test |
| SchemaValidator.ContinueOnErrors_UniqueItems | test/unittest/schematest.cpp:2853 | legacy_test |
| SchemaValidator.DuplicateKeyword | test/unittest/schematest.cpp:2982 | legacy_test |
| SchemaValidator.Enum_InvalidType | test/unittest/schematest.cpp:264 | legacy_test |
| SchemaValidator.Enum_Typed | test/unittest/schematest.cpp:242 | legacy_test |
| SchemaValidator.Enum_Typeless | test/unittest/schematest.cpp:252 | legacy_test |
| SchemaValidator.EscapedPointer | test/unittest/schematest.cpp:1824 | legacy_test |
| SchemaValidator.Hasher | test/unittest/schematest.cpp:51 | legacy_test |
| SchemaValidator.Integer | test/unittest/schematest.cpp:541 | legacy_test |
| SchemaValidator.Integer_MultipleOf | test/unittest/schematest.cpp:706 | legacy_test |
| SchemaValidator.Integer_MultipleOf64Boundary | test/unittest/schematest.cpp:729 | legacy_test |
| SchemaValidator.Integer_Range | test/unittest/schematest.cpp:566 | legacy_test |
| SchemaValidator.Integer_Range64Boundary | test/unittest/schematest.cpp:594 | legacy_test |
| SchemaValidator.Integer_Range64BoundaryExclusive | test/unittest/schematest.cpp:683 | legacy_test |
| SchemaValidator.Integer_RangeU64Boundary | test/unittest/schematest.cpp:626 | legacy_test |
| SchemaValidator.Issue1017_allOfHandler | test/unittest/schematest.cpp:2423 | legacy_test |
| SchemaValidator.Issue608 | test/unittest/schematest.cpp:2399 | legacy_test |
| SchemaValidator.Issue728_AllOfRef | test/unittest/schematest.cpp:2414 | legacy_test |
| SchemaValidator.MultiType | test/unittest/schematest.cpp:227 | legacy_test |
| SchemaValidator.MultiTypeInObject | test/unittest/schematest.cpp:1696 | legacy_test |
| SchemaValidator.MultiTypeWithObject | test/unittest/schematest.cpp:1719 | legacy_test |
| SchemaValidator.Not | test/unittest/schematest.cpp:365 | legacy_test |
| SchemaValidator.Null | test/unittest/schematest.cpp:1648 | legacy_test |
| SchemaValidator.NullableFalse | test/unittest/schematest.cpp:3540 | legacy_test |
| SchemaValidator.NullableTrue | test/unittest/schematest.cpp:3509 | legacy_test |
| SchemaValidator.Number_MultipleOf | test/unittest/schematest.cpp:995 | legacy_test |
| SchemaValidator.Number_MultipleOfOne | test/unittest/schematest.cpp:1039 | legacy_test |
| SchemaValidator.Number_Range | test/unittest/schematest.cpp:744 | legacy_test |
| SchemaValidator.Number_RangeDouble | test/unittest/schematest.cpp:855 | legacy_test |
| SchemaValidator.Number_RangeDoubleU64Boundary | test/unittest/schematest.cpp:944 | legacy_test |
| SchemaValidator.Number_RangeInt | test/unittest/schematest.cpp:780 | legacy_test |
| SchemaValidator.Object | test/unittest/schematest.cpp:1054 | legacy_test |
| SchemaValidator.Object_AdditionalPropertiesBoolean | test/unittest/schematest.cpp:1108 | legacy_test |
| SchemaValidator.Object_AdditionalPropertiesObject | test/unittest/schematest.cpp:1134 | legacy_test |
| SchemaValidator.Object_PatternProperties | test/unittest/schematest.cpp:1322 | legacy_test |
| SchemaValidator.Object_PatternProperties_AdditionalPropertiesBoolean | test/unittest/schematest.cpp:1441 | legacy_test |
| SchemaValidator.Object_PatternProperties_AdditionalPropertiesObject | test/unittest/schematest.cpp:1414 | legacy_test |
| SchemaValidator.Object_PatternProperties_ErrorConflict | test/unittest/schematest.cpp:1351 | legacy_test |
| SchemaValidator.Object_Properties | test/unittest/schematest.cpp:1075 | legacy_test |
| SchemaValidator.Object_Properties_PatternProperties | test/unittest/schematest.cpp:1378 | legacy_test |
| SchemaValidator.Object_PropertiesRange | test/unittest/schematest.cpp:1221 | legacy_test |
| SchemaValidator.Object_PropertyDependencies | test/unittest/schematest.cpp:1248 | legacy_test |
| SchemaValidator.Object_Required | test/unittest/schematest.cpp:1160 | legacy_test |
| SchemaValidator.Object_Required_PassWithDefault | test/unittest/schematest.cpp:1191 | legacy_test |
| SchemaValidator.Object_SchemaDependencies | test/unittest/schematest.cpp:1284 | legacy_test |
| SchemaValidator.ObjectInArray | test/unittest/schematest.cpp:1676 | legacy_test |
| SchemaValidator.OneOf | test/unittest/schematest.cpp:337 | legacy_test |
| SchemaValidator.ReadOnlyWhenWriting | test/unittest/schematest.cpp:3465 | legacy_test |
| SchemaValidator.Ref | test/unittest/schematest.cpp:376 | legacy_test |
| SchemaValidator.Ref_AllOf | test/unittest/schematest.cpp:404 | legacy_test |
| SchemaValidator.Ref_internal_id_1 | test/unittest/schematest.cpp:2537 | legacy_test |
| SchemaValidator.Ref_internal_id_2 | test/unittest/schematest.cpp:2555 | legacy_test |
| SchemaValidator.Ref_internal_id_and_schema_pointer | test/unittest/schematest.cpp:2591 | legacy_test |
| SchemaValidator.Ref_internal_id_in_array | test/unittest/schematest.cpp:2573 | legacy_test |
| SchemaValidator.Ref_internal_multiple_ids | test/unittest/schematest.cpp:2610 | legacy_test |
| SchemaValidator.Ref_remote | test/unittest/schematest.cpp:2442 | legacy_test |
| SchemaValidator.Ref_remote_change_resolution_scope_absolute_path | test/unittest/schematest.cpp:2499 | legacy_test |
| SchemaValidator.Ref_remote_change_resolution_scope_absolute_path_document | test/unittest/schematest.cpp:2518 | legacy_test |
| SchemaValidator.Ref_remote_change_resolution_scope_relative_path | test/unittest/schematest.cpp:2480 | legacy_test |
| SchemaValidator.Ref_remote_change_resolution_scope_uri | test/unittest/schematest.cpp:2461 | legacy_test |
| SchemaValidator.Ref_remote_issue1210 | test/unittest/schematest.cpp:2639 | legacy_test |
| SchemaValidator.Schema_DraftAndVersion | test/unittest/schematest.cpp:3318 | legacy_test |
| SchemaValidator.Schema_IgnoreDraftEmbedded | test/unittest/schematest.cpp:3088 | legacy_test |
| SchemaValidator.Schema_MultipleErrors | test/unittest/schematest.cpp:3335 | legacy_test |
| SchemaValidator.Schema_ReadOnlyAndWriteOnly | test/unittest/schematest.cpp:3455 | legacy_test |
| SchemaValidator.Schema_RefCyclical | test/unittest/schematest.cpp:3440 | legacy_test |
| SchemaValidator.Schema_RefEmptyString | test/unittest/schematest.cpp:3365 | legacy_test |
| SchemaValidator.Schema_RefNoRemoteProvider | test/unittest/schematest.cpp:3374 | legacy_test |
| SchemaValidator.Schema_RefNoRemoteSchema | test/unittest/schematest.cpp:3383 | legacy_test |
| SchemaValidator.Schema_RefPlainNameOpenApi | test/unittest/schematest.cpp:3346 | legacy_test |
| SchemaValidator.Schema_RefPlainNameRemote | test/unittest/schematest.cpp:3355 | legacy_test |
| SchemaValidator.Schema_RefPointerInvalid | test/unittest/schematest.cpp:3393 | legacy_test |
| SchemaValidator.Schema_RefPointerInvalidRemote | test/unittest/schematest.cpp:3402 | legacy_test |
| SchemaValidator.Schema_RefUnknownPlainName | test/unittest/schematest.cpp:3412 | legacy_test |
| SchemaValidator.Schema_RefUnknownPointer | test/unittest/schematest.cpp:3421 | legacy_test |
| SchemaValidator.Schema_RefUnknownPointerRemote | test/unittest/schematest.cpp:3430 | legacy_test |
| SchemaValidator.Schema_StartUnknown | test/unittest/schematest.cpp:3327 | legacy_test |
| SchemaValidator.Schema_SupportedDraft4 | test/unittest/schematest.cpp:3044 | legacy_test |
| SchemaValidator.Schema_SupportedDraft4NoFrag | test/unittest/schematest.cpp:3055 | legacy_test |
| SchemaValidator.Schema_SupportedDraft5 | test/unittest/schematest.cpp:3066 | legacy_test |
| SchemaValidator.Schema_SupportedDraft5NoFrag | test/unittest/schematest.cpp:3077 | legacy_test |
| SchemaValidator.Schema_SupportedDraft5Static | test/unittest/schematest.cpp:3033 | legacy_test |
| SchemaValidator.Schema_SupportedDraftOverride | test/unittest/schematest.cpp:3099 | legacy_test |
| SchemaValidator.Schema_SupportedNoSpec | test/unittest/schematest.cpp:3011 | legacy_test |
| SchemaValidator.Schema_SupportedNoSpecStatic | test/unittest/schematest.cpp:3022 | legacy_test |
| SchemaValidator.Schema_SupportedNotObject | test/unittest/schematest.cpp:3000 | legacy_test |
| SchemaValidator.Schema_SupportedVersion20 | test/unittest/schematest.cpp:3219 | legacy_test |
| SchemaValidator.Schema_SupportedVersion20Static | test/unittest/schematest.cpp:3208 | legacy_test |
| SchemaValidator.Schema_SupportedVersion30x | test/unittest/schematest.cpp:3230 | legacy_test |
| SchemaValidator.Schema_SupportedVersionOverride | test/unittest/schematest.cpp:3241 | legacy_test |
| SchemaValidator.Schema_UnknownDraft | test/unittest/schematest.cpp:3132 | legacy_test |
| SchemaValidator.Schema_UnknownDraftNotString | test/unittest/schematest.cpp:3143 | legacy_test |
| SchemaValidator.Schema_UnknownDraftOverride | test/unittest/schematest.cpp:3110 | legacy_test |
| SchemaValidator.Schema_UnknownVersion | test/unittest/schematest.cpp:3274 | legacy_test |
| SchemaValidator.Schema_UnknownVersionNotString | test/unittest/schematest.cpp:3296 | legacy_test |
| SchemaValidator.Schema_UnknownVersionOverride | test/unittest/schematest.cpp:3252 | legacy_test |
| SchemaValidator.Schema_UnknownVersionShort | test/unittest/schematest.cpp:3285 | legacy_test |
| SchemaValidator.Schema_UnsupportedDraft2019_09 | test/unittest/schematest.cpp:3186 | legacy_test |
| SchemaValidator.Schema_UnsupportedDraft2020_12 | test/unittest/schematest.cpp:3197 | legacy_test |
| SchemaValidator.Schema_UnsupportedDraft3 | test/unittest/schematest.cpp:3154 | legacy_test |
| SchemaValidator.Schema_UnsupportedDraft6 | test/unittest/schematest.cpp:3165 | legacy_test |
| SchemaValidator.Schema_UnsupportedDraft7 | test/unittest/schematest.cpp:3175 | legacy_test |
| SchemaValidator.Schema_UnsupportedDraftOverride | test/unittest/schematest.cpp:3121 | legacy_test |
| SchemaValidator.Schema_UnsupportedVersion31 | test/unittest/schematest.cpp:3307 | legacy_test |
| SchemaValidator.Schema_UnsupportedVersionOverride | test/unittest/schematest.cpp:3263 | legacy_test |
| SchemaValidator.SchemaPointer | test/unittest/schematest.cpp:1842 | legacy_test |
| SchemaValidator.String | test/unittest/schematest.cpp:448 | legacy_test |
| SchemaValidator.String_LengthRange | test/unittest/schematest.cpp:486 | legacy_test |
| SchemaValidator.String_Pattern | test/unittest/schematest.cpp:508 | legacy_test |
| SchemaValidator.String_Pattern_Invalid | test/unittest/schematest.cpp:529 | legacy_test |
| SchemaValidator.TestSuite | test/unittest/schematest.cpp:2183 | legacy_test |
| SchemaValidator.Typeless | test/unittest/schematest.cpp:217 | legacy_test |
| SchemaValidator.UnknownValidationError | test/unittest/schematest.cpp:2977 | legacy_test |
| SchemaValidator.ValidateMetaSchema | test/unittest/schematest.cpp:2050 | legacy_test |
| SchemaValidator.ValidateMetaSchema_UTF16 | test/unittest/schematest.cpp:2078 | legacy_test |
| SchemaValidator.WriteOnlyWhenReading | test/unittest/schematest.cpp:3487 | legacy_test |## 3. 镜像测试层

### 3.1 镜像测试层结构

```text
rapidjson-refactoring/
├── rapidjson_refactoring_sys/
│   └── schema_ffi/                               # Schema 相关 FFI 适配代码（规划）
│
├── migrations/
│   └── schema-validate.legacy_to_mirror.json     # gtest -> Rust 镜像测试映射表
│
├── reports/
│   └── schema-validate.mirror.junit.xml          # 镜像层执行结果
│
└── rapidjson-rs/
    └── tests/
        └── mirrors/
            └── schema_validate_mirror.rs         # Schema 的 Rust 镜像测试源文件
```

### 3.2 镜像测试清单

| 测试 ID | 涉及 C/C++ 数据结构/接口 | 镜像测试 ID（Rust Test 路径） | 备注 |
|---------|-------------------------|------------------------------|------|
| Schema.Issue552 | test/unittest/schematest.cpp:2385 | schema_validate_mirror::Schema_Issue552 | |
| Schema.Issue848 | test/unittest/schematest.cpp:2370 | schema_validate_mirror::Schema_Issue848 | |
| Schema.TestSuite | test/perftest/schematest.cpp:198 | schema_validate_mirror::Schema_TestSuite | |
| SchemaValidatingReader.Invalid | test/unittest/schematest.cpp:2308 | schema_validate_mirror::SchemaValidatingReader_Invalid | |
| SchemaValidatingReader.Simple | test/unittest/schematest.cpp:2293 | schema_validate_mirror::SchemaValidatingReader_Simple | |
| SchemaValidatingWriter.Simple | test/unittest/schematest.cpp:2337 | schema_validate_mirror::SchemaValidatingWriter_Simple | |
| SchemaValidator.AllOf | test/unittest/schematest.cpp:278 | schema_validate_mirror::SchemaValidator_AllOf | |
| SchemaValidator.AllOf_Nested | test/unittest/schematest.cpp:1742 | schema_validate_mirror::SchemaValidator_AllOf_Nested | |
| SchemaValidator.AnyOf | test/unittest/schematest.cpp:311 | schema_validate_mirror::SchemaValidator_AnyOf | |
| SchemaValidator.Array | test/unittest/schematest.cpp:1465 | schema_validate_mirror::SchemaValidator_Array | |
| SchemaValidator.Array_AdditionalItems | test/unittest/schematest.cpp:1544 | schema_validate_mirror::SchemaValidator_Array_AdditionalItems | |
| SchemaValidator.Array_ItemsList | test/unittest/schematest.cpp:1480 | schema_validate_mirror::SchemaValidator_Array_ItemsList | |
| SchemaValidator.Array_ItemsRange | test/unittest/schematest.cpp:1579 | schema_validate_mirror::SchemaValidator_Array_ItemsRange | |
| SchemaValidator.Array_ItemsTuple | test/unittest/schematest.cpp:1501 | schema_validate_mirror::SchemaValidator_Array_ItemsTuple | |
| SchemaValidator.Array_UniqueItems | test/unittest/schematest.cpp:1606 | schema_validate_mirror::SchemaValidator_Array_UniqueItems | |
| SchemaValidator.Boolean | test/unittest/schematest.cpp:1627 | schema_validate_mirror::SchemaValidator_Boolean | |
| SchemaValidator.ContinueOnErrors | test/unittest/schematest.cpp:2678 | schema_validate_mirror::SchemaValidator_ContinueOnErrors | |
| SchemaValidator.ContinueOnErrors_AllOf | test/unittest/schematest.cpp:2799 | schema_validate_mirror::SchemaValidator_ContinueOnErrors_AllOf | |
| SchemaValidator.ContinueOnErrors_AnyOf | test/unittest/schematest.cpp:2825 | schema_validate_mirror::SchemaValidator_ContinueOnErrors_AnyOf | |
| SchemaValidator.ContinueOnErrors_BadSimpleType | test/unittest/schematest.cpp:2950 | schema_validate_mirror::SchemaValidator_ContinueOnErrors_BadSimpleType | |
| SchemaValidator.ContinueOnErrors_Enum | test/unittest/schematest.cpp:2875 | schema_validate_mirror::SchemaValidator_ContinueOnErrors_Enum | |
| SchemaValidator.ContinueOnErrors_OneOf | test/unittest/schematest.cpp:2773 | schema_validate_mirror::SchemaValidator_ContinueOnErrors_OneOf | |
| SchemaValidator.ContinueOnErrors_RogueArray | test/unittest/schematest.cpp:2894 | schema_validate_mirror::SchemaValidator_ContinueOnErrors_RogueArray | |
| SchemaValidator.ContinueOnErrors_RogueObject | test/unittest/schematest.cpp:2913 | schema_validate_mirror::SchemaValidator_ContinueOnErrors_RogueObject | |
| SchemaValidator.ContinueOnErrors_RogueString | test/unittest/schematest.cpp:2928 | schema_validate_mirror::SchemaValidator_ContinueOnErrors_RogueString | |
| SchemaValidator.ContinueOnErrors_UniqueItems | test/unittest/schematest.cpp:2853 | schema_validate_mirror::SchemaValidator_ContinueOnErrors_UniqueItems | |
| SchemaValidator.DuplicateKeyword | test/unittest/schematest.cpp:2982 | schema_validate_mirror::SchemaValidator_DuplicateKeyword | |
| SchemaValidator.Enum_InvalidType | test/unittest/schematest.cpp:264 | schema_validate_mirror::SchemaValidator_Enum_InvalidType | |
| SchemaValidator.Enum_Typed | test/unittest/schematest.cpp:242 | schema_validate_mirror::SchemaValidator_Enum_Typed | |
| SchemaValidator.Enum_Typeless | test/unittest/schematest.cpp:252 | schema_validate_mirror::SchemaValidator_Enum_Typeless | |
| SchemaValidator.EscapedPointer | test/unittest/schematest.cpp:1824 | schema_validate_mirror::SchemaValidator_EscapedPointer | |
| SchemaValidator.Hasher | test/unittest/schematest.cpp:51 | schema_validate_mirror::SchemaValidator_Hasher | |
| SchemaValidator.Integer | test/unittest/schematest.cpp:541 | schema_validate_mirror::SchemaValidator_Integer | |
| SchemaValidator.Integer_MultipleOf | test/unittest/schematest.cpp:706 | schema_validate_mirror::SchemaValidator_Integer_MultipleOf | |
| SchemaValidator.Integer_MultipleOf64Boundary | test/unittest/schematest.cpp:729 | schema_validate_mirror::SchemaValidator_Integer_MultipleOf64Boundary | |
| SchemaValidator.Integer_Range | test/unittest/schematest.cpp:566 | schema_validate_mirror::SchemaValidator_Integer_Range | |
| SchemaValidator.Integer_Range64Boundary | test/unittest/schematest.cpp:594 | schema_validate_mirror::SchemaValidator_Integer_Range64Boundary | |
| SchemaValidator.Integer_Range64BoundaryExclusive | test/unittest/schematest.cpp:683 | schema_validate_mirror::SchemaValidator_Integer_Range64BoundaryExclusive | |
| SchemaValidator.Integer_RangeU64Boundary | test/unittest/schematest.cpp:626 | schema_validate_mirror::SchemaValidator_Integer_RangeU64Boundary | |
| SchemaValidator.Issue1017_allOfHandler | test/unittest/schematest.cpp:2423 | schema_validate_mirror::SchemaValidator_Issue1017_allOfHandler | |
| SchemaValidator.Issue608 | test/unittest/schematest.cpp:2399 | schema_validate_mirror::SchemaValidator_Issue608 | |
| SchemaValidator.Issue728_AllOfRef | test/unittest/schematest.cpp:2414 | schema_validate_mirror::SchemaValidator_Issue728_AllOfRef | |
| SchemaValidator.MultiType | test/unittest/schematest.cpp:227 | schema_validate_mirror::SchemaValidator_MultiType | |
| SchemaValidator.MultiTypeInObject | test/unittest/schematest.cpp:1696 | schema_validate_mirror::SchemaValidator_MultiTypeInObject | |
| SchemaValidator.MultiTypeWithObject | test/unittest/schematest.cpp:1719 | schema_validate_mirror::SchemaValidator_MultiTypeWithObject | |
| SchemaValidator.Not | test/unittest/schematest.cpp:365 | schema_validate_mirror::SchemaValidator_Not | |
| SchemaValidator.Null | test/unittest/schematest.cpp:1648 | schema_validate_mirror::SchemaValidator_Null | |
| SchemaValidator.NullableFalse | test/unittest/schematest.cpp:3540 | schema_validate_mirror::SchemaValidator_NullableFalse | |
| SchemaValidator.NullableTrue | test/unittest/schematest.cpp:3509 | schema_validate_mirror::SchemaValidator_NullableTrue | |
| SchemaValidator.Number_MultipleOf | test/unittest/schematest.cpp:995 | schema_validate_mirror::SchemaValidator_Number_MultipleOf | |
| SchemaValidator.Number_MultipleOfOne | test/unittest/schematest.cpp:1039 | schema_validate_mirror::SchemaValidator_Number_MultipleOfOne | |
| SchemaValidator.Number_Range | test/unittest/schematest.cpp:744 | schema_validate_mirror::SchemaValidator_Number_Range | |
| SchemaValidator.Number_RangeDouble | test/unittest/schematest.cpp:855 | schema_validate_mirror::SchemaValidator_Number_RangeDouble | |
| SchemaValidator.Number_RangeDoubleU64Boundary | test/unittest/schematest.cpp:944 | schema_validate_mirror::SchemaValidator_Number_RangeDoubleU64Boundary | |
| SchemaValidator.Number_RangeInt | test/unittest/schematest.cpp:780 | schema_validate_mirror::SchemaValidator_Number_RangeInt | |
| SchemaValidator.Object | test/unittest/schematest.cpp:1054 | schema_validate_mirror::SchemaValidator_Object | |
| SchemaValidator.Object_AdditionalPropertiesBoolean | test/unittest/schematest.cpp:1108 | schema_validate_mirror::SchemaValidator_Object_AdditionalPropertiesBoolean | |
| SchemaValidator.Object_AdditionalPropertiesObject | test/unittest/schematest.cpp:1134 | schema_validate_mirror::SchemaValidator_Object_AdditionalPropertiesObject | |
| SchemaValidator.Object_PatternProperties | test/unittest/schematest.cpp:1322 | schema_validate_mirror::SchemaValidator_Object_PatternProperties | |
| SchemaValidator.Object_PatternProperties_AdditionalPropertiesBoolean | test/unittest/schematest.cpp:1441 | schema_validate_mirror::SchemaValidator_Object_PatternProperties_AdditionalPropertiesBoolean | |
| SchemaValidator.Object_PatternProperties_AdditionalPropertiesObject | test/unittest/schematest.cpp:1414 | schema_validate_mirror::SchemaValidator_Object_PatternProperties_AdditionalPropertiesObject | |
| SchemaValidator.Object_PatternProperties_ErrorConflict | test/unittest/schematest.cpp:1351 | schema_validate_mirror::SchemaValidator_Object_PatternProperties_ErrorConflict | |
| SchemaValidator.Object_Properties | test/unittest/schematest.cpp:1075 | schema_validate_mirror::SchemaValidator_Object_Properties | |
| SchemaValidator.Object_Properties_PatternProperties | test/unittest/schematest.cpp:1378 | schema_validate_mirror::SchemaValidator_Object_Properties_PatternProperties | |
| SchemaValidator.Object_PropertiesRange | test/unittest/schematest.cpp:1221 | schema_validate_mirror::SchemaValidator_Object_PropertiesRange | |
| SchemaValidator.Object_PropertyDependencies | test/unittest/schematest.cpp:1248 | schema_validate_mirror::SchemaValidator_Object_PropertyDependencies | |
| SchemaValidator.Object_Required | test/unittest/schematest.cpp:1160 | schema_validate_mirror::SchemaValidator_Object_Required | |
| SchemaValidator.Object_Required_PassWithDefault | test/unittest/schematest.cpp:1191 | schema_validate_mirror::SchemaValidator_Object_Required_PassWithDefault | |
| SchemaValidator.Object_SchemaDependencies | test/unittest/schematest.cpp:1284 | schema_validate_mirror::SchemaValidator_Object_SchemaDependencies | |
| SchemaValidator.ObjectInArray | test/unittest/schematest.cpp:1676 | schema_validate_mirror::SchemaValidator_ObjectInArray | |
| SchemaValidator.OneOf | test/unittest/schematest.cpp:337 | schema_validate_mirror::SchemaValidator_OneOf | |
| SchemaValidator.ReadOnlyWhenWriting | test/unittest/schematest.cpp:3465 | schema_validate_mirror::SchemaValidator_ReadOnlyWhenWriting | |
| SchemaValidator.Ref | test/unittest/schematest.cpp:376 | schema_validate_mirror::SchemaValidator_Ref | |
| SchemaValidator.Ref_AllOf | test/unittest/schematest.cpp:404 | schema_validate_mirror::SchemaValidator_Ref_AllOf | |
| SchemaValidator.Ref_internal_id_1 | test/unittest/schematest.cpp:2537 | schema_validate_mirror::SchemaValidator_Ref_internal_id_1 | |
| SchemaValidator.Ref_internal_id_2 | test/unittest/schematest.cpp:2555 | schema_validate_mirror::SchemaValidator_Ref_internal_id_2 | |
| SchemaValidator.Ref_internal_id_and_schema_pointer | test/unittest/schematest.cpp:2591 | schema_validate_mirror::SchemaValidator_Ref_internal_id_and_schema_pointer | |
| SchemaValidator.Ref_internal_id_in_array | test/unittest/schematest.cpp:2573 | schema_validate_mirror::SchemaValidator_Ref_internal_id_in_array | |
| SchemaValidator.Ref_internal_multiple_ids | test/unittest/schematest.cpp:2610 | schema_validate_mirror::SchemaValidator_Ref_internal_multiple_ids | |
| SchemaValidator.Ref_remote | test/unittest/schematest.cpp:2442 | schema_validate_mirror::SchemaValidator_Ref_remote | |
| SchemaValidator.Ref_remote_change_resolution_scope_absolute_path | test/unittest/schematest.cpp:2499 | schema_validate_mirror::SchemaValidator_Ref_remote_change_resolution_scope_absolute_path | |
| SchemaValidator.Ref_remote_change_resolution_scope_absolute_path_document | test/unittest/schematest.cpp:2518 | schema_validate_mirror::SchemaValidator_Ref_remote_change_resolution_scope_absolute_path_document | |
| SchemaValidator.Ref_remote_change_resolution_scope_relative_path | test/unittest/schematest.cpp:2480 | schema_validate_mirror::SchemaValidator_Ref_remote_change_resolution_scope_relative_path | |
| SchemaValidator.Ref_remote_change_resolution_scope_uri | test/unittest/schematest.cpp:2461 | schema_validate_mirror::SchemaValidator_Ref_remote_change_resolution_scope_uri | |
| SchemaValidator.Ref_remote_issue1210 | test/unittest/schematest.cpp:2639 | schema_validate_mirror::SchemaValidator_Ref_remote_issue1210 | |
| SchemaValidator.Schema_DraftAndVersion | test/unittest/schematest.cpp:3318 | schema_validate_mirror::SchemaValidator_Schema_DraftAndVersion | |
| SchemaValidator.Schema_IgnoreDraftEmbedded | test/unittest/schematest.cpp:3088 | schema_validate_mirror::SchemaValidator_Schema_IgnoreDraftEmbedded | |
| SchemaValidator.Schema_MultipleErrors | test/unittest/schematest.cpp:3335 | schema_validate_mirror::SchemaValidator_Schema_MultipleErrors | |
| SchemaValidator.Schema_ReadOnlyAndWriteOnly | test/unittest/schematest.cpp:3455 | schema_validate_mirror::SchemaValidator_Schema_ReadOnlyAndWriteOnly | |
| SchemaValidator.Schema_RefCyclical | test/unittest/schematest.cpp:3440 | schema_validate_mirror::SchemaValidator_Schema_RefCyclical | |
| SchemaValidator.Schema_RefEmptyString | test/unittest/schematest.cpp:3365 | schema_validate_mirror::SchemaValidator_Schema_RefEmptyString | |
| SchemaValidator.Schema_RefNoRemoteProvider | test/unittest/schematest.cpp:3374 | schema_validate_mirror::SchemaValidator_Schema_RefNoRemoteProvider | |
| SchemaValidator.Schema_RefNoRemoteSchema | test/unittest/schematest.cpp:3383 | schema_validate_mirror::SchemaValidator_Schema_RefNoRemoteSchema | |
| SchemaValidator.Schema_RefPlainNameOpenApi | test/unittest/schematest.cpp:3346 | schema_validate_mirror::SchemaValidator_Schema_RefPlainNameOpenApi | |
| SchemaValidator.Schema_RefPlainNameRemote | test/unittest/schematest.cpp:3355 | schema_validate_mirror::SchemaValidator_Schema_RefPlainNameRemote | |
| SchemaValidator.Schema_RefPointerInvalid | test/unittest/schematest.cpp:3393 | schema_validate_mirror::SchemaValidator_Schema_RefPointerInvalid | |
| SchemaValidator.Schema_RefPointerInvalidRemote | test/unittest/schematest.cpp:3402 | schema_validate_mirror::SchemaValidator_Schema_RefPointerInvalidRemote | |
| SchemaValidator.Schema_RefUnknownPlainName | test/unittest/schematest.cpp:3412 | schema_validate_mirror::SchemaValidator_Schema_RefUnknownPlainName | |
| SchemaValidator.Schema_RefUnknownPointer | test/unittest/schematest.cpp:3421 | schema_validate_mirror::SchemaValidator_Schema_RefUnknownPointer | |
| SchemaValidator.Schema_RefUnknownPointerRemote | test/unittest/schematest.cpp:3430 | schema_validate_mirror::SchemaValidator_Schema_RefUnknownPointerRemote | |
| SchemaValidator.Schema_StartUnknown | test/unittest/schematest.cpp:3327 | schema_validate_mirror::SchemaValidator_Schema_StartUnknown | |
| SchemaValidator.Schema_SupportedDraft4 | test/unittest/schematest.cpp:3044 | schema_validate_mirror::SchemaValidator_Schema_SupportedDraft4 | |
| SchemaValidator.Schema_SupportedDraft4NoFrag | test/unittest/schematest.cpp:3055 | schema_validate_mirror::SchemaValidator_Schema_SupportedDraft4NoFrag | |
| SchemaValidator.Schema_SupportedDraft5 | test/unittest/schematest.cpp:3066 | schema_validate_mirror::SchemaValidator_Schema_SupportedDraft5 | |
| SchemaValidator.Schema_SupportedDraft5NoFrag | test/unittest/schematest.cpp:3077 | schema_validate_mirror::SchemaValidator_Schema_SupportedDraft5NoFrag | |
| SchemaValidator.Schema_SupportedDraft5Static | test/unittest/schematest.cpp:3033 | schema_validate_mirror::SchemaValidator_Schema_SupportedDraft5Static | |
| SchemaValidator.Schema_SupportedDraftOverride | test/unittest/schematest.cpp:3099 | schema_validate_mirror::SchemaValidator_Schema_SupportedDraftOverride | |
| SchemaValidator.Schema_SupportedNoSpec | test/unittest/schematest.cpp:3011 | schema_validate_mirror::SchemaValidator_Schema_SupportedNoSpec | |
| SchemaValidator.Schema_SupportedNoSpecStatic | test/unittest/schematest.cpp:3022 | schema_validate_mirror::SchemaValidator_Schema_SupportedNoSpecStatic | |
| SchemaValidator.Schema_SupportedNotObject | test/unittest/schematest.cpp:3000 | schema_validate_mirror::SchemaValidator_Schema_SupportedNotObject | |
| SchemaValidator.Schema_SupportedVersion20 | test/unittest/schematest.cpp:3219 | schema_validate_mirror::SchemaValidator_Schema_SupportedVersion20 | |
| SchemaValidator.Schema_SupportedVersion20Static | test/unittest/schematest.cpp:3208 | schema_validate_mirror::SchemaValidator_Schema_SupportedVersion20Static | |
| SchemaValidator.Schema_SupportedVersion30x | test/unittest/schematest.cpp:3230 | schema_validate_mirror::SchemaValidator_Schema_SupportedVersion30x | |
| SchemaValidator.Schema_SupportedVersionOverride | test/unittest/schematest.cpp:3241 | schema_validate_mirror::SchemaValidator_Schema_SupportedVersionOverride | |
| SchemaValidator.Schema_UnknownDraft | test/unittest/schematest.cpp:3132 | schema_validate_mirror::SchemaValidator_Schema_UnknownDraft | |
| SchemaValidator.Schema_UnknownDraftNotString | test/unittest/schematest.cpp:3143 | schema_validate_mirror::SchemaValidator_Schema_UnknownDraftNotString | |
| SchemaValidator.Schema_UnknownDraftOverride | test/unittest/schematest.cpp:3110 | schema_validate_mirror::SchemaValidator_Schema_UnknownDraftOverride | |
| SchemaValidator.Schema_UnknownVersion | test/unittest/schematest.cpp:3274 | schema_validate_mirror::SchemaValidator_Schema_UnknownVersion | |
| SchemaValidator.Schema_UnknownVersionNotString | test/unittest/schematest.cpp:3296 | schema_validate_mirror::SchemaValidator_Schema_UnknownVersionNotString | |
| SchemaValidator.Schema_UnknownVersionOverride | test/unittest/schematest.cpp:3252 | schema_validate_mirror::SchemaValidator_Schema_UnknownVersionOverride | |
| SchemaValidator.Schema_UnknownVersionShort | test/unittest/schematest.cpp:3285 | schema_validate_mirror::SchemaValidator_Schema_UnknownVersionShort | |
| SchemaValidator.Schema_UnsupportedDraft2019_09 | test/unittest/schematest.cpp:3186 | schema_validate_mirror::SchemaValidator_Schema_UnsupportedDraft2019_09 | |
| SchemaValidator.Schema_UnsupportedDraft2020_12 | test/unittest/schematest.cpp:3197 | schema_validate_mirror::SchemaValidator_Schema_UnsupportedDraft2020_12 | |
| SchemaValidator.Schema_UnsupportedDraft3 | test/unittest/schematest.cpp:3154 | schema_validate_mirror::SchemaValidator_Schema_UnsupportedDraft3 | |
| SchemaValidator.Schema_UnsupportedDraft6 | test/unittest/schematest.cpp:3165 | schema_validate_mirror::SchemaValidator_Schema_UnsupportedDraft6 | |
| SchemaValidator.Schema_UnsupportedDraft7 | test/unittest/schematest.cpp:3175 | schema_validate_mirror::SchemaValidator_Schema_UnsupportedDraft7 | |
| SchemaValidator.Schema_UnsupportedDraftOverride | test/unittest/schematest.cpp:3121 | schema_validate_mirror::SchemaValidator_Schema_UnsupportedDraftOverride | |
| SchemaValidator.Schema_UnsupportedVersion31 | test/unittest/schematest.cpp:3307 | schema_validate_mirror::SchemaValidator_Schema_UnsupportedVersion31 | |
| SchemaValidator.Schema_UnsupportedVersionOverride | test/unittest/schematest.cpp:3263 | schema_validate_mirror::SchemaValidator_Schema_UnsupportedVersionOverride | |
| SchemaValidator.SchemaPointer | test/unittest/schematest.cpp:1842 | schema_validate_mirror::SchemaValidator_SchemaPointer | |
| SchemaValidator.String | test/unittest/schematest.cpp:448 | schema_validate_mirror::SchemaValidator_String | |
| SchemaValidator.String_LengthRange | test/unittest/schematest.cpp:486 | schema_validate_mirror::SchemaValidator_String_LengthRange | |
| SchemaValidator.String_Pattern | test/unittest/schematest.cpp:508 | schema_validate_mirror::SchemaValidator_String_Pattern | |
| SchemaValidator.String_Pattern_Invalid | test/unittest/schematest.cpp:529 | schema_validate_mirror::SchemaValidator_String_Pattern_Invalid | |
| SchemaValidator.TestSuite | test/unittest/schematest.cpp:2183 | schema_validate_mirror::SchemaValidator_TestSuite | |
| SchemaValidator.Typeless | test/unittest/schematest.cpp:217 | schema_validate_mirror::SchemaValidator_Typeless | |
| SchemaValidator.UnknownValidationError | test/unittest/schematest.cpp:2977 | schema_validate_mirror::SchemaValidator_UnknownValidationError | |
| SchemaValidator.ValidateMetaSchema | test/unittest/schematest.cpp:2050 | schema_validate_mirror::SchemaValidator_ValidateMetaSchema | |
| SchemaValidator.ValidateMetaSchema_UTF16 | test/unittest/schematest.cpp:2078 | schema_validate_mirror::SchemaValidator_ValidateMetaSchema_UTF16 | |
| SchemaValidator.WriteOnlyWhenReading | test/unittest/schematest.cpp:3487 | schema_validate_mirror::SchemaValidator_WriteOnlyWhenReading | |#### 3.3 迁移策略建议

| 测试类别 | 进入 L1 优先级建议 | 原因 |
|----------|--------------------|------|
| 基础类型与关键字（type/enum/number/string/object/array） | high | Schema 功能核心，应优先迁移。 |
| 组合关键字（allOf/anyOf/oneOf/not） | high | 行为复杂，测试覆盖重要。 |
| `$ref` 与远程引用相关测试 | high | 影响 Schema 复用与模块化。 |
| draft/version 相关测试 | medium | 影响规范兼容性，重要但优先级略低。 |
| 性能测试 | medium | 行为稳定后重点对齐性能。 |

### 3.4 镜像测试层交付件

| 交付件 | 说明 |
|--------|------|
| `reports/schema-validate.mirror.junit.xml` | 镜像层执行结果。 |
| `migrations/schema-validate.legacy_to_mirror.json` | gtest -> Rust 镜像测试映射表。 |
| `rapidjson-rs/tests/mirrors/schema_validate_mirror.rs` | Schema 镜像测试源文件。 |

---

## 4. 孪生测试层

### 4.1 孪生测试层结构

```text
rapidjson-refactoring/
├── reports/
│   ├── schema-validate.rust.junit.xml            # 孪生层执行结果（Rust Schema 实现）
│   └── schema-validate.parity.json               # 镜像 vs 孪生 一致性报告
│
├── migrations/
│   └── schema-validate.mirror_to_rust.json       # 镜像测试 -> 孪生测试映射表
│
└── rapidjson-rs/
    └── tests/
        └── rust/
            └── schema_validate.rs                # Schema 的 Rust 原生测试源文件
```

### 4.2 孪生测试清单

| 测试 ID | 镜像测试 ID | 涉及 Rust 数据结构/接口 | 原生测试 ID（Test 路径） | 是否验收 | 备注 |
|---------|-------------|------------------------|---------------------------|----------|------|
| Schema.Issue552 | schema_validate_mirror::Schema_Issue552 | schema_validate | schema_validate::Schema_Issue552_rust | | |
| Schema.Issue848 | schema_validate_mirror::Schema_Issue848 | schema_validate | schema_validate::Schema_Issue848_rust | | |
| Schema.TestSuite | schema_validate_mirror::Schema_TestSuite | schema_validate | schema_validate::Schema_TestSuite_rust | | |
| SchemaValidatingReader.Invalid | schema_validate_mirror::SchemaValidatingReader_Invalid | schema_validate | schema_validate::SchemaValidatingReader_Invalid_rust | | |
| SchemaValidatingReader.Simple | schema_validate_mirror::SchemaValidatingReader_Simple | schema_validate | schema_validate::SchemaValidatingReader_Simple_rust | | |
| SchemaValidatingWriter.Simple | schema_validate_mirror::SchemaValidatingWriter_Simple | schema_validate | schema_validate::SchemaValidatingWriter_Simple_rust | | |
| SchemaValidator.AllOf | schema_validate_mirror::SchemaValidator_AllOf | schema_validate | schema_validate::SchemaValidator_AllOf_rust | | |
| SchemaValidator.AllOf_Nested | schema_validate_mirror::SchemaValidator_AllOf_Nested | schema_validate | schema_validate::SchemaValidator_AllOf_Nested_rust | | |
| SchemaValidator.AnyOf | schema_validate_mirror::SchemaValidator_AnyOf | schema_validate | schema_validate::SchemaValidator_AnyOf_rust | | |
| SchemaValidator.Array | schema_validate_mirror::SchemaValidator_Array | schema_validate | schema_validate::SchemaValidator_Array_rust | | |
| SchemaValidator.Array_AdditionalItems | schema_validate_mirror::SchemaValidator_Array_AdditionalItems | schema_validate | schema_validate::SchemaValidator_Array_AdditionalItems_rust | | |
| SchemaValidator.Array_ItemsList | schema_validate_mirror::SchemaValidator_Array_ItemsList | schema_validate | schema_validate::SchemaValidator_Array_ItemsList_rust | | |
| SchemaValidator.Array_ItemsRange | schema_validate_mirror::SchemaValidator_Array_ItemsRange | schema_validate | schema_validate::SchemaValidator_Array_ItemsRange_rust | | |
| SchemaValidator.Array_ItemsTuple | schema_validate_mirror::SchemaValidator_Array_ItemsTuple | schema_validate | schema_validate::SchemaValidator_Array_ItemsTuple_rust | | |
| SchemaValidator.Array_UniqueItems | schema_validate_mirror::SchemaValidator_Array_UniqueItems | schema_validate | schema_validate::SchemaValidator_Array_UniqueItems_rust | | |
| SchemaValidator.Boolean | schema_validate_mirror::SchemaValidator_Boolean | schema_validate | schema_validate::SchemaValidator_Boolean_rust | | |
| SchemaValidator.ContinueOnErrors | schema_validate_mirror::SchemaValidator_ContinueOnErrors | schema_validate | schema_validate::SchemaValidator_ContinueOnErrors_rust | | |
| SchemaValidator.ContinueOnErrors_AllOf | schema_validate_mirror::SchemaValidator_ContinueOnErrors_AllOf | schema_validate | schema_validate::SchemaValidator_ContinueOnErrors_AllOf_rust | | |
| SchemaValidator.ContinueOnErrors_AnyOf | schema_validate_mirror::SchemaValidator_ContinueOnErrors_AnyOf | schema_validate | schema_validate::SchemaValidator_ContinueOnErrors_AnyOf_rust | | |
| SchemaValidator.ContinueOnErrors_BadSimpleType | schema_validate_mirror::SchemaValidator_ContinueOnErrors_BadSimpleType | schema_validate | schema_validate::SchemaValidator_ContinueOnErrors_BadSimpleType_rust | | |
| SchemaValidator.ContinueOnErrors_Enum | schema_validate_mirror::SchemaValidator_ContinueOnErrors_Enum | schema_validate | schema_validate::SchemaValidator_ContinueOnErrors_Enum_rust | | |
| SchemaValidator.ContinueOnErrors_OneOf | schema_validate_mirror::SchemaValidator_ContinueOnErrors_OneOf | schema_validate | schema_validate::SchemaValidator_ContinueOnErrors_OneOf_rust | | |
| SchemaValidator.ContinueOnErrors_RogueArray | schema_validate_mirror::SchemaValidator_ContinueOnErrors_RogueArray | schema_validate | schema_validate::SchemaValidator_ContinueOnErrors_RogueArray_rust | | |
| SchemaValidator.ContinueOnErrors_RogueObject | schema_validate_mirror::SchemaValidator_ContinueOnErrors_RogueObject | schema_validate | schema_validate::SchemaValidator_ContinueOnErrors_RogueObject_rust | | |
| SchemaValidator.ContinueOnErrors_RogueString | schema_validate_mirror::SchemaValidator_ContinueOnErrors_RogueString | schema_validate | schema_validate::SchemaValidator_ContinueOnErrors_RogueString_rust | | |
| SchemaValidator.ContinueOnErrors_UniqueItems | schema_validate_mirror::SchemaValidator_ContinueOnErrors_UniqueItems | schema_validate | schema_validate::SchemaValidator_ContinueOnErrors_UniqueItems_rust | | |
| SchemaValidator.DuplicateKeyword | schema_validate_mirror::SchemaValidator_DuplicateKeyword | schema_validate | schema_validate::SchemaValidator_DuplicateKeyword_rust | | |
| SchemaValidator.Enum_InvalidType | schema_validate_mirror::SchemaValidator_Enum_InvalidType | schema_validate | schema_validate::SchemaValidator_Enum_InvalidType_rust | | |
| SchemaValidator.Enum_Typed | schema_validate_mirror::SchemaValidator_Enum_Typed | schema_validate | schema_validate::SchemaValidator_Enum_Typed_rust | | |
| SchemaValidator.Enum_Typeless | schema_validate_mirror::SchemaValidator_Enum_Typeless | schema_validate | schema_validate::SchemaValidator_Enum_Typeless_rust | | |
| SchemaValidator.EscapedPointer | schema_validate_mirror::SchemaValidator_EscapedPointer | schema_validate | schema_validate::SchemaValidator_EscapedPointer_rust | | |
| SchemaValidator.Hasher | schema_validate_mirror::SchemaValidator_Hasher | schema_validate | schema_validate::SchemaValidator_Hasher_rust | | |
| SchemaValidator.Integer | schema_validate_mirror::SchemaValidator_Integer | schema_validate | schema_validate::SchemaValidator_Integer_rust | | |
| SchemaValidator.Integer_MultipleOf | schema_validate_mirror::SchemaValidator_Integer_MultipleOf | schema_validate | schema_validate::SchemaValidator_Integer_MultipleOf_rust | | |
| SchemaValidator.Integer_MultipleOf64Boundary | schema_validate_mirror::SchemaValidator_Integer_MultipleOf64Boundary | schema_validate | schema_validate::SchemaValidator_Integer_MultipleOf64Boundary_rust | | |
| SchemaValidator.Integer_Range | schema_validate_mirror::SchemaValidator_Integer_Range | schema_validate | schema_validate::SchemaValidator_Integer_Range_rust | | |
| SchemaValidator.Integer_Range64Boundary | schema_validate_mirror::SchemaValidator_Integer_Range64Boundary | schema_validate | schema_validate::SchemaValidator_Integer_Range64Boundary_rust | | |
| SchemaValidator.Integer_Range64BoundaryExclusive | schema_validate_mirror::SchemaValidator_Integer_Range64BoundaryExclusive | schema_validate | schema_validate::SchemaValidator_Integer_Range64BoundaryExclusive_rust | | |
| SchemaValidator.Integer_RangeU64Boundary | schema_validate_mirror::SchemaValidator_Integer_RangeU64Boundary | schema_validate | schema_validate::SchemaValidator_Integer_RangeU64Boundary_rust | | |
| SchemaValidator.Issue1017_allOfHandler | schema_validate_mirror::SchemaValidator_Issue1017_allOfHandler | schema_validate | schema_validate::SchemaValidator_Issue1017_allOfHandler_rust | | |
| SchemaValidator.Issue608 | schema_validate_mirror::SchemaValidator_Issue608 | schema_validate | schema_validate::SchemaValidator_Issue608_rust | | |
| SchemaValidator.Issue728_AllOfRef | schema_validate_mirror::SchemaValidator_Issue728_AllOfRef | schema_validate | schema_validate::SchemaValidator_Issue728_AllOfRef_rust | | |
| SchemaValidator.MultiType | schema_validate_mirror::SchemaValidator_MultiType | schema_validate | schema_validate::SchemaValidator_MultiType_rust | | |
| SchemaValidator.MultiTypeInObject | schema_validate_mirror::SchemaValidator_MultiTypeInObject | schema_validate | schema_validate::SchemaValidator_MultiTypeInObject_rust | | |
| SchemaValidator.MultiTypeWithObject | schema_validate_mirror::SchemaValidator_MultiTypeWithObject | schema_validate | schema_validate::SchemaValidator_MultiTypeWithObject_rust | | |
| SchemaValidator.Not | schema_validate_mirror::SchemaValidator_Not | schema_validate | schema_validate::SchemaValidator_Not_rust | | |
| SchemaValidator.Null | schema_validate_mirror::SchemaValidator_Null | schema_validate | schema_validate::SchemaValidator_Null_rust | | |
| SchemaValidator.NullableFalse | schema_validate_mirror::SchemaValidator_NullableFalse | schema_validate | schema_validate::SchemaValidator_NullableFalse_rust | | |
| SchemaValidator.NullableTrue | schema_validate_mirror::SchemaValidator_NullableTrue | schema_validate | schema_validate::SchemaValidator_NullableTrue_rust | | |
| SchemaValidator.Number_MultipleOf | schema_validate_mirror::SchemaValidator_Number_MultipleOf | schema_validate | schema_validate::SchemaValidator_Number_MultipleOf_rust | | |
| SchemaValidator.Number_MultipleOfOne | schema_validate_mirror::SchemaValidator_Number_MultipleOfOne | schema_validate | schema_validate::SchemaValidator_Number_MultipleOfOne_rust | | |
| SchemaValidator.Number_Range | schema_validate_mirror::SchemaValidator_Number_Range | schema_validate | schema_validate::SchemaValidator_Number_Range_rust | | |
| SchemaValidator.Number_RangeDouble | schema_validate_mirror::SchemaValidator_Number_RangeDouble | schema_validate | schema_validate::SchemaValidator_Number_RangeDouble_rust | | |
| SchemaValidator.Number_RangeDoubleU64Boundary | schema_validate_mirror::SchemaValidator_Number_RangeDoubleU64Boundary | schema_validate | schema_validate::SchemaValidator_Number_RangeDoubleU64Boundary_rust | | |
| SchemaValidator.Number_RangeInt | schema_validate_mirror::SchemaValidator_Number_RangeInt | schema_validate | schema_validate::SchemaValidator_Number_RangeInt_rust | | |
| SchemaValidator.Object | schema_validate_mirror::SchemaValidator_Object | schema_validate | schema_validate::SchemaValidator_Object_rust | | |
| SchemaValidator.Object_AdditionalPropertiesBoolean | schema_validate_mirror::SchemaValidator_Object_AdditionalPropertiesBoolean | schema_validate | schema_validate::SchemaValidator_Object_AdditionalPropertiesBoolean_rust | | |
| SchemaValidator.Object_AdditionalPropertiesObject | schema_validate_mirror::SchemaValidator_Object_AdditionalPropertiesObject | schema_validate | schema_validate::SchemaValidator_Object_AdditionalPropertiesObject_rust | | |
| SchemaValidator.Object_PatternProperties | schema_validate_mirror::SchemaValidator_Object_PatternProperties | schema_validate | schema_validate::SchemaValidator_Object_PatternProperties_rust | | |
| SchemaValidator.Object_PatternProperties_AdditionalPropertiesBoolean | schema_validate_mirror::SchemaValidator_Object_PatternProperties_AdditionalPropertiesBoolean | schema_validate | schema_validate::SchemaValidator_Object_PatternProperties_AdditionalPropertiesBoolean_rust | | |
| SchemaValidator.Object_PatternProperties_AdditionalPropertiesObject | schema_validate_mirror::SchemaValidator_Object_PatternProperties_AdditionalPropertiesObject | schema_validate | schema_validate::SchemaValidator_Object_PatternProperties_AdditionalPropertiesObject_rust | | |
| SchemaValidator.Object_PatternProperties_ErrorConflict | schema_validate_mirror::SchemaValidator_Object_PatternProperties_ErrorConflict | schema_validate | schema_validate::SchemaValidator_Object_PatternProperties_ErrorConflict_rust | | |
| SchemaValidator.Object_Properties | schema_validate_mirror::SchemaValidator_Object_Properties | schema_validate | schema_validate::SchemaValidator_Object_Properties_rust | | |
| SchemaValidator.Object_Properties_PatternProperties | schema_validate_mirror::SchemaValidator_Object_Properties_PatternProperties | schema_validate | schema_validate::SchemaValidator_Object_Properties_PatternProperties_rust | | |
| SchemaValidator.Object_PropertiesRange | schema_validate_mirror::SchemaValidator_Object_PropertiesRange | schema_validate | schema_validate::SchemaValidator_Object_PropertiesRange_rust | | |
| SchemaValidator.Object_PropertyDependencies | schema_validate_mirror::SchemaValidator_Object_PropertyDependencies | schema_validate | schema_validate::SchemaValidator_Object_PropertyDependencies_rust | | |
| SchemaValidator.Object_Required | schema_validate_mirror::SchemaValidator_Object_Required | schema_validate | schema_validate::SchemaValidator_Object_Required_rust | | |
| SchemaValidator.Object_Required_PassWithDefault | schema_validate_mirror::SchemaValidator_Object_Required_PassWithDefault | schema_validate | schema_validate::SchemaValidator_Object_Required_PassWithDefault_rust | | |
| SchemaValidator.Object_SchemaDependencies | schema_validate_mirror::SchemaValidator_Object_SchemaDependencies | schema_validate | schema_validate::SchemaValidator_Object_SchemaDependencies_rust | | |
| SchemaValidator.ObjectInArray | schema_validate_mirror::SchemaValidator_ObjectInArray | schema_validate | schema_validate::SchemaValidator_ObjectInArray_rust | | |
| SchemaValidator.OneOf | schema_validate_mirror::SchemaValidator_OneOf | schema_validate | schema_validate::SchemaValidator_OneOf_rust | | |
| SchemaValidator.ReadOnlyWhenWriting | schema_validate_mirror::SchemaValidator_ReadOnlyWhenWriting | schema_validate | schema_validate::SchemaValidator_ReadOnlyWhenWriting_rust | | |
| SchemaValidator.Ref | schema_validate_mirror::SchemaValidator_Ref | schema_validate | schema_validate::SchemaValidator_Ref_rust | | |
| SchemaValidator.Ref_AllOf | schema_validate_mirror::SchemaValidator_Ref_AllOf | schema_validate | schema_validate::SchemaValidator_Ref_AllOf_rust | | |
| SchemaValidator.Ref_internal_id_1 | schema_validate_mirror::SchemaValidator_Ref_internal_id_1 | schema_validate | schema_validate::SchemaValidator_Ref_internal_id_1_rust | | |
| SchemaValidator.Ref_internal_id_2 | schema_validate_mirror::SchemaValidator_Ref_internal_id_2 | schema_validate | schema_validate::SchemaValidator_Ref_internal_id_2_rust | | |
| SchemaValidator.Ref_internal_id_and_schema_pointer | schema_validate_mirror::SchemaValidator_Ref_internal_id_and_schema_pointer | schema_validate | schema_validate::SchemaValidator_Ref_internal_id_and_schema_pointer_rust | | |
| SchemaValidator.Ref_internal_id_in_array | schema_validate_mirror::SchemaValidator_Ref_internal_id_in_array | schema_validate | schema_validate::SchemaValidator_Ref_internal_id_in_array_rust | | |
| SchemaValidator.Ref_internal_multiple_ids | schema_validate_mirror::SchemaValidator_Ref_internal_multiple_ids | schema_validate | schema_validate::SchemaValidator_Ref_internal_multiple_ids_rust | | |
| SchemaValidator.Ref_remote | schema_validate_mirror::SchemaValidator_Ref_remote | schema_validate | schema_validate::SchemaValidator_Ref_remote_rust | | |
| SchemaValidator.Ref_remote_change_resolution_scope_absolute_path | schema_validate_mirror::SchemaValidator_Ref_remote_change_resolution_scope_absolute_path | schema_validate | schema_validate::SchemaValidator_Ref_remote_change_resolution_scope_absolute_path_rust | | |
| SchemaValidator.Ref_remote_change_resolution_scope_absolute_path_document | schema_validate_mirror::SchemaValidator_Ref_remote_change_resolution_scope_absolute_path_document | schema_validate | schema_validate::SchemaValidator_Ref_remote_change_resolution_scope_absolute_path_document_rust | | |
| SchemaValidator.Ref_remote_change_resolution_scope_relative_path | schema_validate_mirror::SchemaValidator_Ref_remote_change_resolution_scope_relative_path | schema_validate | schema_validate::SchemaValidator_Ref_remote_change_resolution_scope_relative_path_rust | | |
| SchemaValidator.Ref_remote_change_resolution_scope_uri | schema_validate_mirror::SchemaValidator_Ref_remote_change_resolution_scope_uri | schema_validate | schema_validate::SchemaValidator_Ref_remote_change_resolution_scope_uri_rust | | |
| SchemaValidator.Ref_remote_issue1210 | schema_validate_mirror::SchemaValidator_Ref_remote_issue1210 | schema_validate | schema_validate::SchemaValidator_Ref_remote_issue1210_rust | | |
| SchemaValidator.Schema_DraftAndVersion | schema_validate_mirror::SchemaValidator_Schema_DraftAndVersion | schema_validate | schema_validate::SchemaValidator_Schema_DraftAndVersion_rust | | |
| SchemaValidator.Schema_IgnoreDraftEmbedded | schema_validate_mirror::SchemaValidator_Schema_IgnoreDraftEmbedded | schema_validate | schema_validate::SchemaValidator_Schema_IgnoreDraftEmbedded_rust | | |
| SchemaValidator.Schema_MultipleErrors | schema_validate_mirror::SchemaValidator_Schema_MultipleErrors | schema_validate | schema_validate::SchemaValidator_Schema_MultipleErrors_rust | | |
| SchemaValidator.Schema_ReadOnlyAndWriteOnly | schema_validate_mirror::SchemaValidator_Schema_ReadOnlyAndWriteOnly | schema_validate | schema_validate::SchemaValidator_Schema_ReadOnlyAndWriteOnly_rust | | |
| SchemaValidator.Schema_RefCyclical | schema_validate_mirror::SchemaValidator_Schema_RefCyclical | schema_validate | schema_validate::SchemaValidator_Schema_RefCyclical_rust | | |
| SchemaValidator.Schema_RefEmptyString | schema_validate_mirror::SchemaValidator_Schema_RefEmptyString | schema_validate | schema_validate::SchemaValidator_Schema_RefEmptyString_rust | | |
| SchemaValidator.Schema_RefNoRemoteProvider | schema_validate_mirror::SchemaValidator_Schema_RefNoRemoteProvider | schema_validate | schema_validate::SchemaValidator_Schema_RefNoRemoteProvider_rust | | |
| SchemaValidator.Schema_RefNoRemoteSchema | schema_validate_mirror::SchemaValidator_Schema_RefNoRemoteSchema | schema_validate | schema_validate::SchemaValidator_Schema_RefNoRemoteSchema_rust | | |
| SchemaValidator.Schema_RefPlainNameOpenApi | schema_validate_mirror::SchemaValidator_Schema_RefPlainNameOpenApi | schema_validate | schema_validate::SchemaValidator_Schema_RefPlainNameOpenApi_rust | | |
| SchemaValidator.Schema_RefPlainNameRemote | schema_validate_mirror::SchemaValidator_Schema_RefPlainNameRemote | schema_validate | schema_validate::SchemaValidator_Schema_RefPlainNameRemote_rust | | |
| SchemaValidator.Schema_RefPointerInvalid | schema_validate_mirror::SchemaValidator_Schema_RefPointerInvalid | schema_validate | schema_validate::SchemaValidator_Schema_RefPointerInvalid_rust | | |
| SchemaValidator.Schema_RefPointerInvalidRemote | schema_validate_mirror::SchemaValidator_Schema_RefPointerInvalidRemote | schema_validate | schema_validate::SchemaValidator_Schema_RefPointerInvalidRemote_rust | | |
| SchemaValidator.Schema_RefUnknownPlainName | schema_validate_mirror::SchemaValidator_Schema_RefUnknownPlainName | schema_validate | schema_validate::SchemaValidator_Schema_RefUnknownPlainName_rust | | |
| SchemaValidator.Schema_RefUnknownPointer | schema_validate_mirror::SchemaValidator_Schema_RefUnknownPointer | schema_validate | schema_validate::SchemaValidator_Schema_RefUnknownPointer_rust | | |
| SchemaValidator.Schema_RefUnknownPointerRemote | schema_validate_mirror::SchemaValidator_Schema_RefUnknownPointerRemote | schema_validate | schema_validate::SchemaValidator_Schema_RefUnknownPointerRemote_rust | | |
| SchemaValidator.Schema_StartUnknown | schema_validate_mirror::SchemaValidator_Schema_StartUnknown | schema_validate | schema_validate::SchemaValidator_Schema_StartUnknown_rust | | |
| SchemaValidator.Schema_SupportedDraft4 | schema_validate_mirror::SchemaValidator_Schema_SupportedDraft4 | schema_validate | schema_validate::SchemaValidator_Schema_SupportedDraft4_rust | | |
| SchemaValidator.Schema_SupportedDraft4NoFrag | schema_validate_mirror::SchemaValidator_Schema_SupportedDraft4NoFrag | schema_validate | schema_validate::SchemaValidator_Schema_SupportedDraft4NoFrag_rust | | |
| SchemaValidator.Schema_SupportedDraft5 | schema_validate_mirror::SchemaValidator_Schema_SupportedDraft5 | schema_validate | schema_validate::SchemaValidator_Schema_SupportedDraft5_rust | | |
| SchemaValidator.Schema_SupportedDraft5NoFrag | schema_validate_mirror::SchemaValidator_Schema_SupportedDraft5NoFrag | schema_validate | schema_validate::SchemaValidator_Schema_SupportedDraft5NoFrag_rust | | |
| SchemaValidator.Schema_SupportedDraft5Static | schema_validate_mirror::SchemaValidator_Schema_SupportedDraft5Static | schema_validate | schema_validate::SchemaValidator_Schema_SupportedDraft5Static_rust | | |
| SchemaValidator.Schema_SupportedDraftOverride | schema_validate_mirror::SchemaValidator_Schema_SupportedDraftOverride | schema_validate | schema_validate::SchemaValidator_Schema_SupportedDraftOverride_rust | | |
| SchemaValidator.Schema_SupportedNoSpec | schema_validate_mirror::SchemaValidator_Schema_SupportedNoSpec | schema_validate | schema_validate::SchemaValidator_Schema_SupportedNoSpec_rust | | |
| SchemaValidator.Schema_SupportedNoSpecStatic | schema_validate_mirror::SchemaValidator_Schema_SupportedNoSpecStatic | schema_validate | schema_validate::SchemaValidator_Schema_SupportedNoSpecStatic_rust | | |
| SchemaValidator.Schema_SupportedNotObject | schema_validate_mirror::SchemaValidator_Schema_SupportedNotObject | schema_validate | schema_validate::SchemaValidator_Schema_SupportedNotObject_rust | | |
| SchemaValidator.Schema_SupportedVersion20 | schema_validate_mirror::SchemaValidator_Schema_SupportedVersion20 | schema_validate | schema_validate::SchemaValidator_Schema_SupportedVersion20_rust | | |
| SchemaValidator.Schema_SupportedVersion20Static | schema_validate_mirror::SchemaValidator_Schema_SupportedVersion20Static | schema_validate | schema_validate::SchemaValidator_Schema_SupportedVersion20Static_rust | | |
| SchemaValidator.Schema_SupportedVersion30x | schema_validate_mirror::SchemaValidator_Schema_SupportedVersion30x | schema_validate | schema_validate::SchemaValidator_Schema_SupportedVersion30x_rust | | |
| SchemaValidator.Schema_SupportedVersionOverride | schema_validate_mirror::SchemaValidator_Schema_SupportedVersionOverride | schema_validate | schema_validate::SchemaValidator_Schema_SupportedVersionOverride_rust | | |
| SchemaValidator.Schema_UnknownDraft | schema_validate_mirror::SchemaValidator_Schema_UnknownDraft | schema_validate | schema_validate::SchemaValidator_Schema_UnknownDraft_rust | | |
| SchemaValidator.Schema_UnknownDraftNotString | schema_validate_mirror::SchemaValidator_Schema_UnknownDraftNotString | schema_validate | schema_validate::SchemaValidator_Schema_UnknownDraftNotString_rust | | |
| SchemaValidator.Schema_UnknownDraftOverride | schema_validate_mirror::SchemaValidator_Schema_UnknownDraftOverride | schema_validate | schema_validate::SchemaValidator_Schema_UnknownDraftOverride_rust | | |
| SchemaValidator.Schema_UnknownVersion | schema_validate_mirror::SchemaValidator_Schema_UnknownVersion | schema_validate | schema_validate::SchemaValidator_Schema_UnknownVersion_rust | | |
| SchemaValidator.Schema_UnknownVersionNotString | schema_validate_mirror::SchemaValidator_Schema_UnknownVersionNotString | schema_validate | schema_validate::SchemaValidator_Schema_UnknownVersionNotString_rust | | |
| SchemaValidator.Schema_UnknownVersionOverride | schema_validate_mirror::SchemaValidator_Schema_UnknownVersionOverride | schema_validate | schema_validate::SchemaValidator_Schema_UnknownVersionOverride_rust | | |
| SchemaValidator.Schema_UnknownVersionShort | schema_validate_mirror::SchemaValidator_Schema_UnknownVersionShort | schema_validate | schema_validate::SchemaValidator_Schema_UnknownVersionShort_rust | | |
| SchemaValidator.Schema_UnsupportedDraft2019_09 | schema_validate_mirror::SchemaValidator_Schema_UnsupportedDraft2019_09 | schema_validate | schema_validate::SchemaValidator_Schema_UnsupportedDraft2019_09_rust | | |
| SchemaValidator.Schema_UnsupportedDraft2020_12 | schema_validate_mirror::SchemaValidator_Schema_UnsupportedDraft2020_12 | schema_validate | schema_validate::SchemaValidator_Schema_UnsupportedDraft2020_12_rust | | |
| SchemaValidator.Schema_UnsupportedDraft3 | schema_validate_mirror::SchemaValidator_Schema_UnsupportedDraft3 | schema_validate | schema_validate::SchemaValidator_Schema_UnsupportedDraft3_rust | | |
| SchemaValidator.Schema_UnsupportedDraft6 | schema_validate_mirror::SchemaValidator_Schema_UnsupportedDraft6 | schema_validate | schema_validate::SchemaValidator_Schema_UnsupportedDraft6_rust | | |
| SchemaValidator.Schema_UnsupportedDraft7 | schema_validate_mirror::SchemaValidator_Schema_UnsupportedDraft7 | schema_validate | schema_validate::SchemaValidator_Schema_UnsupportedDraft7_rust | | |
| SchemaValidator.Schema_UnsupportedDraftOverride | schema_validate_mirror::SchemaValidator_Schema_UnsupportedDraftOverride | schema_validate | schema_validate::SchemaValidator_Schema_UnsupportedDraftOverride_rust | | |
| SchemaValidator.Schema_UnsupportedVersion31 | schema_validate_mirror::SchemaValidator_Schema_UnsupportedVersion31 | schema_validate | schema_validate::SchemaValidator_Schema_UnsupportedVersion31_rust | | |
| SchemaValidator.Schema_UnsupportedVersionOverride | schema_validate_mirror::SchemaValidator_Schema_UnsupportedVersionOverride | schema_validate | schema_validate::SchemaValidator_Schema_UnsupportedVersionOverride_rust | | |
| SchemaValidator.SchemaPointer | schema_validate_mirror::SchemaValidator_SchemaPointer | schema_validate | schema_validate::SchemaValidator_SchemaPointer_rust | | |
| SchemaValidator.String | schema_validate_mirror::SchemaValidator_String | schema_validate | schema_validate::SchemaValidator_String_rust | | |
| SchemaValidator.String_LengthRange | schema_validate_mirror::SchemaValidator_String_LengthRange | schema_validate | schema_validate::SchemaValidator_String_LengthRange_rust | | |
| SchemaValidator.String_Pattern | schema_validate_mirror::SchemaValidator_String_Pattern | schema_validate | schema_validate::SchemaValidator_String_Pattern_rust | | |
| SchemaValidator.String_Pattern_Invalid | schema_validate_mirror::SchemaValidator_String_Pattern_Invalid | schema_validate | schema_validate::SchemaValidator_String_Pattern_Invalid_rust | | |
| SchemaValidator.TestSuite | schema_validate_mirror::SchemaValidator_TestSuite | schema_validate | schema_validate::SchemaValidator_TestSuite_rust | | |
| SchemaValidator.Typeless | schema_validate_mirror::SchemaValidator_Typeless | schema_validate | schema_validate::SchemaValidator_Typeless_rust | | |
| SchemaValidator.UnknownValidationError | schema_validate_mirror::SchemaValidator_UnknownValidationError | schema_validate | schema_validate::SchemaValidator_UnknownValidationError_rust | | |
| SchemaValidator.ValidateMetaSchema | schema_validate_mirror::SchemaValidator_ValidateMetaSchema | schema_validate | schema_validate::SchemaValidator_ValidateMetaSchema_rust | | |
| SchemaValidator.ValidateMetaSchema_UTF16 | schema_validate_mirror::SchemaValidator_ValidateMetaSchema_UTF16 | schema_validate | schema_validate::SchemaValidator_ValidateMetaSchema_UTF16_rust | | |
| SchemaValidator.WriteOnlyWhenReading | schema_validate_mirror::SchemaValidator_WriteOnlyWhenReading | schema_validate | schema_validate::SchemaValidator_WriteOnlyWhenReading_rust | | |#### 4.3 迁移策略建议

| 测试类别 | 进入 L2 优先级建议 | 原因 |
|----------|--------------------|------|
| 基础关键字 | high | Schema 使用最频繁的部分，应优先完成孪生验证。 |
| 组合关键字 | high | 行为复杂，易出现差异。 |
| `$ref`/远程引用 | high | 核心扩展功能，重要程度高。 |
| draft/version | medium | 规范兼容性重要，但可在核心行为稳定后逐步迁移。 |
| 流式校验 | medium | 与 SAX/Writer 集成，需要在这些 feature 稳定后迁移。 |

### 4.4 孪生测试层交付件

| 交付件 | 说明 |
|--------|------|
| `reports/schema-validate.rust.junit.xml` | 孪生层执行结果。 |
| `reports/schema-validate.parity.json` | 镜像 vs 孪生 一致性报告。 |
| `migrations/schema-validate.mirror_to_rust.json` | 镜像测试 -> 孪生测试映射表。 |
| `rapidjson-rs/tests/rust/schema_validate.rs` | Schema 孪生测试源文件。 |

---

## 变更历史

| 版本 | 日期 | 变更内容 | 变更人 |
|------|------|----------|--------|
| `0.1.0-draft` | `2026-05-14` | 初始化 FEAT-006 `schema-validate` feature 级测试设计文档。 | `TBD` |
