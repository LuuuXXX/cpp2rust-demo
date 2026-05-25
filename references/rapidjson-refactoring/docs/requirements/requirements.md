# RapidJSON 需求文档

> 版本：v1.2.0 | 版权：Tencent / Milo Yip | 许可证：MIT

---

## 1. 项目概述

RapidJSON 是一个高效的 JSON 解析器及生成器，灵感来自 RapidXml。它提供 SAX（事件驱动）和 DOM（文档对像模型）两种风格的 API，专為高性能、低内存佔用和跨平台场景设计。

> 本文档记录 Rust 版本的功能需求，描述以 Rust 惯用语义為主。
> C++ 兼容性需求仅出现在第 2.10 节（FFI 层）。

## 2. 功能需求

### 2.1 JSON 解析（Parsing）

| 编号 | 需求描述 | 优先级 |
|------|---------|--------|
| REQ-P-01 | 支持标準 JSON（RFC7159/ECMA-404）解析 | 高 |
| REQ-P-02 | 支持递归下降解析（默认，速度快） | 高 |
| REQ-P-03 | 支持迭代式解析（避免栈溢出，内存可控） | 中 |
| REQ-P-04 | 支持 In-Situ 原位解析（零拷贝字符串） | 中 |
| REQ-P-05 | 支持从字符串解析 | 高 |
| REQ-P-06 | 支持从输入流（Stream）解析 | 高 |
| REQ-P-07 | 支持从文件流解析 | 中 |
| REQ-P-08 | 支持宽鬆语法：单行 `//` 和多行 `/* */` 注释 | 低 |
| REQ-P-09 | 支持宽鬆语法：对象和数组末尾逗号 | 低 |
| REQ-P-10 | 支持宽鬆语法：`NaN`/`Infinity` 作為 double 值 | 低 |
| REQ-P-11 | 支持数字以字符串形式解析 | 低 |
| REQ-P-12 | 支持在同一流中解析多个 JSON（StopWhenDone） | 中 |
| REQ-P-13 | 支持全精度数字解析（精确无 ULP 误差） | 中 |
| REQ-P-14 | 提供完整的解析错误码及偏移量 | 高 |
| REQ-P-15 | 支持错误信息本地化（英文内建，可自定义） | 低 |
| REQ-P-16 | 支持 SSE2/SSE4.2/ARM Neon SIMD 加速空白跳过 | 中 |
| REQ-P-17 | 支持转义单引号 `\'` | 低 |
| REQ-P-18 | 支持解析时跳过 BOM（UTF-8 BOM `EF BB BF`） | 低 |

### 2.2 JSON 生成（Generation）

| 编号 | 需求描述 | 优先级 |
|------|---------|--------|
| REQ-G-01 | 支持紧凑 JSON 输出（Writer，无空白） | 高 |
| REQ-G-02 | 支持格式化 JSON 输出（PrettyWriter，缩进换行） | 高 |
| REQ-G-03 | 支持写入字符串缓衝区 | 高 |
| REQ-G-04 | 支持写入文件流 | 中 |
| REQ-G-05 | 支持写入自定义输出流 | 中 |
| REQ-G-06 | 支持输出时编码验证 | 低 |
| REQ-G-07 | 支持输出 `NaN`/`Infinity` | 低 |

### 2.3 DOM 操作

| 编号 | 需求描述 | 优先级 |
|------|---------|--------|
| REQ-D-01 | 支持 7 种 JSON 值类型：Null、Bool、Number、String、Array、Object | 高 |
| REQ-D-02 | 支持类型查询：`is_null()`、`is_bool()`、`is_number()`、`is_string()` 等 | 高 |
| REQ-D-03 | 支持数值精确类型：`i32`、`u32`、`i64`、`u64`、`f64` | 高 |
| REQ-D-04 | 支持按键查询对象成员 | 高 |
| REQ-D-05 | 支持按下标访问数组元素 | 高 |
| REQ-D-06 | 支持迭代器遍歷数组和对象 | 高 |
| REQ-D-07 | 支持范围 for 循环遍歷数组和对象 | 中 |
| REQ-D-08 | 支持值创建：`set_int()`、`set_string()`、`set_object()`、`set_array()` 等 | 高 |
| REQ-D-09 | 支持从基本类型隐式构造 Value | 中 |
| REQ-D-10 | 支持对象添加成员 | 高 |
| REQ-D-11 | 支持对象删除成员 | 高 |
| REQ-D-12 | 支持数组添加/移除元素 | 高 |
| REQ-D-13 | 支持数组范围删除元素 | 中 |
| REQ-D-14 | 支持深拷贝 | 中 |
| REQ-D-15 | 支持值交换 | 中 |
| REQ-D-16 | 支持包含空字符 `\0` 的字符串 | 中 |
| REQ-D-17 | 支持零拷贝字符串引用（不复製原始字符串数据） | 中 |
| REQ-D-18 | 支持值比较（相等性判断） | 中 |

