# C++ 到 Rust Safe FFI 自动化工具 - 方案 v3

## 1. 背景

### 1.1 问题

C++ 的模板实例化发生在编译器的语义分析阶段，纯头文件解析无法捕获。

```
源文件 (.cpp)
    ↓ 预处理 (-E): 宏展开、#include 展开
    ↓ 语法分析: 生成 AST（模板还是声明）
    ↓ 语义分析: 模板实例化（std::vector<int> 在此时确定）
    ↓ 生成目标文件 (.o)
```

| 方案 | 原理 | 模板实例化 |
|------|------|------------|
| 预处理捕获 | LD_PRELOAD hook，执行 `-E` | ❌ 仅宏展开 |
| 头文件解析 | libclang 解析 .h | ❌ 无实例化 |
| **AST 编译捕获** | libclang 编译源文件，遍历 AST | ✅ 完整支持 |

**v2** 采用 AST 编译捕获，解决了模板实例化问题。

### 1.2 v2 的局限

v2 仍有一些 C++ 特性无法直接生成等价的 Rust FFI 代码：

| 特性 | v2 支持 | 局限原因 |
|------|---------|----------|
| 运算符重载 `operator+` | ❌ | C++/Rust 运算符语义不完全等价 |
| 友元函数 | ❌ | 可访问私有成员，FFI 映射困难 |
| Lambda 表达式 | ⚠️ | 隐式捕获 Rust 不支持 |
| typeid/RTTI | ❌ | 运行时类型信息静态分析无法获取 |
| 可变参数模板 | ⚠️ | 参数包展开语义复杂 |

### 1.3 v3 思路

**后处理降级**：将 C++ 高级特性转换为更基础的 Rust FFI 表达，同时生成清晰的 TODO 指引。

```
┌─────────────────────────────────────────────────────────────────┐
│                        init 阶段                                 │
│   C++ 源文件 ──▶ AST 编译捕获 ──▶ 后处理 ──▶ Rust FFI + TODO  │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼ 用户手动处理 TODO
┌─────────────────────────────────────────────────────────────────┐
│                       merge 阶段                                 │
│   Rust FFI + TODO ──▶ 合并处理 ──▶ 纯 Rust 项目                │
└─────────────────────────────────────────────────────────────────┘
```

**init 阶段**：生成 Rust FFI 代码 + TODO 清单（TODO 独立文件）
**merge 阶段**：用户处理完 TODO 后，生成纯 Rust 项目

## 2. 技术方案

### 2.1 整体架构

```
cpp2rust-ffi tool (v3)
├── src/
│   ├── main.rs                          # CLI 入口
│   ├── compiler/                         # AST 编译引擎
│   │   ├── ast_compiler.rs              # libclang 封装
│   │   └── cursor_visitor.rs            # AST 遍历器
│   ├── extractor/                        # 信息提取器
│   │   ├── class_extractor.rs           # 类/结构体
│   │   ├── function_extractor.rs         # 函数
│   │   ├── template_extractor.rs         # 模板实例化
│   │   ├── vtable_extractor.rs          # 虚函数表
│   │   └── enum_extractor.rs            # 枚举
│   ├── postprocessor/                    # 后处理器（v3 新增）
│   │   ├── operator_fallback.rs          # 运算符重载
│   │   ├── friend_fn_handler.rs         # 友元函数
│   │   ├── lambda_handler.rs            # Lambda
│   │   ├── rtti_handler.rs             # typeid/RTTI
│   │   └── variadic_handler.rs          # 可变参数模板
│   ├── generator/                        # 代码生成器
│   │   ├── class_generator.rs           # 类 FFI
│   │   ├── template_generator.rs         # 模板实例化
│   │   ├── vtable_generator.rs          # 虚函数表
│   │   └── project_generator.rs         # 项目脚手架
│   └── todo_collector.rs                # TODO 清单收集器
└── Cargo.toml
```

### 2.2 四阶段处理流程

