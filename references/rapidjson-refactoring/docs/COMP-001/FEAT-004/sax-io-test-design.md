# Feature 级测试设计文档

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
| component 设计追溯 | [`rapidjson-rs-test-design.md`](../rapidjson-rs-test-design.md) |
| 对应开发文档 | [`sax-io-dev-design.md` 开发设计文档](./sax-io-dev-design.md) |
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
| crates.io 使用策略 | SAX/Writer 测试仅使用 gtest 与 `cargo test`，不引入第三方 Rust 测试/基准框架。 |
| 对当前 feature 技术选型的影响 | 流式解析与生成的所有测试基于 legacy gtest 与 Rust 自测实现，不依赖其他 JSON 库或框架进行对比。 |

### 1.1 测试目标

**本 feature 测试重点:**
- 验证 `Reader`/`Writer` 在事件序列、错误处理和宽松语法支持方面与 C++ 行为等价。
- 覆盖需求文档 2.4 中 SAX 解析/生成的所有主要场景，包括迭代式解析和事件过滤中间层。
- 验证 `cursorstreamwrapper` 等流包装在 SAX 场景中的行为与 C++ 相同。

| 范围类别 | 内容说明 | 备注 |
|----------|----------|------|
| 包含范围 | SAX 解析/生成、迭代式解析、多文档流、宽松语法（注释/尾逗号等） | 对应需求文档 2.4 及相关宽松语法需求 |
| 排除范围 | DOM 行为和 Schema 校验（仅通过 Handler 接口验证事件序列） | 由其它 feature 测试承担 |

### 1.2 测试工具链

| 工具 | 用途 | 版本 |
|------|------|------|
| C/C++ gtest | 执行 `readertest.cpp`、`writertest.cpp`、`cursorstreamwrappertest.cpp`、`jsoncheckertest.cpp` 中 SAX 相关测试 | 与 legacy RapidJSON 项目一致 |
| C/C++ 构建系统（如 CMake） | 构建 legacy SAX 实现与测试 | 同 legacy 项目 |
| `cargo test` | 执行 Rust SAX 镜像与孪生测试 | 与 workspace Rust 版本一致 |
| Python 3 | 如需，可编写辅助脚本生成结构化 diff 报告或性能对比数据 | 可选 |

### 1.3 测试环境

沿用 component 级测试环境，本 feature 无额外特殊要求。本节**无特殊设计**。

### 1.4 三层测试防护网摘要