### 2.4 SAX 事件处理

| 编号 | 需求描述 | 优先级 |
|------|---------|--------|
| REQ-S-01 | 支持 SAX 事件驱动解析（Reader → Handler） | 高 |
| REQ-S-02 | 支持 SAX 事件驱动生成（Handler → Writer） | 高 |
| REQ-S-03 | Handler 接口：Null、Bool、Int、Uint、Int64、Uint64、Double | 高 |
| REQ-S-04 | Handler 接口：String、Key、StartObject、EndObject、StartArray、EndArray | 高 |
| REQ-S-05 | 支持 Handler 默认实现（空操作基线） | 中 |
| REQ-S-06 | 支持逐 Token 解析（Iterative Parse） | 低 |
| REQ-S-07 | 支持事件过滤中间层（如 capitalize、filterkey） | 中 |

### 2.5 JSON Pointer（RFC 6901）

| 编号 | 需求描述 | 优先级 |
|------|---------|--------|
| REQ-JP-01 | 支持通过 Pointer 路径获取值 | 中 |
| REQ-JP-02 | 支持通过 Pointer 路径设置值 | 中 |
| REQ-JP-03 | 支持通过 Pointer 创建值（自动创建父节点） | 中 |
| REQ-JP-04 | 支持带默认值获取 | 中 |
| REQ-JP-05 | 支持通过 Pointer 交换值 | 低 |
| REQ-JP-06 | 支持通过 Pointer 删除值 | 中 |
| REQ-JP-07 | 支持 URI Fragment 表示法（`#` 前缀 + 百分号编码） | 低 |
| REQ-JP-08 | 支持 Pointer 字符串化和 URI Fragment 序列化 | 低 |
| REQ-JP-09 | 支持用户自定义 Token 数组（无动态分配） | 低 |
| REQ-JP-10 | 支持 Pointer 解析错误报告（错误码 + 偏移量） | 中 |

### 2.6 JSON Schema（Draft v4）

| 编号 | 需求描述 | 优先级 |
|------|---------|--------|
| REQ-JS-01 | 支持编译 JSON Schema 為 SchemaDocument | 中 |
| REQ-JS-02 | 支持 DOM 校验（Accept 模式） | 中 |
| REQ-JS-03 | 支持 SAX 解析时同步校验 | 中 |
| REQ-JS-04 | 支持序列化时校验 | 低 |
| REQ-JS-05 | 支持远程 Schema 引用（用户自定义 Provider） | 低 |
| REQ-JS-06 | 支持校验错误报告（违规关键字、实例路径、Schema 路径） | 中 |
| REQ-JS-07 | 内建正则引擎（支持 `pattern`/`patternProperties`） | 低 |
| REQ-JS-08 | 支持 URI 解析（scheme、authority、path、query、fragment） | 低 |
| REQ-JS-09 | 支持 Swagger v2 及 OpenAPI v3.0.x Schema | 低 |

### 2.7 编码与 Unicode

| 编号 | 需求描述 | 优先级 |
|------|---------|--------|
| REQ-E-01 | 支持 UTF-8、UTF-16、UTF-32（含大端序/小端序） | 高 |
| REQ-E-02 | 支持自动检测输入流编码（BOM 及特徵检测） | 中 |
| REQ-E-03 | 支持编码间内部转码（如 UTF-8 输入 → UTF-16 DOM） | 中 |
| REQ-E-04 | 支持编码验证 | 中 |
| REQ-E-05 | 支持 Unicode 代理对（Surrogate Pair） | 中 |
| REQ-E-06 | 支持自定义编码和字符类型 | 低 |
| REQ-E-07 | 支持 ASCII 编码 | 低 |

### 2.8 流（Stream）

| 编号 | 需求描述 | 优先级 |
|------|---------|--------|
| REQ-ST-01 | 支持内存字符串输入流 | 高 |
| REQ-ST-02 | 支持内存字符串输出缓衝 | 高 |
| REQ-ST-03 | 支持文件读取流 | 高 |
| REQ-ST-04 | 支持文件写入流 | 高 |
| REQ-ST-05 | 支持编码输入流 | 中 |
| REQ-ST-06 | 支持编码输出流 | 中 |
| REQ-ST-07 | 支持自动编码检测流 | 中 |
| REQ-ST-08 | 支持内存字节缓衝区（非字符串终止） | 中 |
| REQ-ST-09 | 支持带游标的流包装器 | 中 |
| REQ-ST-10 | 支持自定义流（仅需实现 peek/take/tell 或 put/flush） | 中 |