```
1. 编译 (compiler/)
   └── libclang 编译 C++ 源文件，触发模板实例化

2. 提取 (extractor/)
   ├── 类/结构体信息
   ├── 函数信息
   ├── 模板实例化 (std::vector<int>)
   ├── 虚函数表信息
   └── 枚举信息

3. 后处理 (postprocessor/) ← v3 新增
   ├── 运算符重载 → 降级为普通 FFI 函数
   ├── 友元函数   → 提取为独立函数
   ├── Lambda     → 闭包转换 / 标记 TODO
   ├── typeid     → 类型名字符串 + 映射表
   └── 可变参数   → 实例化检测

4. 生成 (generator/)
   └── Rust FFI 代码 + TODO 清单
```

### 2.3 输出目录结构

#### init 阶段输出（Rust FFI + TODO）

```
rust_hicc/
├── Cargo.toml
├── build.rs
└── src/
    ├── lib.rs
    ├── foo.rs                    # FFI 代码（无 TODO 注释）
    ├── foo.rs.todo.json          # TODO 清单（独立文件）
    ├── foo.rs.todo.md
    ├── bar.rs
    ├── bar.rs.todo.json
    ├── bar.rs.todo.md
    └── utils.rs
```

#### merge 阶段输出（纯 Rust 项目）

```
rust_hicc_clean/
├── Cargo.toml
├── build.rs
└── src/
    ├── lib.rs
    ├── foo.rs                    # 纯 Rust 代码（无 TODO）
    ├── bar.rs
    └── utils.rs
```

### 2.4 lib.rs 格式

```rust
//! Rust FFI 库入口
//!
//! 由 cpp2rust-ffi v3 init 自动生成
//! 源文件: foo.cpp, bar.cpp, utils.cpp

pub mod foo;
pub mod bar;
pub mod utils;

pub use foo::Foo;
pub use bar::Bar;

pub type OpaquePtr = *mut std::ffi::c_void;
```

### 2.5 编译单元文件格式

```rust
//! Foo 编译单元 FFI
//!
//! 源文件: foo.cpp

use crate::OpaquePtr;

mod inner {
    use super::*;

    // === cpp! 块 ===
    hicc::cpp! {
        #include "foo.h"
        class Foo {
        public:
            Foo();
            ~Foo();
            void do_something(int value);
        };
    }

    // === 类型别名 ===
    pub type Foo = OpaquePtr;

    // === extern "C" 函数 ===
    #[link(name = "foo")]
    unsafe extern "C" {
        fn foo_new() -> Foo;
        fn foo_delete(f: Foo);
        fn foo_do_something(f: Foo, value: i32);
    }

    // === Rust 封装 ===
    impl Foo {
        pub fn new() -> Self { unsafe { foo_new() } }
        pub fn do_something(&self, value: i32) {
            unsafe { foo_do_something(self.0, value) }
        }
    }
}

pub use inner::Foo;
```

### 2.6 模块划分规则

| C++ 实体 | Rust 映射 | 说明 |
|----------|-----------|------|
| `namespace A { namespace B { ... } }` | `mod A { mod B { ... } }` | 命名空间 → 模块嵌套 |
| `class Foo` (顶层) | `mod foo { ... }` + `pub use foo::Foo;` | 顶层类 → 同名模块 + 类型导出 |
| `class Foo` (嵌套) | `mod A { mod B { pub struct Foo; } }` | 嵌套类 → 嵌套模块 |
| `enum Bar` | `pub enum Bar` | 枚举直接定义 |
| `template<class T> class Foo` | 只处理实例化 | 模板不实例化不生成 |

## 3. 核心设计

### 3.1 数据结构

```rust
/// AST 编译引擎
pub struct AstCompiler {
    index: clang::Index,
    compiler_args: Vec<String>,
}

impl AstCompiler {
    pub fn compile(&self, source_path: &Path) -> Result<CompilationResult>;
}

/// 编译结果
pub struct CompilationResult {
    pub template_instantiations: Vec<TemplateInstantiation>,
    pub types: Vec<TypeInfo>,
    pub functions: Vec<FunctionInfo>,
    pub vtable_info: Vec<VtableInfo>,
}

/// TODO 条目
pub struct TodoItem {
    pub id: String,
    pub severity: Severity,
    pub feature: FeatureType,
    pub file: String,
    pub line: u32,
    pub column: u32,
    pub source_snippet: String,
    pub reason: String,
    pub suggested_fix: String,
    pub generated_code: Option<String>,
    pub status: TodoStatus,  // unresolved / resolved
}

#[derive(Debug, Clone)]
pub enum FeatureType {
    OperatorOverload,
    FriendFunction,
    LambdaExpression,
    TypeidRtti,
    VariadicTemplate,
}

#[derive(Debug, Clone)]
pub enum Severity {
    Error,   // 必须手动处理
    Warning, // 建议手动处理
    Info,    // 可选处理
}

#[derive(Debug, Clone)]
pub enum TodoStatus {
    Unresolved,
    Resolved,
}
```