| 层级 | 层级名称 | 当前 feature 覆盖说明 | 关联章节 |
|------|----------|-------------------|----------|
| L0 | 基线层 | 通过 `readertest.cpp`、`writertest.cpp`、`cursorstreamwrappertest.cpp`、`jsoncheckertest.cpp` 及 perftest 中 SAX/Writer 用例冻结 C++ 行为。 | [2](#2-基线层) |
| L1 | 镜像测试层 | 为上述 gtest 编写 Rust 镜像测试，通过 FFI 调用 C++ Reader/Writer 实现。 | [3](#3-镜像测试层) |
| L2 | 孪生测试层 | 基于 `rapidjson-rs` Reader/Writer 实现编写纯 Rust 测试，与镜像测试并跑验证行为一致。 | [4](#4-孪生测试层) |

---

## 2. 基线层

### 2.1 基线层结构

```text
rapidjson-refactoring/
├── rapidjson_legacy/
│   └── test/
│       ├── unittest/
│       │   ├── readertest.cpp                    # Reader 行为测试
│       │   ├── writertest.cpp                    # Writer 行为测试
│       │   ├── cursorstreamwrappertest.cpp       # CursorStreamWrapper 行为测试
│       │   └── jsoncheckertest.cpp               # JSONChecker 行为测试（与 SAX 密切相关）
│       └── perftest/
│           └── rapidjsontest.cpp                 # 含 SAX/Writer 性能测试
│
├── inventory/
│   └── sax-io.legacy_tests.json                  # SAX 相关 Legacy 测试资产清单
│
├── baseline/
│   └── sax-io.golden_samples.jsonl               # SAX 行为黄金样本（输入/事件序列/输出）
│
└── reports/
    ├── sax-io.legacy.junit.xml                   # Legacy SAX 测试执行结果
    └── sax-io.legacy.coverage.xml                # Legacy SAX 测试覆盖率报告
```

### 2.2 基线测试清单

| 测试 ID | file | oracle 来源 |
|---------|------|-------------|
| CursorStreamWrapper.MissingArrayBracket | test/unittest/cursorstreamwrappertest.cpp:81 | legacy_test |
| CursorStreamWrapper.MissingArrayComma | test/unittest/cursorstreamwrappertest.cpp:90 | legacy_test |
| CursorStreamWrapper.MissingColon | test/unittest/cursorstreamwrappertest.cpp:54 | legacy_test |
| CursorStreamWrapper.MissingComma | test/unittest/cursorstreamwrappertest.cpp:72 | legacy_test |
| CursorStreamWrapper.MissingFirstBracket | test/unittest/cursorstreamwrappertest.cpp:36 | legacy_test |
| CursorStreamWrapper.MissingLastArrayBracket | test/unittest/cursorstreamwrappertest.cpp:99 | legacy_test |
| CursorStreamWrapper.MissingLastBracket | test/unittest/cursorstreamwrappertest.cpp:108 | legacy_test |
| CursorStreamWrapper.MissingQuotes | test/unittest/cursorstreamwrappertest.cpp:45 | legacy_test |
| CursorStreamWrapper.MissingSecondQuotes | test/unittest/cursorstreamwrappertest.cpp:63 | legacy_test |
| JsonChecker.Reader | test/unittest/jsoncheckertest.cpp:69 | legacy_test |
| PrettyWriter.Basic | test/unittest/prettywritertest.cpp:60 | legacy_test |
| PrettyWriter.FileWriteStream | test/unittest/prettywritertest.cpp:167 | legacy_test |
| PrettyWriter.FormatOptions | test/unittest/prettywritertest.cpp:69 | legacy_test |
| PrettyWriter.Inf | test/unittest/prettywritertest.cpp:281 | legacy_test |
| PrettyWriter.InvalidEventSequence | test/unittest/prettywritertest.cpp:211 | legacy_test |
| PrettyWriter.Issue_1336 | test/unittest/prettywritertest.cpp:342 | legacy_test |
| PrettyWriter.Issue_889 | test/unittest/prettywritertest.cpp:305 | legacy_test |
| PrettyWriter.MoveCtor | test/unittest/prettywritertest.cpp:329 | legacy_test |
| PrettyWriter.NaN | test/unittest/prettywritertest.cpp:262 | legacy_test |
| PrettyWriter.OStreamWrapper | test/unittest/prettywritertest.cpp:151 | legacy_test |
| PrettyWriter.RawValue | test/unittest/prettywritertest.cpp:192 | legacy_test |
| PrettyWriter.SetIndent | test/unittest/prettywritertest.cpp:79 | legacy_test |
| PrettyWriter.String | test/unittest/prettywritertest.cpp:106 | legacy_test |
| PrettyWriter.String_STDSTRING | test/unittest/prettywritertest.cpp:116 | legacy_test |
| Reader.BaseReaderHandler_Default | test/unittest/readertest.cpp:1754 | legacy_test |
| Reader.CustomStringStream | test/unittest/readertest.cpp:1373 | legacy_test |
| Reader.EmptyExceptForCommaErrors | test/unittest/readertest.cpp:2260 | legacy_test |
| Reader.EmptyExceptForCommaErrorsIterative | test/unittest/readertest.cpp:2264 | legacy_test |
| Reader.EofAfterOneLineComment | test/unittest/readertest.cpp:1891 | legacy_test |
| Reader.EscapedApostrophe | test/unittest/readertest.cpp:2368 | legacy_test |
| Reader.IncompleteMultilineComment | test/unittest/readertest.cpp:1901 | legacy_test |
| Reader.IncompleteMultilineComment2 | test/unittest/readertest.cpp:1911 | legacy_test |
| Reader.InlineCommentsAreDisabledByDefault | test/unittest/readertest.cpp:1859 | legacy_test |
| Reader.IterativeParsing_Count | test/unittest/readertest.cpp:1606 | legacy_test |
| Reader.IterativeParsing_ErrorHandling | test/unittest/readertest.cpp:1471 | legacy_test |
| Reader.IterativeParsing_General | test/unittest/readertest.cpp:1569 | legacy_test |
| Reader.IterativeParsing_ShortCircuit | test/unittest/readertest.cpp:1703 | legacy_test |
| Reader.IterativePullParsing_General | test/unittest/readertest.cpp:1641 | legacy_test |
| Reader.MultipleTrailingCommaErrors | test/unittest/readertest.cpp:2228 | legacy_test |
| Reader.MultipleTrailingCommaErrorsIterative | test/unittest/readertest.cpp:2232 | legacy_test |
| Reader.NumbersAsStrings | test/unittest/readertest.cpp:1961 | legacy_test |
| Reader.NumbersAsStringsWChar_t | test/unittest/readertest.cpp:2073 | legacy_test |
| Reader.OnelineCommentsAreDisabledByDefault | test/unittest/readertest.cpp:1882 | legacy_test |
| Reader.Parse_EmptyObject | test/unittest/readertest.cpp:1189 | legacy_test |
| Reader.Parse_IStreamWrapper_StringStream | test/unittest/readertest.cpp:1444 | legacy_test |
| Reader.Parse_MultipleRoot | test/unittest/readertest.cpp:1222 | legacy_test |
| Reader.ParseArray | test/unittest/readertest.cpp:1074 | legacy_test |
| Reader.ParseArray_Error | test/unittest/readertest.cpp:1084 | legacy_test |
| Reader.ParseComments | test/unittest/readertest.cpp:1808 | legacy_test |
| Reader.ParseDocument_Error | test/unittest/readertest.cpp:1267 | legacy_test |
| Reader.ParseEmptyArray | test/unittest/readertest.cpp:1064 | legacy_test |
| Reader.ParseEmptyInlineComment | test/unittest/readertest.cpp:1826 | legacy_test |
| Reader.ParseEmptyOnelineComment | test/unittest/readertest.cpp:1836 | legacy_test |
| Reader.ParseFalse | test/unittest/readertest.cpp:60 | legacy_test |
| Reader.ParseInsitu_MultipleRoot | test/unittest/readertest.cpp:1245 | legacy_test |
| Reader.ParseInsituIterative_MultipleRoot | test/unittest/readertest.cpp:1249 | legacy_test |
| Reader.ParseIterative_MultipleRoot | test/unittest/readertest.cpp:1226 | legacy_test |
| Reader.ParseMultipleCommentsInARow | test/unittest/readertest.cpp:1846 | legacy_test |
| Reader.ParseNanAndInfinity | test/unittest/readertest.cpp:2298 | legacy_test |
| Reader.ParseNumber_FullPrecisionDouble | test/unittest/readertest.cpp:592 | legacy_test |
| Reader.ParseNumber_Integer | test/unittest/readertest.cpp:113 | legacy_test |
| Reader.ParseNumber_NormalPrecisionDouble | test/unittest/readertest.cpp:588 | legacy_test |
| Reader.ParseNumber_NormalPrecisionError | test/unittest/readertest.cpp:596 | legacy_test |
| Reader.ParseNumberError_FullPrecisionDouble | test/unittest/readertest.cpp:752 | legacy_test |
| Reader.ParseNumberError_NormalPrecisionDouble | test/unittest/readertest.cpp:748 | legacy_test |
| Reader.ParseObject | test/unittest/readertest.cpp:1155 | legacy_test |
| Reader.ParseObject_Error | test/unittest/readertest.cpp:1289 | legacy_test |
| Reader.ParseString | test/unittest/readertest.cpp:783 | legacy_test |
| Reader.ParseString_Error | test/unittest/readertest.cpp:908 | legacy_test |
| Reader.ParseString_NonDestructive | test/unittest/readertest.cpp:890 | legacy_test |
| Reader.ParseString_Transcoding | test/unittest/readertest.cpp:868 | legacy_test |
| Reader.ParseString_TranscodingWithValidation | test/unittest/readertest.cpp:879 | legacy_test |
| Reader.ParseTerminationByHandler | test/unittest/readertest.cpp:1788 | legacy_test |
| Reader.ParseTrue | test/unittest/readertest.cpp:52 | legacy_test |
| Reader.ParseValue_Error | test/unittest/readertest.cpp:1280 | legacy_test |
| Reader.SkipWhitespace | test/unittest/readertest.cpp:1324 | legacy_test |
| Reader.TrailingCommaHandlerTermination | test/unittest/readertest.cpp:2290 | legacy_test |
| Reader.TrailingCommaHandlerTerminationIterative | test/unittest/readertest.cpp:2294 | legacy_test |
| Reader.TrailingCommas | test/unittest/readertest.cpp:2195 | legacy_test |
| Reader.TrailingCommasIterative | test/unittest/readertest.cpp:2199 | legacy_test |
| Reader.UnrecognizedComment | test/unittest/readertest.cpp:1921 | legacy_test |
| Write.RawValue_Issue1152 | test/unittest/writertest.cpp:571 | legacy_test |
| Writer.AssertIncorrectArrayLevel | test/unittest/writertest.cpp:263 | legacy_test |
| Writer.AssertIncorrectEndArray | test/unittest/writertest.cpp:278 | legacy_test |
| Writer.AssertIncorrectEndObject | test/unittest/writertest.cpp:271 | legacy_test |
| Writer.AssertIncorrectObjectLevel | test/unittest/writertest.cpp:255 | legacy_test |
| Writer.AssertMultipleRoot | test/unittest/writertest.cpp:306 | legacy_test |
| Writer.AssertObjectKeyNotString | test/unittest/writertest.cpp:285 | legacy_test |
| Writer.AssertRootMayBeAnyValue | test/unittest/writertest.cpp:236 | legacy_test |
| Writer.Compact | test/unittest/writertest.cpp:30 | legacy_test |
| Writer.Double | test/unittest/writertest.cpp:129 | legacy_test |
| Writer.Inf | test/unittest/writertest.cpp:515 | legacy_test |
| Writer.InfToNull | test/unittest/writertest.cpp:539 | legacy_test |
| Writer.Int | test/unittest/writertest.cpp:64 | legacy_test |
| Writer.Int64 | test/unittest/writertest.cpp:78 | legacy_test |
| Writer.InvalidEncoding | test/unittest/writertest.cpp:377 | legacy_test |
| Writer.InvalidEventSequence | test/unittest/writertest.cpp:433 | legacy_test |
| Writer.Issue_889 | test/unittest/writertest.cpp:103 | legacy_test |
| Writer.MoveCtor | test/unittest/writertest.cpp:617 | legacy_test |
| Writer.NaN | test/unittest/writertest.cpp:484 | legacy_test |
| Writer.NaNToNull | test/unittest/writertest.cpp:503 | legacy_test |
| Writer.OStreamWrapper | test/unittest/writertest.cpp:221 | legacy_test |
| Writer.RawValue | test/unittest/writertest.cpp:557 | legacy_test |
| Writer.Root | test/unittest/writertest.cpp:54 | legacy_test |
| Writer.RootArrayIsComplete | test/unittest/writertest.cpp:342 | legacy_test |
| Writer.RootObjectIsComplete | test/unittest/writertest.cpp:328 | legacy_test |
| Writer.RootValueIsComplete | test/unittest/writertest.cpp:356 | legacy_test |
| Writer.ScanWriteUnescapedString | test/unittest/writertest.cpp:116 | legacy_test |
| Writer.String | test/unittest/writertest.cpp:88 | legacy_test |
| Writer.Transcode | test/unittest/writertest.cpp:160 | legacy_test |
| Writer.UInt | test/unittest/writertest.cpp:70 | legacy_test |
| Writer.Uint64 | test/unittest/writertest.cpp:83 | legacy_test |
| Writer.ValidateEncoding | test/unittest/writertest.cpp:406 | legacy_test |## 3. 镜像测试层

### 3.1 镜像测试层结构

```text
rapidjson-refactoring/
├── rapidjson_refactoring_sys/
│   └── sax_ffi/                                   # SAX 相关 FFI 适配代码（规划）
│
├── migrations/
│   └── sax-io.legacy_to_mirror.json               # gtest -> Rust 镜像测试映射表
│
├── reports/
│   └── sax-io.mirror.junit.xml                    # 镜像层执行结果
│
└── rapidjson-rs/
    └── tests/
        └── mirrors/
            └── sax_io_mirror.rs                   # SAX 的 Rust 镜像测试源文件
```

### 3.2 镜像测试清单

| 测试 ID | 涉及 C/C++ 数据结构/接口 | 镜像测试 ID（Rust Test 路径） | 备注 |
|---------|-------------------------|------------------------------|------|
| CursorStreamWrapper.MissingArrayBracket | test/unittest/cursorstreamwrappertest.cpp:81 | sax_io_mirror::CursorStreamWrapper_MissingArrayBracket | |
| CursorStreamWrapper.MissingArrayComma | test/unittest/cursorstreamwrappertest.cpp:90 | sax_io_mirror::CursorStreamWrapper_MissingArrayComma | |
| CursorStreamWrapper.MissingColon | test/unittest/cursorstreamwrappertest.cpp:54 | sax_io_mirror::CursorStreamWrapper_MissingColon | |
| CursorStreamWrapper.MissingComma | test/unittest/cursorstreamwrappertest.cpp:72 | sax_io_mirror::CursorStreamWrapper_MissingComma | |
| CursorStreamWrapper.MissingFirstBracket | test/unittest/cursorstreamwrappertest.cpp:36 | sax_io_mirror::CursorStreamWrapper_MissingFirstBracket | |
| CursorStreamWrapper.MissingLastArrayBracket | test/unittest/cursorstreamwrappertest.cpp:99 | sax_io_mirror::CursorStreamWrapper_MissingLastArrayBracket | |
| CursorStreamWrapper.MissingLastBracket | test/unittest/cursorstreamwrappertest.cpp:108 | sax_io_mirror::CursorStreamWrapper_MissingLastBracket | |
| CursorStreamWrapper.MissingQuotes | test/unittest/cursorstreamwrappertest.cpp:45 | sax_io_mirror::CursorStreamWrapper_MissingQuotes | |
| CursorStreamWrapper.MissingSecondQuotes | test/unittest/cursorstreamwrappertest.cpp:63 | sax_io_mirror::CursorStreamWrapper_MissingSecondQuotes | |
| JsonChecker.Reader | test/unittest/jsoncheckertest.cpp:69 | sax_io_mirror::JsonChecker_Reader | |
| PrettyWriter.Basic | test/unittest/prettywritertest.cpp:60 | sax_io_mirror::PrettyWriter_Basic | |
| PrettyWriter.FileWriteStream | test/unittest/prettywritertest.cpp:167 | sax_io_mirror::PrettyWriter_FileWriteStream | |
| PrettyWriter.FormatOptions | test/unittest/prettywritertest.cpp:69 | sax_io_mirror::PrettyWriter_FormatOptions | |
| PrettyWriter.Inf | test/unittest/prettywritertest.cpp:281 | sax_io_mirror::PrettyWriter_Inf | |
| PrettyWriter.InvalidEventSequence | test/unittest/prettywritertest.cpp:211 | sax_io_mirror::PrettyWriter_InvalidEventSequence | |
| PrettyWriter.Issue_1336 | test/unittest/prettywritertest.cpp:342 | sax_io_mirror::PrettyWriter_Issue_1336 | |
| PrettyWriter.Issue_889 | test/unittest/prettywritertest.cpp:305 | sax_io_mirror::PrettyWriter_Issue_889 | |
| PrettyWriter.MoveCtor | test/unittest/prettywritertest.cpp:329 | sax_io_mirror::PrettyWriter_MoveCtor | |
| PrettyWriter.NaN | test/unittest/prettywritertest.cpp:262 | sax_io_mirror::PrettyWriter_NaN | |
| PrettyWriter.OStreamWrapper | test/unittest/prettywritertest.cpp:151 | sax_io_mirror::PrettyWriter_OStreamWrapper | |
| PrettyWriter.RawValue | test/unittest/prettywritertest.cpp:192 | sax_io_mirror::PrettyWriter_RawValue | |
| PrettyWriter.SetIndent | test/unittest/prettywritertest.cpp:79 | sax_io_mirror::PrettyWriter_SetIndent | |
| PrettyWriter.String | test/unittest/prettywritertest.cpp:106 | sax_io_mirror::PrettyWriter_String | |
| PrettyWriter.String_STDSTRING | test/unittest/prettywritertest.cpp:116 | sax_io_mirror::PrettyWriter_String_STDSTRING | |
| Reader.BaseReaderHandler_Default | test/unittest/readertest.cpp:1754 | sax_io_mirror::Reader_BaseReaderHandler_Default | |
| Reader.CustomStringStream | test/unittest/readertest.cpp:1373 | sax_io_mirror::Reader_CustomStringStream | |
| Reader.EmptyExceptForCommaErrors | test/unittest/readertest.cpp:2260 | sax_io_mirror::Reader_EmptyExceptForCommaErrors | |
| Reader.EmptyExceptForCommaErrorsIterative | test/unittest/readertest.cpp:2264 | sax_io_mirror::Reader_EmptyExceptForCommaErrorsIterative | |
| Reader.EofAfterOneLineComment | test/unittest/readertest.cpp:1891 | sax_io_mirror::Reader_EofAfterOneLineComment | |
| Reader.EscapedApostrophe | test/unittest/readertest.cpp:2368 | sax_io_mirror::Reader_EscapedApostrophe | |
| Reader.IncompleteMultilineComment | test/unittest/readertest.cpp:1901 | sax_io_mirror::Reader_IncompleteMultilineComment | |
| Reader.IncompleteMultilineComment2 | test/unittest/readertest.cpp:1911 | sax_io_mirror::Reader_IncompleteMultilineComment2 | |
| Reader.InlineCommentsAreDisabledByDefault | test/unittest/readertest.cpp:1859 | sax_io_mirror::Reader_InlineCommentsAreDisabledByDefault | |
| Reader.IterativeParsing_Count | test/unittest/readertest.cpp:1606 | sax_io_mirror::Reader_IterativeParsing_Count | |
| Reader.IterativeParsing_ErrorHandling | test/unittest/readertest.cpp:1471 | sax_io_mirror::Reader_IterativeParsing_ErrorHandling | |
| Reader.IterativeParsing_General | test/unittest/readertest.cpp:1569 | sax_io_mirror::Reader_IterativeParsing_General | |
| Reader.IterativeParsing_ShortCircuit | test/unittest/readertest.cpp:1703 | sax_io_mirror::Reader_IterativeParsing_ShortCircuit | |
| Reader.IterativePullParsing_General | test/unittest/readertest.cpp:1641 | sax_io_mirror::Reader_IterativePullParsing_General | |
| Reader.MultipleTrailingCommaErrors | test/unittest/readertest.cpp:2228 | sax_io_mirror::Reader_MultipleTrailingCommaErrors | |
| Reader.MultipleTrailingCommaErrorsIterative | test/unittest/readertest.cpp:2232 | sax_io_mirror::Reader_MultipleTrailingCommaErrorsIterative | |
| Reader.NumbersAsStrings | test/unittest/readertest.cpp:1961 | sax_io_mirror::Reader_NumbersAsStrings | |
| Reader.NumbersAsStringsWChar_t | test/unittest/readertest.cpp:2073 | sax_io_mirror::Reader_NumbersAsStringsWChar_t | |
| Reader.OnelineCommentsAreDisabledByDefault | test/unittest/readertest.cpp:1882 | sax_io_mirror::Reader_OnelineCommentsAreDisabledByDefault | |
| Reader.Parse_EmptyObject | test/unittest/readertest.cpp:1189 | sax_io_mirror::Reader_Parse_EmptyObject | |
| Reader.Parse_IStreamWrapper_StringStream | test/unittest/readertest.cpp:1444 | sax_io_mirror::Reader_Parse_IStreamWrapper_StringStream | |
| Reader.Parse_MultipleRoot | test/unittest/readertest.cpp:1222 | sax_io_mirror::Reader_Parse_MultipleRoot | |
| Reader.ParseArray | test/unittest/readertest.cpp:1074 | sax_io_mirror::Reader_ParseArray | |
| Reader.ParseArray_Error | test/unittest/readertest.cpp:1084 | sax_io_mirror::Reader_ParseArray_Error | |
| Reader.ParseComments | test/unittest/readertest.cpp:1808 | sax_io_mirror::Reader_ParseComments | |
| Reader.ParseDocument_Error | test/unittest/readertest.cpp:1267 | sax_io_mirror::Reader_ParseDocument_Error | |
| Reader.ParseEmptyArray | test/unittest/readertest.cpp:1064 | sax_io_mirror::Reader_ParseEmptyArray | |
| Reader.ParseEmptyInlineComment | test/unittest/readertest.cpp:1826 | sax_io_mirror::Reader_ParseEmptyInlineComment | |
| Reader.ParseEmptyOnelineComment | test/unittest/readertest.cpp:1836 | sax_io_mirror::Reader_ParseEmptyOnelineComment | |
| Reader.ParseFalse | test/unittest/readertest.cpp:60 | sax_io_mirror::Reader_ParseFalse | |
| Reader.ParseInsitu_MultipleRoot | test/unittest/readertest.cpp:1245 | sax_io_mirror::Reader_ParseInsitu_MultipleRoot | |
| Reader.ParseInsituIterative_MultipleRoot | test/unittest/readertest.cpp:1249 | sax_io_mirror::Reader_ParseInsituIterative_MultipleRoot | |
| Reader.ParseIterative_MultipleRoot | test/unittest/readertest.cpp:1226 | sax_io_mirror::Reader_ParseIterative_MultipleRoot | |
| Reader.ParseMultipleCommentsInARow | test/unittest/readertest.cpp:1846 | sax_io_mirror::Reader_ParseMultipleCommentsInARow | |
| Reader.ParseNanAndInfinity | test/unittest/readertest.cpp:2298 | sax_io_mirror::Reader_ParseNanAndInfinity | |
| Reader.ParseNumber_FullPrecisionDouble | test/unittest/readertest.cpp:592 | sax_io_mirror::Reader_ParseNumber_FullPrecisionDouble | |
| Reader.ParseNumber_Integer | test/unittest/readertest.cpp:113 | sax_io_mirror::Reader_ParseNumber_Integer | |
| Reader.ParseNumber_NormalPrecisionDouble | test/unittest/readertest.cpp:588 | sax_io_mirror::Reader_ParseNumber_NormalPrecisionDouble | |
| Reader.ParseNumber_NormalPrecisionError | test/unittest/readertest.cpp:596 | sax_io_mirror::Reader_ParseNumber_NormalPrecisionError | |
| Reader.ParseNumberError_FullPrecisionDouble | test/unittest/readertest.cpp:752 | sax_io_mirror::Reader_ParseNumberError_FullPrecisionDouble | |
| Reader.ParseNumberError_NormalPrecisionDouble | test/unittest/readertest.cpp:748 | sax_io_mirror::Reader_ParseNumberError_NormalPrecisionDouble | |
| Reader.ParseObject | test/unittest/readertest.cpp:1155 | sax_io_mirror::Reader_ParseObject | |
| Reader.ParseObject_Error | test/unittest/readertest.cpp:1289 | sax_io_mirror::Reader_ParseObject_Error | |
| Reader.ParseString | test/unittest/readertest.cpp:783 | sax_io_mirror::Reader_ParseString | |
| Reader.ParseString_Error | test/unittest/readertest.cpp:908 | sax_io_mirror::Reader_ParseString_Error | |
| Reader.ParseString_NonDestructive | test/unittest/readertest.cpp:890 | sax_io_mirror::Reader_ParseString_NonDestructive | |
| Reader.ParseString_Transcoding | test/unittest/readertest.cpp:868 | sax_io_mirror::Reader_ParseString_Transcoding | |
| Reader.ParseString_TranscodingWithValidation | test/unittest/readertest.cpp:879 | sax_io_mirror::Reader_ParseString_TranscodingWithValidation | |
| Reader.ParseTerminationByHandler | test/unittest/readertest.cpp:1788 | sax_io_mirror::Reader_ParseTerminationByHandler | |
| Reader.ParseTrue | test/unittest/readertest.cpp:52 | sax_io_mirror::Reader_ParseTrue | |
| Reader.ParseValue_Error | test/unittest/readertest.cpp:1280 | sax_io_mirror::Reader_ParseValue_Error | |
| Reader.SkipWhitespace | test/unittest/readertest.cpp:1324 | sax_io_mirror::Reader_SkipWhitespace | |
| Reader.TrailingCommaHandlerTermination | test/unittest/readertest.cpp:2290 | sax_io_mirror::Reader_TrailingCommaHandlerTermination | |
| Reader.TrailingCommaHandlerTerminationIterative | test/unittest/readertest.cpp:2294 | sax_io_mirror::Reader_TrailingCommaHandlerTerminationIterative | |
| Reader.TrailingCommas | test/unittest/readertest.cpp:2195 | sax_io_mirror::Reader_TrailingCommas | |
| Reader.TrailingCommasIterative | test/unittest/readertest.cpp:2199 | sax_io_mirror::Reader_TrailingCommasIterative | |
| Reader.UnrecognizedComment | test/unittest/readertest.cpp:1921 | sax_io_mirror::Reader_UnrecognizedComment | |
| Write.RawValue_Issue1152 | test/unittest/writertest.cpp:571 | sax_io_mirror::Write_RawValue_Issue1152 | |
| Writer.AssertIncorrectArrayLevel | test/unittest/writertest.cpp:263 | sax_io_mirror::Writer_AssertIncorrectArrayLevel | |
| Writer.AssertIncorrectEndArray | test/unittest/writertest.cpp:278 | sax_io_mirror::Writer_AssertIncorrectEndArray | |
| Writer.AssertIncorrectEndObject | test/unittest/writertest.cpp:271 | sax_io_mirror::Writer_AssertIncorrectEndObject | |
| Writer.AssertIncorrectObjectLevel | test/unittest/writertest.cpp:255 | sax_io_mirror::Writer_AssertIncorrectObjectLevel | |
| Writer.AssertMultipleRoot | test/unittest/writertest.cpp:306 | sax_io_mirror::Writer_AssertMultipleRoot | |
| Writer.AssertObjectKeyNotString | test/unittest/writertest.cpp:285 | sax_io_mirror::Writer_AssertObjectKeyNotString | |
| Writer.AssertRootMayBeAnyValue | test/unittest/writertest.cpp:236 | sax_io_mirror::Writer_AssertRootMayBeAnyValue | |
| Writer.Compact | test/unittest/writertest.cpp:30 | sax_io_mirror::Writer_Compact | |
| Writer.Double | test/unittest/writertest.cpp:129 | sax_io_mirror::Writer_Double | |
| Writer.Inf | test/unittest/writertest.cpp:515 | sax_io_mirror::Writer_Inf | |
| Writer.InfToNull | test/unittest/writertest.cpp:539 | sax_io_mirror::Writer_InfToNull | |
| Writer.Int | test/unittest/writertest.cpp:64 | sax_io_mirror::Writer_Int | |
| Writer.Int64 | test/unittest/writertest.cpp:78 | sax_io_mirror::Writer_Int64 | |
| Writer.InvalidEncoding | test/unittest/writertest.cpp:377 | sax_io_mirror::Writer_InvalidEncoding | |
| Writer.InvalidEventSequence | test/unittest/writertest.cpp:433 | sax_io_mirror::Writer_InvalidEventSequence | |
| Writer.Issue_889 | test/unittest/writertest.cpp:103 | sax_io_mirror::Writer_Issue_889 | |
| Writer.MoveCtor | test/unittest/writertest.cpp:617 | sax_io_mirror::Writer_MoveCtor | |
| Writer.NaN | test/unittest/writertest.cpp:484 | sax_io_mirror::Writer_NaN | |
| Writer.NaNToNull | test/unittest/writertest.cpp:503 | sax_io_mirror::Writer_NaNToNull | |
| Writer.OStreamWrapper | test/unittest/writertest.cpp:221 | sax_io_mirror::Writer_OStreamWrapper | |
| Writer.RawValue | test/unittest/writertest.cpp:557 | sax_io_mirror::Writer_RawValue | |
| Writer.Root | test/unittest/writertest.cpp:54 | sax_io_mirror::Writer_Root | |
| Writer.RootArrayIsComplete | test/unittest/writertest.cpp:342 | sax_io_mirror::Writer_RootArrayIsComplete | |
| Writer.RootObjectIsComplete | test/unittest/writertest.cpp:328 | sax_io_mirror::Writer_RootObjectIsComplete | |
| Writer.RootValueIsComplete | test/unittest/writertest.cpp:356 | sax_io_mirror::Writer_RootValueIsComplete | |
| Writer.ScanWriteUnescapedString | test/unittest/writertest.cpp:116 | sax_io_mirror::Writer_ScanWriteUnescapedString | |
| Writer.String | test/unittest/writertest.cpp:88 | sax_io_mirror::Writer_String | |
| Writer.Transcode | test/unittest/writertest.cpp:160 | sax_io_mirror::Writer_Transcode | |
| Writer.UInt | test/unittest/writertest.cpp:70 | sax_io_mirror::Writer_UInt | |
| Writer.Uint64 | test/unittest/writertest.cpp:83 | sax_io_mirror::Writer_Uint64 | |
| Writer.ValidateEncoding | test/unittest/writertest.cpp:406 | sax_io_mirror::Writer_ValidateEncoding | |#### 3.3 迁移策略建议

| 测试 ID | 进入 L1 优先级建议 | 原因 |
|---------|--------------------|------|
| `Reader.*` | high | 解析行为核心，应尽早镜像。 |
| `Writer.*` | high | 输出行为核心，影响多数输出场景。 |
| `CursorStreamWrapper.*` | medium | 支撑特定流场景，重要但次于 Reader/Writer。 |
| `JsonChecker.Reader` | medium | 重要但可在 Reader 核心稳定后迁移。 |
| `RapidJson.Reader*` 性能用例 | medium | 以行为基线为主，性能对比可稍后引入。 |

### 3.4 镜像测试层交付件

| 交付件 | 说明 |
|--------|------|
| `reports/sax-io.mirror.junit.xml` | 镜像层执行结果。 |
| `migrations/sax-io.legacy_to_mirror.json` | gtest -> Rust 镜像测试映射表。 |
| `rapidjson-rs/tests/mirrors/sax_io_mirror.rs` | SAX 镜像测试源文件。 |

---

## 4. 孪生测试层

### 4.1 孪生测试层结构

```text
rapidjson-refactoring/
├── reports/
│   ├── sax-io.rust.junit.xml                      # 孪生层执行结果（Rust Reader/Writer 实现）
│   └── sax-io.parity.json                         # 镜像 vs 孪生 一致性报告
│
├── migrations/
│   └── sax-io.mirror_to_rust.json                 # 镜像测试 -> 孪生测试映射表
│
└── rapidjson-rs/
    └── tests/
        └── rust/
            └── sax_io.rs                          # SAX 的 Rust 原生测试源文件
```

### 4.2 孪生测试清单

| 测试 ID | 镜像测试 ID | 涉及 Rust 数据结构/接口 | 原生测试 ID（Test 路径） | 是否验收 | 备注 |
|---------|-------------|------------------------|---------------------------|----------|------|
| CursorStreamWrapper.MissingArrayBracket | sax_io_mirror::CursorStreamWrapper_MissingArrayBracket | sax_io | sax_io::CursorStreamWrapper_MissingArrayBracket_rust | | |
| CursorStreamWrapper.MissingArrayComma | sax_io_mirror::CursorStreamWrapper_MissingArrayComma | sax_io | sax_io::CursorStreamWrapper_MissingArrayComma_rust | | |
| CursorStreamWrapper.MissingColon | sax_io_mirror::CursorStreamWrapper_MissingColon | sax_io | sax_io::CursorStreamWrapper_MissingColon_rust | | |
| CursorStreamWrapper.MissingComma | sax_io_mirror::CursorStreamWrapper_MissingComma | sax_io | sax_io::CursorStreamWrapper_MissingComma_rust | | |
| CursorStreamWrapper.MissingFirstBracket | sax_io_mirror::CursorStreamWrapper_MissingFirstBracket | sax_io | sax_io::CursorStreamWrapper_MissingFirstBracket_rust | | |
| CursorStreamWrapper.MissingLastArrayBracket | sax_io_mirror::CursorStreamWrapper_MissingLastArrayBracket | sax_io | sax_io::CursorStreamWrapper_MissingLastArrayBracket_rust | | |
| CursorStreamWrapper.MissingLastBracket | sax_io_mirror::CursorStreamWrapper_MissingLastBracket | sax_io | sax_io::CursorStreamWrapper_MissingLastBracket_rust | | |
| CursorStreamWrapper.MissingQuotes | sax_io_mirror::CursorStreamWrapper_MissingQuotes | sax_io | sax_io::CursorStreamWrapper_MissingQuotes_rust | | |
| CursorStreamWrapper.MissingSecondQuotes | sax_io_mirror::CursorStreamWrapper_MissingSecondQuotes | sax_io | sax_io::CursorStreamWrapper_MissingSecondQuotes_rust | | |
| JsonChecker.Reader | sax_io_mirror::JsonChecker_Reader | sax_io | sax_io::JsonChecker_Reader_rust | | |
| PrettyWriter.Basic | sax_io_mirror::PrettyWriter_Basic | sax_io | sax_io::PrettyWriter_Basic_rust | | |
| PrettyWriter.FileWriteStream | sax_io_mirror::PrettyWriter_FileWriteStream | sax_io | sax_io::PrettyWriter_FileWriteStream_rust | | |
| PrettyWriter.FormatOptions | sax_io_mirror::PrettyWriter_FormatOptions | sax_io | sax_io::PrettyWriter_FormatOptions_rust | | |
| PrettyWriter.Inf | sax_io_mirror::PrettyWriter_Inf | sax_io | sax_io::PrettyWriter_Inf_rust | | |
| PrettyWriter.InvalidEventSequence | sax_io_mirror::PrettyWriter_InvalidEventSequence | sax_io | sax_io::PrettyWriter_InvalidEventSequence_rust | | |
| PrettyWriter.Issue_1336 | sax_io_mirror::PrettyWriter_Issue_1336 | sax_io | sax_io::PrettyWriter_Issue_1336_rust | | |
| PrettyWriter.Issue_889 | sax_io_mirror::PrettyWriter_Issue_889 | sax_io | sax_io::PrettyWriter_Issue_889_rust | | |
| PrettyWriter.MoveCtor | sax_io_mirror::PrettyWriter_MoveCtor | sax_io | sax_io::PrettyWriter_MoveCtor_rust | | |
| PrettyWriter.NaN | sax_io_mirror::PrettyWriter_NaN | sax_io | sax_io::PrettyWriter_NaN_rust | | |
| PrettyWriter.OStreamWrapper | sax_io_mirror::PrettyWriter_OStreamWrapper | sax_io | sax_io::PrettyWriter_OStreamWrapper_rust | | |
| PrettyWriter.RawValue | sax_io_mirror::PrettyWriter_RawValue | sax_io | sax_io::PrettyWriter_RawValue_rust | | |
| PrettyWriter.SetIndent | sax_io_mirror::PrettyWriter_SetIndent | sax_io | sax_io::PrettyWriter_SetIndent_rust | | |
| PrettyWriter.String | sax_io_mirror::PrettyWriter_String | sax_io | sax_io::PrettyWriter_String_rust | | |
| PrettyWriter.String_STDSTRING | sax_io_mirror::PrettyWriter_String_STDSTRING | sax_io | sax_io::PrettyWriter_String_STDSTRING_rust | | |
| Reader.BaseReaderHandler_Default | sax_io_mirror::Reader_BaseReaderHandler_Default | sax_io | sax_io::Reader_BaseReaderHandler_Default_rust | | |
| Reader.CustomStringStream | sax_io_mirror::Reader_CustomStringStream | sax_io | sax_io::Reader_CustomStringStream_rust | | |
| Reader.EmptyExceptForCommaErrors | sax_io_mirror::Reader_EmptyExceptForCommaErrors | sax_io | sax_io::Reader_EmptyExceptForCommaErrors_rust | | |
| Reader.EmptyExceptForCommaErrorsIterative | sax_io_mirror::Reader_EmptyExceptForCommaErrorsIterative | sax_io | sax_io::Reader_EmptyExceptForCommaErrorsIterative_rust | | |
| Reader.EofAfterOneLineComment | sax_io_mirror::Reader_EofAfterOneLineComment | sax_io | sax_io::Reader_EofAfterOneLineComment_rust | | |
| Reader.EscapedApostrophe | sax_io_mirror::Reader_EscapedApostrophe | sax_io | sax_io::Reader_EscapedApostrophe_rust | | |
| Reader.IncompleteMultilineComment | sax_io_mirror::Reader_IncompleteMultilineComment | sax_io | sax_io::Reader_IncompleteMultilineComment_rust | | |
| Reader.IncompleteMultilineComment2 | sax_io_mirror::Reader_IncompleteMultilineComment2 | sax_io | sax_io::Reader_IncompleteMultilineComment2_rust | | |
| Reader.InlineCommentsAreDisabledByDefault | sax_io_mirror::Reader_InlineCommentsAreDisabledByDefault | sax_io | sax_io::Reader_InlineCommentsAreDisabledByDefault_rust | | |
| Reader.IterativeParsing_Count | sax_io_mirror::Reader_IterativeParsing_Count | sax_io | sax_io::Reader_IterativeParsing_Count_rust | | |
| Reader.IterativeParsing_ErrorHandling | sax_io_mirror::Reader_IterativeParsing_ErrorHandling | sax_io | sax_io::Reader_IterativeParsing_ErrorHandling_rust | | |
| Reader.IterativeParsing_General | sax_io_mirror::Reader_IterativeParsing_General | sax_io | sax_io::Reader_IterativeParsing_General_rust | | |
| Reader.IterativeParsing_ShortCircuit | sax_io_mirror::Reader_IterativeParsing_ShortCircuit | sax_io | sax_io::Reader_IterativeParsing_ShortCircuit_rust | | |
| Reader.IterativePullParsing_General | sax_io_mirror::Reader_IterativePullParsing_General | sax_io | sax_io::Reader_IterativePullParsing_General_rust | | |
| Reader.MultipleTrailingCommaErrors | sax_io_mirror::Reader_MultipleTrailingCommaErrors | sax_io | sax_io::Reader_MultipleTrailingCommaErrors_rust | | |
| Reader.MultipleTrailingCommaErrorsIterative | sax_io_mirror::Reader_MultipleTrailingCommaErrorsIterative | sax_io | sax_io::Reader_MultipleTrailingCommaErrorsIterative_rust | | |
| Reader.NumbersAsStrings | sax_io_mirror::Reader_NumbersAsStrings | sax_io | sax_io::Reader_NumbersAsStrings_rust | | |
| Reader.NumbersAsStringsWChar_t | sax_io_mirror::Reader_NumbersAsStringsWChar_t | sax_io | sax_io::Reader_NumbersAsStringsWChar_t_rust | | |
| Reader.OnelineCommentsAreDisabledByDefault | sax_io_mirror::Reader_OnelineCommentsAreDisabledByDefault | sax_io | sax_io::Reader_OnelineCommentsAreDisabledByDefault_rust | | |
| Reader.Parse_EmptyObject | sax_io_mirror::Reader_Parse_EmptyObject | sax_io | sax_io::Reader_Parse_EmptyObject_rust | | |
| Reader.Parse_IStreamWrapper_StringStream | sax_io_mirror::Reader_Parse_IStreamWrapper_StringStream | sax_io | sax_io::Reader_Parse_IStreamWrapper_StringStream_rust | | |
| Reader.Parse_MultipleRoot | sax_io_mirror::Reader_Parse_MultipleRoot | sax_io | sax_io::Reader_Parse_MultipleRoot_rust | | |
| Reader.ParseArray | sax_io_mirror::Reader_ParseArray | sax_io | sax_io::Reader_ParseArray_rust | | |
| Reader.ParseArray_Error | sax_io_mirror::Reader_ParseArray_Error | sax_io | sax_io::Reader_ParseArray_Error_rust | | |
| Reader.ParseComments | sax_io_mirror::Reader_ParseComments | sax_io | sax_io::Reader_ParseComments_rust | | |
| Reader.ParseDocument_Error | sax_io_mirror::Reader_ParseDocument_Error | sax_io | sax_io::Reader_ParseDocument_Error_rust | | |
| Reader.ParseEmptyArray | sax_io_mirror::Reader_ParseEmptyArray | sax_io | sax_io::Reader_ParseEmptyArray_rust | | |
| Reader.ParseEmptyInlineComment | sax_io_mirror::Reader_ParseEmptyInlineComment | sax_io | sax_io::Reader_ParseEmptyInlineComment_rust | | |
| Reader.ParseEmptyOnelineComment | sax_io_mirror::Reader_ParseEmptyOnelineComment | sax_io | sax_io::Reader_ParseEmptyOnelineComment_rust | | |
| Reader.ParseFalse | sax_io_mirror::Reader_ParseFalse | sax_io | sax_io::Reader_ParseFalse_rust | | |
| Reader.ParseInsitu_MultipleRoot | sax_io_mirror::Reader_ParseInsitu_MultipleRoot | sax_io | sax_io::Reader_ParseInsitu_MultipleRoot_rust | | |
| Reader.ParseInsituIterative_MultipleRoot | sax_io_mirror::Reader_ParseInsituIterative_MultipleRoot | sax_io | sax_io::Reader_ParseInsituIterative_MultipleRoot_rust | | |
| Reader.ParseIterative_MultipleRoot | sax_io_mirror::Reader_ParseIterative_MultipleRoot | sax_io | sax_io::Reader_ParseIterative_MultipleRoot_rust | | |
| Reader.ParseMultipleCommentsInARow | sax_io_mirror::Reader_ParseMultipleCommentsInARow | sax_io | sax_io::Reader_ParseMultipleCommentsInARow_rust | | |
| Reader.ParseNanAndInfinity | sax_io_mirror::Reader_ParseNanAndInfinity | sax_io | sax_io::Reader_ParseNanAndInfinity_rust | | |
| Reader.ParseNumber_FullPrecisionDouble | sax_io_mirror::Reader_ParseNumber_FullPrecisionDouble | sax_io | sax_io::Reader_ParseNumber_FullPrecisionDouble_rust | | |
| Reader.ParseNumber_Integer | sax_io_mirror::Reader_ParseNumber_Integer | sax_io | sax_io::Reader_ParseNumber_Integer_rust | | |
| Reader.ParseNumber_NormalPrecisionDouble | sax_io_mirror::Reader_ParseNumber_NormalPrecisionDouble | sax_io | sax_io::Reader_ParseNumber_NormalPrecisionDouble_rust | | |
| Reader.ParseNumber_NormalPrecisionError | sax_io_mirror::Reader_ParseNumber_NormalPrecisionError | sax_io | sax_io::Reader_ParseNumber_NormalPrecisionError_rust | | |
| Reader.ParseNumberError_FullPrecisionDouble | sax_io_mirror::Reader_ParseNumberError_FullPrecisionDouble | sax_io | sax_io::Reader_ParseNumberError_FullPrecisionDouble_rust | | |
| Reader.ParseNumberError_NormalPrecisionDouble | sax_io_mirror::Reader_ParseNumberError_NormalPrecisionDouble | sax_io | sax_io::Reader_ParseNumberError_NormalPrecisionDouble_rust | | |
| Reader.ParseObject | sax_io_mirror::Reader_ParseObject | sax_io | sax_io::Reader_ParseObject_rust | | |
| Reader.ParseObject_Error | sax_io_mirror::Reader_ParseObject_Error | sax_io | sax_io::Reader_ParseObject_Error_rust | | |
| Reader.ParseString | sax_io_mirror::Reader_ParseString | sax_io | sax_io::Reader_ParseString_rust | | |
| Reader.ParseString_Error | sax_io_mirror::Reader_ParseString_Error | sax_io | sax_io::Reader_ParseString_Error_rust | | |
| Reader.ParseString_NonDestructive | sax_io_mirror::Reader_ParseString_NonDestructive | sax_io | sax_io::Reader_ParseString_NonDestructive_rust | | |
| Reader.ParseString_Transcoding | sax_io_mirror::Reader_ParseString_Transcoding | sax_io | sax_io::Reader_ParseString_Transcoding_rust | | |
| Reader.ParseString_TranscodingWithValidation | sax_io_mirror::Reader_ParseString_TranscodingWithValidation | sax_io | sax_io::Reader_ParseString_TranscodingWithValidation_rust | | |
| Reader.ParseTerminationByHandler | sax_io_mirror::Reader_ParseTerminationByHandler | sax_io | sax_io::Reader_ParseTerminationByHandler_rust | | |
| Reader.ParseTrue | sax_io_mirror::Reader_ParseTrue | sax_io | sax_io::Reader_ParseTrue_rust | | |
| Reader.ParseValue_Error | sax_io_mirror::Reader_ParseValue_Error | sax_io | sax_io::Reader_ParseValue_Error_rust | | |
| Reader.SkipWhitespace | sax_io_mirror::Reader_SkipWhitespace | sax_io | sax_io::Reader_SkipWhitespace_rust | | |
| Reader.TrailingCommaHandlerTermination | sax_io_mirror::Reader_TrailingCommaHandlerTermination | sax_io | sax_io::Reader_TrailingCommaHandlerTermination_rust | | |
| Reader.TrailingCommaHandlerTerminationIterative | sax_io_mirror::Reader_TrailingCommaHandlerTerminationIterative | sax_io | sax_io::Reader_TrailingCommaHandlerTerminationIterative_rust | | |
| Reader.TrailingCommas | sax_io_mirror::Reader_TrailingCommas | sax_io | sax_io::Reader_TrailingCommas_rust | | |
| Reader.TrailingCommasIterative | sax_io_mirror::Reader_TrailingCommasIterative | sax_io | sax_io::Reader_TrailingCommasIterative_rust | | |
| Reader.UnrecognizedComment | sax_io_mirror::Reader_UnrecognizedComment | sax_io | sax_io::Reader_UnrecognizedComment_rust | | |
| Write.RawValue_Issue1152 | sax_io_mirror::Write_RawValue_Issue1152 | sax_io | sax_io::Write_RawValue_Issue1152_rust | | |
| Writer.AssertIncorrectArrayLevel | sax_io_mirror::Writer_AssertIncorrectArrayLevel | sax_io | sax_io::Writer_AssertIncorrectArrayLevel_rust | | |
| Writer.AssertIncorrectEndArray | sax_io_mirror::Writer_AssertIncorrectEndArray | sax_io | sax_io::Writer_AssertIncorrectEndArray_rust | | |
| Writer.AssertIncorrectEndObject | sax_io_mirror::Writer_AssertIncorrectEndObject | sax_io | sax_io::Writer_AssertIncorrectEndObject_rust | | |
| Writer.AssertIncorrectObjectLevel | sax_io_mirror::Writer_AssertIncorrectObjectLevel | sax_io | sax_io::Writer_AssertIncorrectObjectLevel_rust | | |
| Writer.AssertMultipleRoot | sax_io_mirror::Writer_AssertMultipleRoot | sax_io | sax_io::Writer_AssertMultipleRoot_rust | | |
| Writer.AssertObjectKeyNotString | sax_io_mirror::Writer_AssertObjectKeyNotString | sax_io | sax_io::Writer_AssertObjectKeyNotString_rust | | |
| Writer.AssertRootMayBeAnyValue | sax_io_mirror::Writer_AssertRootMayBeAnyValue | sax_io | sax_io::Writer_AssertRootMayBeAnyValue_rust | | |
| Writer.Compact | sax_io_mirror::Writer_Compact | sax_io | sax_io::Writer_Compact_rust | | |
| Writer.Double | sax_io_mirror::Writer_Double | sax_io | sax_io::Writer_Double_rust | | |
| Writer.Inf | sax_io_mirror::Writer_Inf | sax_io | sax_io::Writer_Inf_rust | | |
| Writer.InfToNull | sax_io_mirror::Writer_InfToNull | sax_io | sax_io::Writer_InfToNull_rust | | |
| Writer.Int | sax_io_mirror::Writer_Int | sax_io | sax_io::Writer_Int_rust | | |
| Writer.Int64 | sax_io_mirror::Writer_Int64 | sax_io | sax_io::Writer_Int64_rust | | |
| Writer.InvalidEncoding | sax_io_mirror::Writer_InvalidEncoding | sax_io | sax_io::Writer_InvalidEncoding_rust | | |
| Writer.InvalidEventSequence | sax_io_mirror::Writer_InvalidEventSequence | sax_io | sax_io::Writer_InvalidEventSequence_rust | | |
| Writer.Issue_889 | sax_io_mirror::Writer_Issue_889 | sax_io | sax_io::Writer_Issue_889_rust | | |
| Writer.MoveCtor | sax_io_mirror::Writer_MoveCtor | sax_io | sax_io::Writer_MoveCtor_rust | | |
| Writer.NaN | sax_io_mirror::Writer_NaN | sax_io | sax_io::Writer_NaN_rust | | |
| Writer.NaNToNull | sax_io_mirror::Writer_NaNToNull | sax_io | sax_io::Writer_NaNToNull_rust | | |
| Writer.OStreamWrapper | sax_io_mirror::Writer_OStreamWrapper | sax_io | sax_io::Writer_OStreamWrapper_rust | | |
| Writer.RawValue | sax_io_mirror::Writer_RawValue | sax_io | sax_io::Writer_RawValue_rust | | |
| Writer.Root | sax_io_mirror::Writer_Root | sax_io | sax_io::Writer_Root_rust | | |
| Writer.RootArrayIsComplete | sax_io_mirror::Writer_RootArrayIsComplete | sax_io | sax_io::Writer_RootArrayIsComplete_rust | | |
| Writer.RootObjectIsComplete | sax_io_mirror::Writer_RootObjectIsComplete | sax_io | sax_io::Writer_RootObjectIsComplete_rust | | |
| Writer.RootValueIsComplete | sax_io_mirror::Writer_RootValueIsComplete | sax_io | sax_io::Writer_RootValueIsComplete_rust | | |
| Writer.ScanWriteUnescapedString | sax_io_mirror::Writer_ScanWriteUnescapedString | sax_io | sax_io::Writer_ScanWriteUnescapedString_rust | | |
| Writer.String | sax_io_mirror::Writer_String | sax_io | sax_io::Writer_String_rust | | |
| Writer.Transcode | sax_io_mirror::Writer_Transcode | sax_io | sax_io::Writer_Transcode_rust | | |
| Writer.UInt | sax_io_mirror::Writer_UInt | sax_io | sax_io::Writer_UInt_rust | | |
| Writer.Uint64 | sax_io_mirror::Writer_Uint64 | sax_io | sax_io::Writer_Uint64_rust | | |
| Writer.ValidateEncoding | sax_io_mirror::Writer_ValidateEncoding | sax_io | sax_io::Writer_ValidateEncoding_rust | | |#### 4.3 迁移策略建议

| 测试 ID | 进入 L2 优先级建议 | 原因 |
|---------|--------------------|------|
| `Reader.*` | high | 解析行为核心。 |
| `Writer.*` | high | 输出行为核心。 |
| `CursorStreamWrapper.*` | medium | 流包装重要但可稍后稳态化。 |
| `JsonChecker.Reader` | medium | 辅助验证 JSON 合法性。 |
| `RapidJson.Reader*` | medium | 性能验证，可在行为稳定后重点对比。 |

### 4.4 孪生测试层交付件

| 交付件 | 说明 |
|--------|------|
| `reports/sax-io.rust.junit.xml` | 孪生层执行结果。 |
| `reports/sax-io.parity.json` | 镜像 vs 孪生 一致性报告。 |
| `migrations/sax-io.mirror_to_rust.json` | 镜像测试 -> 孪生测试映射表。 |
| `rapidjson-rs/tests/rust/sax_io.rs` | SAX 孪生测试源文件。 |

---

## 变更历史

| 版本 | 日期 | 变更内容 | 变更人 |
|------|------|----------|--------|
| `0.1.0-draft` | `2026-05-14` | 初始化 FEAT-004 `sax-io` feature 级测试设计文档。 | `TBD` |
