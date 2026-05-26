# Clang AST 解析能力详解

## 1. 概述

本文档详细展示 Clang AST（Abstract Syntax Tree）对 C++ 特性的解析能力，以及如何利用这些信息生成 Rust FFI 代码。

## 2. C++ 特性与 AST 节点映射

### 2.1 特性覆盖总表

根据 `./examples/` 中 48 个示例的 C++ 特性覆盖：

| 示例 | C++ 特性 | Clang AST 节点 | v1 支持 | v2 支持 |
|------|-----------|----------------|---------|---------|
| 001 | extern "C" 函数 | `FunctionDecl` | ✅ | ✅ |
| 002 | 函数重载 | `FunctionDecl` | ✅ | ✅ |
| 003 | 默认参数 | `FunctionDecl` | ✅ | ✅ |
| 004 | 内联函数 | `FunctionDecl` + `inline` | ✅ | ✅ |
| 005 | 可变参数函数 | `FunctionDecl` | ✅ | ✅ |
| 006 | 基础类 | `CXXRecordDecl` | ✅ | ✅ |
| 007 | 构造/析构函数 | `CXXConstructorDecl`, `CXXDestructorDecl` | ✅ | ✅ |
| 008 | 拷贝构造函数 | `CXXConstructorDecl` | ✅ | ✅ |
| 009 | 移动构造函数 | `CXXConstructorDecl` | ✅ | ✅ |
| 010 | 静态成员 | `VarDecl` (static) | ✅ | ✅ |
| 011 | const 成员函数 | `CXXMethodDecl` + `const` | ✅ | ✅ |
| 012 | volatile 成员函数 | `CXXMethodDecl` + `volatile` | ✅ | ✅ |
| 013 | 单继承 | `CXXBaseSpecifier` | ✅ | ✅ |
| 014 | 多继承 | `CXXBaseSpecifier` | ✅ | ✅ |
| 015 | 虚函数基础 | `CXXMethodDecl` + `virtual` | ✅ | ✅ |
| 016 | 纯虚函数/抽象类 | `CXXMethodDecl` + `= 0` | ⚠️ | ✅ |
| 017 | override 说明符 | `CXXMethodDecl` + `override` | ✅ | ✅ |
| 018 | 菱形继承 | `CXXBaseSpecifier` + virtual | ⚠️ | ✅ |
| 019 | 运算符重载 | `CXXMethodDecl` (operator) | ❌ | ❌ |
| 020 | 友元函数 | `FriendDecl` | ❌ | ❌ |
| 021 | explicit 构造函数 | `CXXConstructorDecl` + `explicit` | ✅ | ✅ |
| 022 | mutable 成员 | `FieldDecl` + `mutable` | ✅ | ✅ |
| 023 | typeid/RTTI | `CXXTypeidExpr` | ❌ | ❌ |
| 024 | 函数模板 | `FunctionDecl` + template | ✅ | ✅ |
| 025 | 类模板 | `ClassTemplateDecl` | ⚠️ | ✅ |
| 026 | 模板偏特化 | `ClassTemplateSpecializationDecl` | ❌ | ⚠️ |
| 027 | 模板显式实例化 | `ClassTemplateSpecialization` | ⚠️ | ✅ |
| 028 | 可变参数模板 | `VariadicTemplate` | ❌ | ⚠️ |
| 029 | unique_ptr | `CXXNewExpr` | ✅ | ✅ |
| 030 | shared_ptr | `CXXNewExpr` | ✅ | ✅ |
| 031 | 自定义删除器 | `FunctionDecl` | ✅ | ✅ |
| 032 | placement new | `CXXNewExpr` | ✅ | ✅ |
| 033 | RAII 模式 | 构造函数/析构函数 | ✅ | ✅ |
| 034 | std::vector | `ClassTemplateSpecialization` | ⚠️ | ✅ |
| 035 | std::map | `ClassTemplateSpecialization` | ⚠️ | ✅ |
| 036 | std::string | `ClassTemplateSpecialization` | ⚠️ | ✅ |
| 037 | std::array | `ClassTemplateSpecialization` | ⚠️ | ✅ |
| 038 | std::tuple | `ClassTemplateSpecialization` | ⚠️ | ✅ |
| 039 | Lambda 表达式 | `LambdaExpr` | ❌ | ⚠️ |
| 040 | std::function | `ClassTemplateSpecialization` | ⚠️ | ✅ |
| 041 | std::bind | `CallExpr` | ❌ | ⚠️ |
| 042 | 异常处理 | `CXXThrowExpr`, `CXXCatchStmt` | ✅ | ✅ |
| 043 | 嵌套命名空间 | `NamespaceDecl` | ✅ | ✅ |
| 044 | enum class | `EnumDecl` | ✅ | ✅ |
| 045 | union | `RecordDecl` (union) | ✅ | ✅ |
| 046 | constexpr | `Expr` + `constexpr` | ✅ | ✅ |
| 047 | noexcept | `NoexceptSpec` | ✅ | ✅ |