### 3.2 libclang AST 编译捕获

```rust
fn capture_instantiated_types(source_path: &Path) -> Result<Vec<TemplateInstantiation>> {
    let index = Index::new(false, true);

    // 编译源文件（触发模板实例化）
    let tu = index.parse_translation_unit(
        source_path,
        &["-std=c++17", "-I/usr/include/c++/11"],
    )?;

    let mut instantiations = Vec::new();
    let cursor = tu.cursor();

    visit_children(&cursor, &mut |c| {
        match c.kind() {
            // 模板实例化：std::vector<int>
            CursorKind::ClassTemplateSpecialization => {
                if let Some(inst) = extract_template_instantiation(&c) {
                    instantiations.push(inst);
                }
            }
            // 虚函数
            CursorKind::CXXMethodDecl => {
                if c.is_virtual_method() {
                    // 处理虚函数
                }
            }
            // 抽象类
            CursorKind::CXXRecordDecl => {
                if c.is_abstract_class() {
                    // 提取 vtable 信息
                }
            }
            _ => {}
        }
    });

    Ok(instantiations)
}
```

### 3.3 模板实例化处理

```rust
impl TemplateInstantiation {
    pub fn is_stl_container(&self) -> bool {
        matches!(self.template_name.as_str(),
            "vector" | "list" | "deque" |
            "map" | "set" | "unordered_map" | "unordered_set" |
            "string" | "basic_string"
        )
    }

    pub fn to_rust_ffi_type(&self) -> RustType {
        if self.is_stl_container() {
            self.to_hicc_std_type()
        } else {
            self.to_opaque_ptr_type()
        }
    }
}
```

### 3.4 虚函数表映射

```rust
pub struct VtableInfo {
    pub class_name: String,
    pub full_name: String,
    pub virtual_methods: Vec<VirtualMethod>,
    pub bases: Vec<BaseInfo>,
}

impl VtableInfo {
    pub fn to_rust_trait(&self) -> String {
        let methods: Vec<String> = self.virtual_methods
            .iter()
            .map(|m| {
                let args = extract_args(&m.signature);
                let ret = extract_return_type(&m.signature);
                if m.is_pure_virtual {
                    format!("fn {}({}) -> {} {{ unimplemented!() }}", m.name, args, ret)
                } else {
                    format!("fn {}({}) -> {};", m.name, args, ret)
                }
            })
            .collect();
        format!("trait {} {{\n    {}\n}}", self.class_name, methods.join("\n    "))
    }
}
```

## 4. 后处理器设计

### 4.1 后处理器接口

```rust
pub trait PostProcessor {
    fn process(&self, ast_info: &AstInfo) -> (ProcessedResult, Vec<TodoItem>);
    fn priority(&self) -> u32;
}

impl AstInfo {
    pub fn post_process(&self) -> PostProcessResult {
        let mut all_todos = Vec::new();
        let mut result = self.clone();

        let mut processors: Vec<Box<dyn PostProcessor>> = vec![
            Box::new(VariadicTemplateHandler::new()),
            Box::new(OperatorOverloadHandler::new()),
            Box::new(FriendFunctionHandler::new()),
            Box::new(LambdaHandler::new()),
            Box::new(RttiHandler::new()),
        ];
        processors.sort_by_key(|p| p.priority());

        for processor in processors {
            let (processed, todos) = processor.process(&result);
            result.merge(processed);
            all_todos.extend(todos);
        }

        all_todos.sort();
        all_todos.dedup();
        PostProcessResult { ast_info: result, todo_items: all_todos }
    }
}
```

### 4.2 运算符重载处理

**问题**：`operator+` 等无法直接映射到 Rust trait。

