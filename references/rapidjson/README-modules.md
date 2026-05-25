# RapidJSON 模块说明

> **原项目**：[Tencent/rapidjson](https://github.com/Tencent/rapidjson)（v1.1.0）  
> **许可证**：MIT License  
> **在本仓库的用途**：作为 cpp2rust-demo 的 C++ 目标库，用于验证 AST 解析与 Rust FFI 脚手架生成能力。

---

## 项目简介

RapidJSON 是一个高性能、仅头文件的 C++ JSON 解析与生成库，支持 SAX 和 DOM 两种 API 风格。其特点：

- **极速**：性能可与 `strlen()` 媲美，可选 SSE2/SSE4.2 加速
- **仅头文件**：无外部依赖，不依赖 STL
- **内存友好**：每个 JSON 值仅占 16 字节（x86-64）
- **Unicode 友好**：原生支持 UTF-8/UTF-16/UTF-32

---

## 主要头文件模块

### 核心模块

| 头文件 | 功能说明 |
|--------|----------|
| `rapidjson.h` | 基础宏定义、版本号、平台适配 |
| `fwd.h` | 前向声明（避免循环包含） |
| `document.h` | **DOM API 核心**：`Document`、`Value`、`GenericDocument` |
| `reader.h` | **SAX API 核心**：`Reader`、`GenericReader`，事件驱动解析 |
| `writer.h` | **JSON 输出**：`Writer`，将 DOM 序列化为 JSON 字符串 |
| `prettywriter.h` | **格式化输出**：`PrettyWriter`，美化缩进输出（继承自 `Writer`） |
| `stringbuffer.h` | 内存中的字符串缓冲区（`StringBuffer`），供 `Writer` 输出使用 |
| `stream.h` | 流接口抽象（`Stream` 概念） |

### 高级功能模块

| 头文件 | 功能说明 |
|--------|----------|
| `pointer.h` | **JSON Pointer**（RFC 6901）：`Pointer`，通过路径访问 JSON 节点 |
| `schema.h` | **JSON Schema**（draft-04）：`SchemaDocument`、`SchemaValidator` |
| `allocators.h` | 内存分配器：`MemoryPoolAllocator`、`CrtAllocator` |
| `encodings.h` | 字符编码定义：`UTF8`、`UTF16`、`UTF32` 及编码检测 |
| `encodedstream.h` | 自动编码检测的流包装器 |

### 流适配模块

| 头文件 | 功能说明 |
|--------|----------|
| `filereadstream.h` | 文件读取流（`FILE*` 包装） |
| `filewritestream.h` | 文件写入流（`FILE*` 包装） |
| `istreamwrapper.h` | `std::istream` 包装器 |
| `ostreamwrapper.h` | `std::ostream` 包装器 |
| `memorybuffer.h` | 内存读写缓冲区 |
| `memorystream.h` | 只读内存流 |
| `cursorstreamwrapper.h` | 带行列游标的流包装器（用于错误定位） |

### 内部实现（`internal/`）

| 头文件 | 功能说明 |
|--------|----------|
| `internal/dtoa.h` | 双精度浮点数快速转字符串 |
| `internal/itoa.h` | 整数快速转字符串 |
| `internal/strfunc.h` | 字符串工具函数 |
| `internal/stack.h` | 内部用栈（解析状态栈） |
| `internal/meta.h` | 模板元编程工具 |
| `internal/biginteger.h` | 大整数（精确浮点解析用） |
| `internal/clzll.h` | 前导零计数（CLZ） |
| `internal/pow10.h` | 10 的幂次预计算表 |
| `internal/regex.h` | 轻量正则表达式（JSON Schema 用） |
| `internal/ieee754.h` | IEEE 754 浮点数位级操作 |

---

## 常用 API 示例

### DOM 解析与访问

```cpp
#include "rapidjson/document.h"
using namespace rapidjson;

const char* json = "{\"name\":\"RapidJSON\",\"version\":1}";
Document doc;
doc.Parse(json);

// 访问字段
const char* name = doc["name"].GetString();
int version = doc["version"].GetInt();
```

### SAX 解析

```cpp
#include "rapidjson/reader.h"
#include "rapidjson/stringbuffer.h"
using namespace rapidjson;

struct MyHandler : BaseReaderHandler<UTF8<>, MyHandler> {
    bool String(const char* str, SizeType length, bool copy) {
        printf("String: %.*s\n", length, str);
        return true;
    }
};

MyHandler handler;
Reader reader;
StringStream ss(json);
reader.Parse(ss, handler);
```

### JSON 输出（Writer）

```cpp
#include "rapidjson/writer.h"
#include "rapidjson/stringbuffer.h"
using namespace rapidjson;

StringBuffer sb;
Writer<StringBuffer> writer(sb);
writer.StartObject();
writer.Key("hello");
writer.String("world");
writer.EndObject();

printf("%s\n", sb.GetString());  // {"hello":"world"}
```

### JSON Pointer

```cpp
#include "rapidjson/pointer.h"
using namespace rapidjson;

Document doc;
doc.Parse("{\"a\":{\"b\":42}}");

Value* v = Pointer("/a/b").Get(doc);
if (v) printf("%d\n", v->GetInt());  // 42
```

---

## cpp2rust-demo 中的使用

在 cpp2rust-demo 中，rapidjson 用于验证以下 AST 解析场景：

| 验证模块 | 对应 rapidjson 头文件 |
|----------|----------------------|
| DOM 解析（`Document`/`Value`） | `document.h` |
| SAX 读取器（`Reader`） | `reader.h` |
| JSON 输出（`Writer`） | `writer.h` |
| 格式化输出（`PrettyWriter`） | `prettywriter.h` |
| JSON Pointer | `pointer.h` |
| JSON Schema | `schema.h` |

验证脚本：`scripts/validate-rapidjson.sh`  
CI 工作流：`.github/workflows/validate-rapidjson.yml`

---

## 构建说明

RapidJSON 是纯头文件库，使用时仅需：

```cmake
target_include_directories(myapp PRIVATE path/to/rapidjson/include)
```

对于 cpp2rust-demo 验证：

```bash
bash scripts/validate-rapidjson.sh
```

---

## 参考文档

- [官方文档（英文）](http://rapidjson.org/)
- [官方文档（中文）](http://rapidjson.org/zh-cn/)
- [GitHub 仓库](https://github.com/Tencent/rapidjson/)
- [变更日志](CHANGELOG.md)