**图例**：✅ 完全支持 ⚠️ 部分支持 ❌ 不支持

## 3. Clang AST 节点详解

### 3.1 声明类节点

#### 3.1.1 翻译单元 `TranslationUnitDecl`

```json
{
  "id": "0x1",
  "kind": "TranslationUnitDecl",
  "loc": {},
  "range": {},
  "inner": [
    // 所有顶层声明都在这里
  ]
}
```

#### 3.1.2 命名空间 `NamespaceDecl`

```json
{
  "id": "0x123",
  "kind": "NamespaceDecl",
  "name": "foo",
  "loc": {
    "offset": 10,
    "file": "example.h",
    "line": 1,
    "col": 11,
    "tokLen": 3
  },
  "range": {
    "begin": {"offset": 0, "col": 1, "tokLen": 9},
    "end": {"offset": 780, "line": 30, "col": 1, "tokLen": 1}
  },
  "inner": [
    // 嵌套的声明
  ]
}
```

**解析要点**：
- `name`: 命名空间名称
- `inner`: 嵌套的所有声明
- 可以递归嵌套（foo::bar::config）

#### 3.1.3 类/结构体 `CXXRecordDecl`

```json
{
  "id": "0x126",
  "kind": "CXXRecordDecl",
  "name": "ConfigManager",
  "tagUsed": "class",
  "completeDefinition": true,
  "definitionData": {
    "canConstDefaultInit": true,
    "defaultCtor": {"exists": true, "userProvided": true},
    "dtor": {"nonTrivial": true, "userDeclared": true},
    "hasUserDeclaredConstructor": true,
    "isStandardLayout": true,
    "isEmpty": true
  },
  "inner": [
    // AccessSpecDecl, FieldDecl, CXXMethodDecl 等
  ]
}
```

**解析要点**：
- `tagUsed`: "class" | "struct" | "union"
- `completeDefinition`: 是否完整定义
- `definitionData`: 类的各种属性
- `inner`: 包含访问说明符、字段、方法等

#### 3.1.4 构造函数 `CXXConstructorDecl`

```json
{
  "id": "0x140",
  "kind": "CXXConstructorDecl",
  "name": "ConfigManager",
  "explicit": false,
  "const": false,
  "inner": [
    // 参数列表
  ]
}
```

**解析要点**：
- `explicit`: 是否为 explicit 构造函数
- `const`: 是否为 const 构造函数

#### 3.1.5 析构函数 `CXXDestructorDecl`

```json
{
  "id": "0x141",
  "kind": "CXXDestructorDecl",
  "name": "~ConfigManager"
}
```

#### 3.1.6 类方法 `CXXMethodDecl`

```json
{
  "id": "0x142",
  "kind": "CXXMethodDecl",
  "name": "set_value",
  "type": {
    "qualType": "void (const char *, int)"
  },
  "virtual": false,
  "const": false,
  "inline": true,
  "inner": [
    {
      "kind": "ParmVarDecl",
      "name": "key",
      "type": {"qualType": "const char *"}
    },
    {
      "kind": "ParmVarDecl",
      "name": "value",
      "type": {"qualType": "int"}
    }
  ]
}
```

**解析要点**：
- `virtual`: 是否为虚函数
- `const`: 是否为 const 方法
- `override`: 是否为 override
- `type.qualType`: 完整的函数签名

#### 3.1.7 虚函数与纯虚函数