| 运算符 | C++ 行为 | Rust trait | 差异 |
|--------|----------|------------|------|
| `operator[]` | 可返回引用 | `Index` 返回值 | C++ 可用于 `a[i] = x` |
| `operator=` | 移动/拷贝语义 | `Assign` | 生命周期差异 |
| `operator++` | 前后置均可 | `Increment` | 后置语义不同 |

**方案**：降级为普通 FFI 函数

```rust
// 输入
class Vec2 { Vec2 operator+(const Vec2& other) const; };

// 输出
#[link(name = "lib")]
unsafe extern "C" {
    fn vec2_add(a: Vec2, b: Vec2) -> Vec2;
}
```

### 4.3 友元函数处理

**问题**：友元函数可访问私有成员。

**方案**：提取为独立函数 + TODO

```rust
// 输入
class Calculator { friend int helper(Calculator& c, int x); };

// 输出
#[link(name = "lib")]
unsafe extern "C" {
    fn helper(c: Calculator, x: i32) -> i32;
}
```

### 4.4 Lambda 处理

**问题**：隐式捕获 `[=]`/`[&]` Rust 不支持。

| 场景 | C++ | Rust | 处理 |
|------|-----|------|------|
| 按值捕获 | `[x]` | `move x` | ✅ 直接转换 |
| 按引用捕获 | `[&x]` | `&x` | ⚠️ 需确认 |
| 隐式捕获 | `[=]`/`[&]` | 不支持 | ❌ 需手动处理 |
| 捕获 this | `[this]` | `self` | ⚠️ 需手动处理 |

**方案**：简单 Lambda 转闭包，复杂场景生成 TODO

```rust
// 输入
auto simple = [](int x) { return x + 1; };
auto implicit = [=](int z) { return z * 2; };

// 输出
let simple: fn(i32) -> i32 = |x| x + 1;
// TODO: 隐式捕获 [=] 需改为 move |z| ...
```

### 4.5 typeid/RTTI 处理

**问题**：typeid 返回运行时类型名，静态分析无法获取。

**方案**：生成类型名字符串函数 + 用户映射表

```rust
// 输入
auto name = typeid(*ptr).name();

// 输出
#[link(name = "lib")]
unsafe extern "C" {
    fn get_type_name(ptr: *mut c_void) -> *const i8;
}

// TODO: 用户需维护映射表
static TYPE_MAP: &[(&str, &str)] = &[
    ("N7DerivedE", "Derived"),      // GCC
    ("class Derived", "Derived"),  // Clang
];
```

### 4.6 可变参数模板处理

**问题**：`Args...` 展开语义复杂。

**方案**：检测实例化调用点，未覆盖的生成 TODO

```rust
// 输入
template<typename... Args>
void print_all(Args... args);

// 调用：print_all(1, "hello", 3.14);

// 输出
#[link(name = "lib")]
unsafe extern "C" {
    fn print_all_i_cs(a: i32, b: *const i8, c: f64);
}
```

## 5. TODO 清单

### 5.1 JSON 格式（机器可解析）

```json
{
  "version": "v3",
  "cpp_file": "main.cpp",
  "output_file": "main.rs",
  "todo_count": { "error": 1, "warning": 2, "info": 1 },
  "todos": [
    {
      "id": "OP-001",
      "severity": "error",
      "feature": "operator_overload",
      "file": "main.cpp",
      "line": 42,
      "column": 5,
      "source_snippet": "Vec2 operator+(const Vec2& other) const;",
      "reason": "C++ 运算符无法直接映射到 Rust trait",
      "suggested_fix": "手动实现 std::ops::Add for Vec2 或使用降级的 FFI 函数 vec2_add",
      "generated_code": "fn vec2_add(a: Vec2, b: Vec2) -> Vec2;",
      "status": "unresolved"
    }
  ]
}
```

### 5.2 Markdown 表格格式（人类可读）

