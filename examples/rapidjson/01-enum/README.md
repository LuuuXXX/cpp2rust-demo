# 场景 01：枚举类型绑定（`enum` / `enum class`）

本示例演示如何从 C++ `enum` 和 `enum class` 声明生成 Rust `#[repr(C)] enum`，
对应 cpp2rust-demo 内部的 `EnumIR` → `types/mod.rs` 流水线。

---

## 背景

RapidJSON 在 `rapidjson/error/error.h` 中定义了 `ParseErrorCode` 枚举；
在 `rapidjson/rapidjson.h` 中定义了 `Type` 枚举。
这些枚举是 RapidJSON JSON DOM 操作的核心返回值类型。

本示例用自包含的等价类型演示同一提取流程，**无需安装 RapidJSON**。

---

## C++ 源码（`entry.cpp`）

```cpp
// Equivalent of rapidjson/error/error.h
enum ParseErrorCode {
    kParseErrorNone = 0,
    kParseErrorDocumentEmpty,
    kParseErrorDocumentRootNotSingular,
    kParseErrorValueInvalid,
    // ... (完整定义见 entry.cpp)
};

// C++11 scoped enum（enum class）示例
enum class WriteErrorCode {
    kWriteErrorNone = 0,
    kWriteErrorInitFailed = 1,
    kWriteErrorBufferFull = 2,
};

// 值类型枚举
enum Type {
    kNullType = 0,
    kFalseType = 1,
    kTrueType = 2,
    kObjectType = 3,
    kArrayType = 4,
    kStringType = 5,
    kNumberType = 6,
};
```

---

## 运行步骤

在仓库根目录执行：

```bash
# 第 1 步：生成分组 FFI（--no-link：header-only，不链接外部库）
cpp2rust-demo init --feature rj01 --link rapidjson --no-link \
    -- clang -x c++ -fsyntax-only examples/rapidjson/01-enum/entry.cpp

# 第 2 步：合并
cpp2rust-demo merge --feature rj01

# 第 3 步：查看产物
cat .cpp2rust/rj01/rust/src/lib.rs
```

---

## 预期生成产物

### `types/mod.rs`（枚举类型定义）

```rust
// ParseErrorCode（非 scoped enum）
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParseErrorCode {
    kParseErrorNone = 0,
    kParseErrorDocumentEmpty = 1,
    kParseErrorDocumentRootNotSingular = 2,
    kParseErrorValueInvalid = 3,
    kParseErrorObjectMissName = 4,
    kParseErrorObjectMissColon = 5,
    kParseErrorObjectMissCommaOrCurlyBracket = 6,
    kParseErrorArrayMissCommaOrSquareBracket = 7,
    kParseErrorStringUnicodeEscapeInvalidHex = 8,
    kParseErrorStringUnicodeSurrogateInvalid = 9,
    kParseErrorStringEscapeInvalid = 10,
    kParseErrorStringMissQuotationMark = 11,
    kParseErrorStringInvalidEncoding = 12,
    kParseErrorNumberTooBig = 13,
    kParseErrorNumberMissFraction = 14,
    kParseErrorNumberMissExponent = 15,
    kParseErrorTermination = 16,
    kParseErrorUnspecificSyntaxError = 17,
}

// WriteErrorCode（enum class / scoped enum）
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WriteErrorCode {
    kWriteErrorNone = 0,
    kWriteErrorInitFailed = 1,
    kWriteErrorBufferFull = 2,
}

// Type
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Type {
    kNullType = 0,
    kFalseType = 1,
    kTrueType = 2,
    kObjectType = 3,
    kArrayType = 4,
    kStringType = 5,
    kNumberType = 6,
}
```

### `free/fn_entry.rs`（枚举参数函数）

```rust
hicc::import_lib! {
    #![link_name = "rapidjson"]

    #[cpp(func = "const char * parseErrorName(ParseErrorCode)")]
    fn parse_error_name(code: ParseErrorCode) -> *const i8;
}
```

---

## 场景解析

### 1. EnumDecl → EnumIR 提取流水线

cpp2rust-demo 在 AST 提取阶段识别 `EnumDecl` 节点，收集：

| AST 字段 | EnumIR 字段 | 说明 |
|---------|------------|------|
| `name` | `name` | 枚举类型名 |
| `fixedUnderlyingType` | `underlying_type` | 底层整型（如 `int`） |
| `scopedEnumTag` | `is_class` | `true` = `enum class`，`false` = 普通 `enum` |
| 子 `EnumConstantDecl` | `variants[]` | 枚举值列表 |
| `EnumConstantDecl.value` | `variants[].value` | 显式值（可为整数字面量或表达式结果） |

### 2. `#[repr(C)]` 的必要性

hicc 需要 Rust 枚举与 C++ 枚举有相同的内存布局（均为底层整型）。
`#[repr(C)]` 保证 Rust 编译器使用与 C ABI 兼容的表示形式，与 `int`（C++ 默认枚举底层类型）对齐。

### 3. `enum` vs `enum class` 的区别

| 特性 | `enum` (C++03) | `enum class` (C++11) |
|------|---------------|---------------------|
| Clang AST | `EnumDecl`（无 `scopedEnumTag`）| `EnumDecl`（有 `scopedEnumTag: "class"`）|
| `EnumIR.is_class` | `false` | `true` |
| Rust 生成 | `pub enum Foo { ... }` | `pub enum Foo { ... }` + `#[doc = "scoped"]` |
| 值访问 | `kFoo`（全局）| `Foo::kFoo`（C++ 侧）/ Rust 侧均用 `Foo::kFoo` |

注意：两种形式的 Rust 生成代码在结构上完全相同（均为 `#[repr(C)] pub enum`），
区别仅在于 C++ 侧的作用域规则，Rust 枚举本身已经是 scoped 的。

### 4. 枚举值的隐式编号

当 C++ 枚举没有显式值时（如 `kParseErrorDocumentEmpty` 没有 `= 1`），
clang AST JSON 中的 `value` 字段为计算后的整数值。
`AstNode::value` 字段使用自定义反序列化器，兼容字符串和数字两种格式。

### 5. 枚举作为函数参数类型

当函数参数或返回值类型为已提取的枚举时，类型门（`is_supported_cpp_type`）通过
`is_known_class_type` 检查将其放行，最终在 Rust 参数列表中直接使用枚举类型名。

---

## 限制说明

| 限制 | 说明 |
|------|------|
| 匿名枚举 | 跳过（无 `name` 的 EnumDecl） |
| 模板枚举 | `template<typename T> enum Foo { ... }` 不支持 |
| `__attribute__((packed))` 枚举 | 底层类型可能不同，需手工验证 ABI |
| 枚举成员函数 | C++ 不允许枚举成员函数，此场景不存在 |