```json
{
  "kind": "CXXMethodDecl",
  "name": "area",
  "type": {"qualType": "double () const"},
  "virtual": true,
  "pure_virtual": true
}
```

**pure_virtual**: `true` 表示纯虚函数 (`= 0`)

#### 3.1.8 继承 `CXXBaseSpecifier`

```json
{
  "kind": "CXXBaseSpecifier",
  "access": "public",
  "virtual": false,
  "baseType": {
    "qualType": "Animal"
  }
}
```

**解析要点**：
- `access`: "public" | "protected" | "private"
- `virtual`: 是否为虚拟继承
- `baseType`: 基类类型

#### 3.1.9 枚举 `EnumDecl`

```json
{
  "kind": "EnumDecl",
  "name": "ErrorCode",
  "scoped": true,
  "enumType": {
    "qualType": "int"
  },
  "inner": [
    {"kind": "EnumConstantDecl", "name": "None", "value": {"value": 0}},
    {"kind": "EnumConstantDecl", "name": "InvalidInput", "value": {"value": 1}}
  ]
}
```

**解析要点**：
- `scoped`: true 表示 enum class，false 表示普通 enum
- `enumType`: 底层类型

#### 3.1.10 函数声明 `FunctionDecl`

```json
{
  "kind": "FunctionDecl",
  "name": "hello_world",
  "type": {"qualType": "void (void)"},
  "inline": true
}
```

### 3.2 类型类节点

#### 3.2.1 类型引用 `TypeRef`

当代码中引用一个类型时（如 `std::vector<int>`）：

```json
{
  "kind": "TypeRef",
  "type": {
    "qualType": "class std::vector<int>"
  },
  "referenced": {
    "kind": "ClassTemplateSpecialization"
  }
}
```

#### 3.2.2 模板实例化 `ClassTemplateSpecialization`

```json
{
  "kind": "ClassTemplateSpecialization",
  "name": "vector",
  "specializationKind": "ExplicitInstantiation",
  "templateArgs": [
    {"kind": "TemplateArgument", "type": {"qualType": "int"}}
  ],
  "inner": []
}
```

**解析要点**：
- `name`: 模板名称
- `specializationKind`: "ExplicitInstantiation" | "ExplicitSpecialization" | "Implicit"
- `templateArgs`: 模板参数列表

### 3.3 表达式类节点

#### 3.3.1 Lambda 表达式 `LambdaExpr`

```json
{
  "kind": "LambdaExpr",
  "captureDefault": "this",
  "hasExplicitParams": true,
  "hasExplicitResultType": false,
  "inner": [
    {
      "kind": "ParmVarDecl",
      "name": "x",
      "type": {"qualType": "int"}
    }
  ]
}
```

#### 3.3.2 new 表达式 `CXXNewExpr`

```json
{
  "kind": "CXXNewExpr",
  "type": {"qualType": "class std::unique_ptr<Config>"},
  "isArray": false,
  "Initializer": {
    "kind": "ParenListExpr"
  }
}
```

### 3.3 模板类节点

#### 3.3.1 模板声明 `ClassTemplateDecl`

```json
{
  "kind": "ClassTemplateDecl",
  "name": "Stack",
  "templateParameters": [
    {
      "kind": "TemplateTypeParmDecl",
      "name": "T",
      "depth": 0,
      "index": 0
    }
  ],
  "underlyingDecl": {
    "kind": "CXXRecordDecl",
    "name": "Stack"
  }
}
```

#### 3.3.2 模板实例化 `ClassTemplateSpecialization`

```json
{
  "kind": "ClassTemplateSpecialization",
  "name": "vector",
  "specializationKind": "Implicit",
  "templateArgs": [
    {"kind": "TemplateArgument", "kind": "Type", "type": {"qualType": "int"}}
  ],
  "type": {"qualType": "class std::vector<int, class std::allocator<int> >"},
  "inner": []
}
```

## 4. Examples 中的实际 AST 结构

### 4.1 嵌套命名空间 (043_namespace_nested)

**C++ 源码**：
```cpp
namespace foo {
    namespace bar {
        namespace config {
            class ConfigManager { /* ... */ };
        }
    }
}
```