```markdown
# TODO 报告

**源文件**: main.cpp | **输出**: main.rs
**统计**: ❌ 1 Error | ⚠️ 2 Warning | ℹ️ 1 Info

---

## ❌ Error: 必须手动处理

| ID | 特性 | 位置 | 代码片段 | 原因 | 处理方案 |
|----|------|------|----------|------|----------|
| OP-001 | 运算符重载 | main.cpp:42 | `Vec2 operator+(...)` | C++ 运算符无法映射到 Rust trait | 手动实现 `std::ops::Add` |

**已生成代码**:
```rust
fn vec2_add(a: Vec2, b: Vec2) -> Vec2;
```

---

## ⚠️ Warning: 建议手动处理

| ID | 特性 | 位置 | 代码片段 | 原因 | 处理方案 |
|----|------|------|----------|------|----------|
| FR-001 | 友元函数 | main.cpp:15 | `friend int helper(...)` | 访问了私有成员 | 确认是否暴露为 API |
| LM-001 | Lambda | main.cpp:28 | `[=](int z){...}` | 隐式捕获需显式指定 | 改为 `move \|z\| ...` |
```

### 5.3 CLI 命令

```bash
# init 阶段：生成 Rust FFI 代码 + TODO 清单
$ cpp2rust-ffi init -i ./cpp -o ./rust_hicc

⚠️  4 个需要手动处理的特性
  → 详细清单见 *.todo.json 或 *.todo.md

# merge 阶段：处理完 TODO 后，生成纯 Rust 项目
$ cpp2rust-ffi merge -i ./rust_hicc -o ./rust_hicc_clean

[cpp2rust-ffi] 检测到 4 个 TODO 已处理
[cpp2rust-ffi] 生成纯 Rust 项目 ./rust_hicc_clean/
```

**merge 命令行为**：
- 检查所有 `.todo.json` 文件，确认 `status: "resolved"`
- 将用户处理后的代码合并到输出目录
- 删除所有 `.todo.json` 和 `.todo.md` 文件

## 6. 特性覆盖详情

### 6.1 统计总览

| 类别 | 示例数 | ✅ 直接 | ⚠️ 后处理 |
|------|--------|---------|-----------|
| 基础类型与函数 | 5 | 5 | 0 |
| 类与对象 | 7 | 7 | 0 |
| 面向对象特性 | 6 | 6 | 0 |
| 运算符与类型 | 5 | 3 | 2 |
| 模板 | 5 | 4 | 1 |
| 智能指针与内存 | 5 | 5 | 0 |
| STL 容器 | 5 | 5 | 0 |
| 函数对象 | 4 | 2 | 2 |
| 其他高级特性 | 6 | 6 | 0 |
| **总计** | **48** | **43** | **5** |

### 6.2 后处理特性详情

| 特性 | 示例 | 处理方式 |
|------|------|----------|
| 运算符重载 | 019 | 降级为 FFI 函数 `vec2_add(a, b)` |
| 友元函数 | 020 | 提取为独立 `extern "C"` 函数 |
| typeid/RTTI | 023 | 生成 `get_type_name()` + 用户映射表 |
| Lambda | 039 | 简单场景转闭包，复杂场景 TODO |
| 可变参数模板 | 028 | 检测实例化调用点 |

### 6.3 详细表格（48 个示例）

#### 基础类型与函数 (1-5)

| 示例 | 特性 | AST 节点 | 支持 | 处理方式 | 原因 |
|------|------|----------|------|----------|------|
| 001_hello_world | extern "C" 函数 | `FunctionDecl` | ✅ | 直接生成 | - |
| 002_function_overload | 函数重载 | `FunctionDecl` (多个同名) | ✅ | 直接生成 | - |
| 003_default_args | 默认参数 | `ParmVarDecl` (带默认值) | ✅ | 直接生成 | - |
| 004_inline_functions | 内联函数 | `FunctionDecl` + `inline` | ✅ | 直接生成 | - |
| 005_variadic_functions | 可变参数函数 | `FunctionDecl` (可变参数) | ✅ | 直接生成 | - |

#### 类与对象 (6-12)

| 示例 | 特性 | AST 节点 | 支持 | 处理方式 | 原因 |
|------|------|----------|------|----------|------|
| 006_class_basic | 基础类 | `CXXRecordDecl` | ✅ | opaque pointer + FFI 函数 | - |
| 007_class_constructor | 构造/析构函数 | `CXXConstructorDecl` | ✅ | `*_new()` / `*_delete()` | - |
| 008_class_copy | 拷贝构造函数 | `CXXConstructorDecl` (copy) | ✅ | `*_copy()` 函数 | - |
| 009_class_move | 移动构造函数 | `CXXConstructorDecl` (move) | ✅ | `*_move()` 函数 | - |
| 010_class_static | 静态成员 | `VarDecl` (static) | ✅ | 静态变量访问函数 | - |
| 011_class_const | const 成员函数 | `CXXMethodDecl` (const) | ✅ | const 限定 FFI 函数 | - |
| 012_class_volatile | volatile 成员函数 | `CXXMethodDecl` (volatile) | ✅ | volatile 限定 FFI 函数 | - |