### 2.9 内存管理

| 编号 | 需求描述 | 优先级 |
|------|---------|--------|
| REQ-M-01 | DOM 每个 Value 保持低内存佔用 | 高 |
| REQ-M-02 | 默认使用内存池分配器（顺序分配，不单独释放） | 高 |
| REQ-M-03 | 支持系统堆分配器（malloc/free） | 中 |
| REQ-M-04 | 支持用户预分配缓衝区（栈/静态数组，零堆分配） | 中 |
| REQ-M-05 | 支持自定义分配器 | 低 |

### 2.10 FFI 层（C/C++ 互操作）

> 本节需求仅适用於 FFI crate（`rapidjson-ffi`），核心库不依赖此节。

| 编号 | 需求描述 | 优先级 |
|------|---------|--------|
| REQ-FFI-01 | 提供 `extern "C"` 函数接口，覆盖核心 Value/Document/Reader/Writer 操作 | 中 |
| FFI 接口命名与原始 RapidJSON C++ API 保持一致 | 方便现有 C++ 项目迁移 | 中 |
| REQ-FFI-03 | FFI 层处理 Rust ↔ C 所有权转移（裸指针封装） | 高 |
| REQ-FFI-04 | FFI 层处理 C 字符串 `*const c_char` ↔ Rust `&str` 转换 | 高 |
| REQ-FFI-05 | FFI 层错误处理使用 C 风格返回码，不跨边界传播 panic | 高 |
| REQ-FFI-06 | FFI 层可选提供 C++ 头文件封装（`rapidjson-ffi-cpp`） | 低 |
| REQ-FFI-07 | FFI 层生命週期安全由 FFI 层自行管理，核心库不感知 FFI | 高 |

## 3. 非功能需求

| 编号 | 需求描述 | 指标 |
|------|---------|------|
| NFR-01 | 高性能：解析速度可与 `strlen()` 相比 | 基準测试通过 |
| NFR-02 | 低内存：DOM 每值保持最小佔用 | 内存分析通过 |
| NFR-03 | 跨平台：Windows、Linux、macOS、iOS、Android | CI 全平台通过 |
| NFR-04 | 跨工具链：MSRV（最低支持的 Rust 版本）向后兼容 | CI 验证 |
| NFR-05 | 无外部依赖：核心功能不依赖第三方 crate | 仅可选依赖用於 SIMD/测试 |
| NFR-06 | 标準合规：完全符合 RFC7159/ECMA-404 | JSON 测试套件通过 |
| NFR-07 | 线程安全：不同线程可使用各自独立实例，只读结构可跨线程共享 | 併发测试通过 |

## 4. 用例概览

### UC-01：解析 JSON 字符串到 DOM
**参与者**：开发者  
**流程**：JSON 字符串 → Document 解析 → 查询/修改 DOM  
**前置条件**：json 為合法 JSON 字符串  
**后置条件**：DOM 包含解析结果，可通过 `has_parse_error()` 检查错误

### UC-02：从文件解析 JSON
**参与者**：开发者  
**流程**：文件 → 文件读取流 → Document 解析 → DOM  
**前置条件**：文件可读  
**后置条件**：DOM 包含解析结果

### UC-03：生成 JSON 字符串
**参与者**：开发者  
**流程**：DOM → Writer 写入字符串缓衝 → 获取 JSON 字符串  
**前置条件**：DOM 已构建  
**后置条件**：输出合法 JSON 字符串

### UC-04：SAX 事件驱动处理
**参与者**：开发者  
**流程**：Reader + 自定义 Handler → 解析流并回调 Handler  
**前置条件**：Handler 实现 SAX 接口  
**后置条件**：Handler 接收到完整 SAX 事件序列

### UC-05：JSON Schema 校验
**参与者**：开发者  
**流程**：Schema JSON → 编译為 SchemaDocument → Validator 校验 DOM → 获取结果  
**前置条件**：Schema 符合 JSON Schema Draft v4  
**后置条件**：获知 JSON 是否符合 Schema

### UC-06：使用 JSON Pointer 访问/修改 DOM
**参与者**：开发者  
**流程**：构造 Pointer 路径 → Get/Set/Erase 操作 DOM  
**前置条件**：DOM 已构建  
**后置条件**：返回目标值或完成修改