**AST 结构**：
```
TranslationUnitDecl
└── NamespaceDecl: foo
    └── NamespaceDecl: bar
        └── NamespaceDecl: config
            └── CXXRecordDecl: ConfigManager
                ├── AccessSpecDecl: public
                ├── VarDecl: MAX_ENTRIES (static)
                ├── FieldDecl: values_
                ├── FieldDecl: keys_
                ├── FieldDecl: count_
                ├── CXXConstructorDecl: ConfigManager
                ├── CXXDestructorDecl: ~ConfigManager
                ├── CXXMethodDecl: set_value
                └── CXXMethodDecl: get_value (const)
```

### 4.2 单继承 (013_inheritance_single)

**C++ 源码**：
```cpp
class Dog : public Animal { /* ... */ };
```

**AST 结构**：
```
CXXRecordDecl: Dog
├── CXXBaseSpecifier: public Animal (virtual=false)
├── CXXConstructorDecl: Dog
├── CXXDestructorDecl: ~Dog
├── CXXMethodDecl: bark
└── CXXMethodDecl: speak (override)
```

### 4.3 纯虚函数/抽象类 (016_virtual_pure)

**C++ 源码**：
```cpp
class AbstractShape {
    virtual ~AbstractShape() = default;
    virtual double area() const = 0;
    virtual const char* getName() const = 0;
};
```

**AST 结构**：
```
CXXRecordDecl: AbstractShape
├── CXXDestructorDecl: ~AbstractShape (virtual)
└── CXXMethodDecl: area
    ├── virtual: true
    ├── pure_virtual: true
    └── const: true
```

### 4.4 模板实例化 (034_vector_basic)

**C++ 源码**：
```cpp
std::vector<int> int_vec;
```

**AST 结构**：
```
VarDecl: int_vec
└── TypeRef: class std::vector<int>
    └── ClassTemplateSpecialization: vector
        ├── templateArgs: [int]
        └── type: std::vector<int, std::allocator<int>>
```

## 5. 类型映射详解

### 5.1 C++ 类型到 Rust FFI 类型

| C++ 类型 | Clang qualType | Rust FFI 类型 |
|----------|----------------|---------------|
| `int` | `"int"` | `i32` |
| `unsigned int` | `"unsigned int"` | `u32` |
| `char` | `"char"` | `i8` |
| `unsigned char` | `"unsigned char"` | `u8` |
| `short` | `"short"` | `i16` |
| `unsigned short` | `"unsigned short"` | `u16` |
| `long` | `"long"` | `i64` |
| `unsigned long` | `"unsigned long"` | `u64` |
| `size_t` | `"size_t"` | `usize` |
| `ssize_t` | `"ssize_t"` | `isize` |
| `void` | `"void"` | `()` |
| `bool` | `"bool"` | `bool` |
| `float` | `"float"` | `f32` |
| `double` | `"double"` | `f64` |
| `int*` | `"int *"` | `*mut i32` |
| `const int*` | `"const int *"` | `*const i32` |
| `int&` | `"int &"` | `&mut i32` |
| `const int&` | `"const int &"` | `&i32` |
| `int[10]` | `"int [10]"` | `[i32; 10]` |
| `void*` | `"void *"` | `*mut std::ffi::c_void` |
| `char*` | `"char *"` | `*mut i8` |
| `const char*` | `"const char *"` | `*const i8` |

### 5.2 函数类型

| C++ 函数签名 | Clang qualType | Rust FFI 类型 |
|--------------|----------------|----------------|
| `void func()` | `"void (void)"` | `fn()` |
| `int func(int)` | `"int (int)"` | `fn(i32) -> i32` |
| `void func(int*)` | `"void (int *)"` | `fn(*mut i32)` |
| `int func(const char*)` | `"int (const char *)"` | `fn(*const i8) -> i32` |

### 5.3 命名空间深度判断

```rust
fn get_namespace_depth(full_name: &str) -> usize {
    full_name.matches("::").count()
}

// 示例
assert_eq!(get_namespace_depth("ConfigManager"), 0);
assert_eq!(get_namespace_depth("foo::ConfigManager"), 1);
assert_eq!(get_namespace_depth("foo::bar::ConfigManager"), 2);
assert_eq!(get_namespace_depth("foo::bar::config::ConfigManager"), 3);

// 决策：depth >= 2 时使用 void* 模式
fn should_use_opaque_pointer(full_name: &str) -> bool {
    get_namespace_depth(full_name) >= 2
}
```