#### 面向对象特性 (13-18)

| 示例 | 特性 | AST 节点 | 支持 | 处理方式 | 原因 |
|------|------|----------|------|----------|------|
| 013_inheritance_single | 单继承 | `CXXBaseSpecifier` | ✅ | 继承链在 AST 中 | - |
| 014_inheritance_multiple | 多继承 | `CXXBaseSpecifier` (多个) | ✅ | 多继承在 AST 中 | - |
| 015_virtual_basic | 虚函数基础 | `CXXMethodDecl` (virtual) | ✅ | 虚函数表映射 | - |
| 016_virtual_pure | 纯虚函数/抽象类 | `CXXMethodDecl` (= 0) | ✅ | Rust trait 接口 | - |
| 017_virtual_override | override 说明符 | `CXXMethodDecl` (override) | ✅ | override 在 AST 中 | - |
| 018_virtual_diamond | 菱形继承 | `CXXBaseSpecifier` (virtual) | ✅ | virtual 继承在 AST 中 | - |

#### 运算符与类型 (19-23)

| 示例 | 特性 | AST 节点 | 支持 | 处理方式 | 原因 |
|------|------|----------|------|----------|------|
| 019_operator_overload | 运算符重载 | `CXXMethodDecl` (operator) | ⚠️ | 降级为 FFI 函数 + TODO | C++/Rust 运算符语义差异 |
| 020_friend_function | 友元函数 | `FriendDecl` | ⚠️ | 提取为独立函数 + TODO | 访问私有成员困难 |
| 021_explicit_ctor | explicit 构造函数 | `CXXConstructorDecl` (explicit) | ✅ | explicit 标记 | - |
| 022_mutable_member | mutable 成员 | `FieldDecl` (mutable) | ✅ | 可变字段访问函数 | - |
| 023_typeid_rtti | typeid 与 RTTI | `CXXTypeidExpr` | ⚠️ | 类型名字符串 + 映射表 + TODO | 运行时类型静态分析无法获取 |

#### 模板 (24-28)

| 示例 | 特性 | AST 节点 | 支持 | 处理方式 | 原因 |
|------|------|----------|------|----------|------|
| 024_template_function | 函数模板 | `FunctionTemplateDecl` | ✅ | 实例化时生成 | - |
| 025_template_class | 类模板 | `ClassTemplateDecl` | ✅ | 实例化时生成 | - |
| 026_template_specialization | 模板偏特化 | `ClassTemplatePartialSpecialization` | ✅ | 只处理实例化 | 偏特化只是声明 |
| 027_template_instantiation | 模板显式实例化 | `ClassTemplateSpecialization` | ✅ | 捕获实例化 | - |
| 028_variadic_template | 可变参数模板 | `VariadicTemplate` | ⚠️ | 检测实例化 + TODO | 参数包展开复杂 |

#### 智能指针与内存 (29-33)

| 示例 | 特性 | AST 节点 | 支持 | 处理方式 | 原因 |
|------|------|----------|------|----------|------|
| 029_unique_ptr | std::unique_ptr | `CXXNewExpr` | ✅ | hicc-smart-ptr 包装 | - |
| 030_shared_ptr | std::shared_ptr | `CXXNewExpr` | ✅ | hicc-smart-ptr 包装 | - |
| 031_custom_deleter | 自定义删除器 | `FunctionDecl` | ✅ | 自定义删除器代码 | - |
| 032_placement_new | Placement new | `CXXNewExpr` | ✅ | placement new 调用 | - |
| 033_raii_pattern | RAII 模式 | 构造/析构函数 | ✅ | 构造/析构配对 | - |

#### STL 容器 (34-38)

| 示例 | 特性 | AST 节点 | 支持 | 处理方式 | 原因 |
|------|------|----------|------|----------|------|
| 034_vector_basic | std::vector | `ClassTemplateSpecialization` | ✅ | hicc-std::vector | - |
| 035_map_basic | std::map | `ClassTemplateSpecialization` | ✅ | hicc-std::map | - |
| 036_string_basic | std::string | `ClassTemplateSpecialization` | ✅ | hicc-std::string | - |
| 037_array_basic | std::array | `ClassTemplateSpecialization` | ✅ | hicc-std::array | - |
| 038_tuple_basic | std::tuple | `ClassTemplateSpecialization` | ✅ | hicc-std::tuple | - |

#### 函数对象 (39-42)

| 示例 | 特性 | AST 节点 | 支持 | 处理方式 | 原因 |
|------|------|----------|------|----------|------|
| 039_lambda_basic | Lambda 表达式 | `LambdaExpr` | ⚠️ | 简单转闭包，复杂 TODO | 隐式捕获不支持 |
| 040_std_function | std::function | `ClassTemplateSpecialization` | ✅ | hicc-std::function | - |
| 041_functional_bind | std::bind | `CallExpr` | ⚠️ | bind 框架 + TODO | 参数绑定复杂 |
| 042_exception_basic | 异常处理 | `CXXThrowExpr` | ✅ | 异常处理框架 | - |

#### 其他高级特性 (43-48)

| 示例 | 特性 | AST 节点 | 支持 | 处理方式 | 原因 |
|------|------|----------|------|----------|------|
| 043_namespace_nested | 嵌套命名空间 | `NamespaceDecl` (嵌套) | ✅ | 模块或 `_` 分隔 | - |
| 044_enum_class | 强类型枚举 | `EnumDecl` (scoped) | ✅ | Rust 枚举类型 | - |
| 045_union_basic | 共用体 | `RecordDecl` (union) | ✅ | Rust union | - |
| 046_constexpr_basic | constexpr | `Expr` (constexpr) | ✅ | constexpr 在 AST 中 | - |
| 047_noexcept_basic | noexcept | `NoexceptSpec` | ✅ | noexcept 在 AST 中 | - |
| 048_summary | FFI 模式总结 | - | ✅ | 综合示例 | - |

**图例**：✅ 完全支持 ⚠️ 部分支持（后处理降级 + TODO）

## 7. 实现计划

| 阶段 | 内容 | 优先级 | 覆盖 |
|------|------|--------|------|
| Phase 1 | v2 AST 编译捕获（复刻） | P0 | 所有 v3 基础 |
| Phase 2 | 后处理器基础框架 | P0 | 后处理基础设施 |
| Phase 3 | 运算符重载后处理 | P1 | 019 |
| Phase 4 | 友元函数后处理 | P1 | 020 |
| Phase 5 | Lambda 后处理 | P1 | 039 |
| Phase 6 | typeid/RTTI 后处理 | P2 | 023 |
| Phase 7 | 可变参数模板后处理 | P2 | 028 |
| Phase 8 | TODO 清单生成器 | P1 | 全部 |
| Phase 9 | 集成测试 + 48 示例验证 | P1 | 全部 |

## 8. 技术依赖

```toml
[dependencies]
clang = "0.1"     # libclang 绑定
clap = "4"         # CLI
anyhow = "1"       # 错误处理
serde = { version = "1", features = ["derive"] }
serde_json = "1"

[build-dependencies]
cc = "1"           # C++ 编译器调用
```

```bash
apt-get install clang-18 libclang-18-dev
apt-get install libstdc++-12-dev
```

## 9. 总结

v3 在 v2 的 AST 编译捕获基础上，新增后处理降级机制：

1. **全覆盖**：48 个示例中 43 个直接生成，5 个后处理降级
2. **清晰指引**：生成结构化 TODO 清单（位置、内容、原因、处理方案）
3. **两阶段分离**：init 生成 FFI + TODO，merge 生成纯 Rust 项目
4. **用户可控**：根据 TODO 指引手动补充，无需猜测

**关键改进**：
- 运算符重载、友元函数、typeid/RTTI、Lambda、可变参数模板这 5 个特性通过后处理降级实现支持
- TODO 清单独立成文件，不污染 Rust 源码
- 用户处理完 TODO 后，merge 生成纯 Rust 项目