## 6. 复杂特性解析

### 6.1 运算符重载

```json
{
  "kind": "CXXMethodDecl",
  "name": "operator+",
  "type": {"qualType": "Point (const Point &) const"}
}
```

**注意**：运算符重载需要语义分析确定运算符含义，v2 不支持。

### 6.2 Lambda 表达式

```json
{
  "kind": "LambdaExpr",
  "captureDefault": "none",
  "hasExplicitParams": true,
  "hasExplicitResultType": false,
  "paraments": [
    {"kind": "ParmVarDecl", "name": "x", "type": {"qualType": "int"}}
  ],
  "body": {
    "kind": "CompoundStmt",
    "inner": []
  }
}
```

**注意**：Lambda 需要闭包分析，v2 部分支持。

### 6.3 可变参数模板

```json
{
  "kind": "VariadicTemplate"
}
```

**注意**：可变参数模板解析复杂，v2 部分支持。

## 7. libclang API 参考

### 7.1 解析流程

```rust
use clang::{Index, CursorKind};

fn parse_cpp_file(path: &Path) -> Result<()> {
    // 1. 创建 Index
    let index = Index::new(false, true);

    // 2. 解析翻译单元
    let tu = index.parse_translation_unit(
        path,
        &["-std=c++17", "-I/usr/include"],
    )?;

    // 3. 遍历 AST
    let cursor = tu.cursor();
    visit_children(&cursor, &mut |c| {
        println!("{:?} : {:?}", c.kind(), c.spelling());
    });

    Ok(())
}
```

### 7.2 Cursor 访问

```rust
fn visit_children<F>(cursor: &Cursor, visitor: &mut F)
where
    F: FnMut(Cursor) -> bool,
{
    cursor.visit_children(&mut |c| {
        if visitor(c) {
            visit_children(&c, visitor);
        }
        clang::CXChildVisitResult::Recurse
    });
}
```

### 7.3 获取类型信息

```rust
fn get_type_info(cursor: &Cursor) -> Option<String> {
    let ty = cursor.cur_type()?;
    Some(ty.spelling()?)
}

fn get_function_signature(cursor: &Cursor) -> Option<String> {
    let ty = cursor.cur_type()?;
    Some(ty.spelling()?)
}
```

## 8. 总结

### 8.1 v1 支持的 AST 节点

| 节点类型 | 用途 | 覆盖 |
|----------|------|------|
| `NamespaceDecl` | 命名空间 | ✅ |
| `CXXRecordDecl` | 类/结构体 | ✅ |
| `CXXMethodDecl` | 类方法 | ✅ |
| `CXXConstructorDecl` | 构造函数 | ✅ |
| `CXXDestructorDecl` | 析构函数 | ✅ |
| `EnumDecl` | 枚举 | ✅ |
| `FunctionDecl` | 函数 | ✅ |
| `FieldDecl` | 成员字段 | ✅ |
| `VarDecl` | 变量/静态成员 | ✅ |
| `ParmVarDecl` | 函数参数 | ✅ |
| `TypedefDecl` | 类型别名 | ✅ |

### 8.2 v2 新增的 AST 节点

| 节点类型 | 用途 | 说明 |
|----------|------|------|
| `ClassTemplateDecl` | 模板声明 | 需要实例化分析 |
| `ClassTemplateSpecialization` | 模板实例化 | **v2 核心** |
| `TemplateRef` | 模板引用 | 用于追踪 |
| `TypeRef` | 类型引用 | 用于获取完整类型 |
| `LambdaExpr` | Lambda 表达式 | 部分支持 |
| `CXXNewExpr` | new 表达式 | 智能指针分析 |
| `CXXDeleteExpr` | delete 表达式 | 析构分析 |

### 8.3 永久不支持

| 节点类型 | 原因 |
|----------|------|
| `FriendDecl` | 友元函数访问控制复杂 |
| `CXXTypeidExpr` | RTTI 需要运行时信息 |
| `CXXReinterpretCastExpr` | 类型重新解释需要语义分析 |
