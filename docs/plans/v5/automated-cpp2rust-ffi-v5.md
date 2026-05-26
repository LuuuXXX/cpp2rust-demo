# C++ 到 Rust Safe FFI 自动化工具 - 方案 v5

## 1. 概述

### 1.1 核心思路

v5 通过 **LD_PRELOAD 编译拦截**机制，在真实编译过程中捕获 C++ 代码信息，生成 Rust FFI 脚手架。

**关键理念**：C++ 模板的价值在于**实例化结果**，而非模板本身。转换时只关注实际被实例化的具体类型（如 `std::vector<int>`），不关注模板声明。

### 1.2 参考实现来源

| 文件 | 源位置 | 用途 |
|------|--------|------|
| `hook/hook.c` | `cpp2rust-demo-20260524/references/c2rust-demo/hook/` | 预处理逻辑（C 版本） |
| `examples/cpp-hook/hook.cpp` | `cpp2rust-demo-20260524/examples/cpp-hook/` | C++ 编译器检测逻辑 |
| `src/capture.rs` | `cpp2rust-demo-20260524/references/c2rust-demo/` | LD_PRELOAD 执行逻辑 |

**注意**：需创建新的 `hook.cpp`，合并两者优点。不直接复用现有文件。

### 1.3 版本定位

v5 是完全独立的新版本，所有输入都必须通过 LD_PRELOAD 编译拦截方式获取。

---

## 2. 快速开始

### 2.1 工作流程

```bash
# Step 0: 准备 Hook 文件
# 创建 hook/hook.cpp（基于 hook.c + cpp-hook/hook.cpp）
# 复制 capture.rs
cp ../cpp2rust-demo-20260524/references/c2rust-demo/src/capture.rs ./src/

# Step 1: 编译拦截
cd cpp-project/
C2RUST_FEATURE_ROOT=.c2rust/v5 \
C2RUST_PROJECT_ROOT=/path/to/cpp-project \
C2RUST_CXX=g++ \
LD_PRELOAD=/path/to/libhook.so \
    make -j4

# Step 2: 初始化
cpp2rust-ffi init -i .c2rust/v5 -o ./rust_hicc

# Step 3: 合并（如需要）
cpp2rust-ffi merge -i ./rust_hicc
```

### 2.2 环境变量

| 变量 | 必填 | 说明 |
|------|------|------|
| `C2RUST_FEATURE_ROOT` | ✅ | 捕获产物输出目录 |
| `C2RUST_PROJECT_ROOT` | ✅ | C++ 项目根目录 |
| `C2RUST_CXX` | ❌ | C++ 编译器，默认检测 g++/clang++/c++ |
| `C2RUST_DEBUG` | ❌ | 设置为 1 输出调试日志 |

---

## 3. 技术架构

### 3.1 整体架构

```
cpp2rust-ffi tool (v5)
├── src/
│   ├── main.rs                    # CLI 入口 (init / merge)
│   ├── hook/                     # LD_PRELOAD Hook 库
│   │   ├── hook.cpp           # C++ 拦截器（新建）
│   │   └── Makefile
│   ├── capture.rs                # 复用：LD_PRELOAD 执行逻辑
│   ├── ast_parser.rs            # C++ AST 解析（clang crate）
│   ├── extractor/               # 信息提取
│   │   ├── class_extractor.rs
│   │   ├── function_extractor.rs
│   │   └── enum_extractor.rs
│   ├── postprocessor/          # 后处理
│   │   ├── operator_handler.rs
│   │   ├── friend_handler.rs
│   │   └── lambda_handler.rs
│   ├── generator/              # 代码生成
│   │   ├── hicc_codegen.rs
│   │   └── project_generator.rs
│   └── instantiation_tracker.rs # 模板实例化追踪
└── Cargo.toml
```

**关键依赖**：`clang = "2"` - libclang 绑定（C++ 支持）

### 3.2 三阶段处理流程

```
┌─────────────────────────────────────────────────────────────┐
│ 1. 编译拦截 (hook.cpp)                                      │
│    LD_PRELOAD 注入 → 预处理捕获 → 编译                        │
│    输出: c/*.c2rust（宏展开后的 C++ 代码）                     │
└─────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────┐
│ 2. AST 提取 (ast_parser.rs + extractor/)                   │
│    clang crate 解析 .c2rust → 类/函数/模板实例化/枚举          │
└─────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────┐
│ 3. 代码生成 (generator/)                                    │
│    hicc 宏格式 Rust 代码 → lib.rs + <unit>.rs               │
└─────────────────────────────────────────────────────────────┘
```

---

## 4. Hook 机制

### 4.1 新建 hook.cpp（必须）

| 现有文件 | 问题 | 解决方案 |
|---------|------|---------|
| `hook/hook.c` | 只支持 C 编译器，只检测 `.c` 文件 | 合并到新 hook.cpp |
| `examples/cpp-hook/hook.cpp` | 输出 AST JSON（redirect bug） | 合并到新 hook.cpp，改为输出预处理文件 |

**新 hook.cpp 特性**：

| 特性 | 值 |
|------|---|
| 编译器检测 | `g++`, `clang++`, `c++`（支持 versioned：`g++-13`） |
| 文件扩展名 | `.cpp`, `.cc`, `.cxx`, `.c++`, `.C`, `.cp` |
| 预处理参数 | `-E -C -P`（删除行号，简化处理） |
| 输出 | `.c2rust` 宏展开文件 |

### 4.2 预处理捕获

```bash
# 预处理命令（hook.cpp 新实现）
g++ -E -C -P -I<inc_paths> -D<defs> foo.cpp -o foo.cpp.c2rust
```

**说明**：使用 `-P` 删除行号信息，简化后续解析。

### 4.3 输出目录结构

```
.c2rust/v5/
├── c/                           # 预处理文件
│   └── src/
│       └── foo.cpp.c2rust      # 宏展开后的 C++ 代码
└── targets.list                  # 链接目标列表
```

---

## 5. AST 解析

### 5.1 clang crate 解析

```rust
// ast_parser.rs
use clang::Clang;

pub fn parse_preprocessed(file: &Path) -> Result<CppAst> {
    let clang = Clang::new()?;
    let index = Index::new(&clang, false, false);

    let tu = index.parser(file)
        .detailed_preprocessing(true)
        .parse()
        .unit();

    for child in tu.cursor().children() {
        match child.kind() {
            CXCursorKind::CXXRecordDecl => { /* 类处理 */ }
            CXCursorKind::FunctionDecl => { /* 函数处理 */ }
            CXCursorKind::ClassTemplateSpecialization => { /* 模板实例化 */ }
            // ...
        }
    }
}
```

### 5.2 支持的 C++ AST 节点

| 节点类型 | clang crate | v5 用途 |
|---------|-------------|--------|
| `CXXRecordDecl` | ✅ | 类/结构体定义 |
| `CXXMethodDecl` | ✅ | 成员函数 |
| `FunctionDecl` | ✅ | 全局函数 |
| `ClassTemplateSpecialization` | ✅ | 模板实例化 |
| `NamespaceDecl` | ✅ | 命名空间 |
| `EnumDecl` | ✅ | 枚举 |
| `CXXConstructorDecl` | ✅ | 构造函数 |
| `CXXDestructorDecl` | ✅ | 析构函数 |

---

## 6. 输出格式

### 6.1 Rust 项目结构

```
rust_hicc/
├── Cargo.toml
└── src/
    ├── lib.rs              # 库入口
    ├── foo.rs              # 编译单元 foo
    ├── bar.rs              # 编译单元 bar
    └── baz.rs              # 编译单元 baz
```

### 6.2 Rust 文件格式（三段式）

```rust
// ========== 1. C++ 实现内联（hicc::cpp! 块） ==========
hicc::cpp! {
    #include "foo.h"

    Foo* foo_new(int value) { return new Foo(value); }
    void foo_delete(Foo* self) { delete self; }
}

// ========== 2. 类方法绑定（hicc::import_class! 块） ==========
hicc::import_class! {
    #[cpp(class = "Foo")]
    class Foo {
        #[cpp(method = "int getValue() const")]
        fn getValue(&self) -> i32;
    }
}

// ========== 3. 全局函数绑定（hicc::import_lib! 块） ==========
hicc::import_lib! {
    #![link_name = "foo"]

    class Foo;

    #[cpp(func = "Foo* foo_new(int value)")]
    fn foo_new(value: i32) -> *mut Foo;

    #[cpp(func = "void foo_delete(Foo* self)")]
    unsafe fn foo_delete(self_: *mut Foo);
}
```

---

## 7. C++ 特性支持

### 7.1 总览

> 图例：✅ 完全自动生成可编译代码　⚠️ 降级生成 + 内联 TODO（代码仍可通过 `cargo check`）

| 类别 | 数量 | ✅ | ⚠️ |
|------|------|----|----|
| 基础类型与函数 | 5 | 5 | 0 |
| 类与对象 | 7 | 7 | 0 |
| 面向对象特性 | 6 | 6 | 0 |
| 运算符与类型 | 5 | 4 | 1 |
| 模板实例化 | 5 | 4 | 1 |
| 智能指针与内存 | 5 | 5 | 0 |
| STL 容器 | 5 | 5 | 0 |
| 函数对象 | 4 | 2 | 2 |
| 其他高级特性 | 6 | 6 | 0 |
| **总计** | **48** | **44** | **4** |

---

### 7.2 基础类型与函数（001–005）

| # | 示例 | C++ 特性 | 支持状态 | AST 节点 | 处理方式 |
|---|------|---------|---------|---------|---------|
| 001 | hello_world | extern "C" 函数 | ✅ | `FunctionDecl` | 直接生成 `import_lib!` 条目 |
| 002 | function_overload | 函数重载 | ✅ | `FunctionDecl`（多个同名） | 每个重载生成独立 shim + 独立 `import_lib!` 条目，名称加后缀区分（如 `_i32`, `_f64`） |
| 003 | default_args | 默认参数 | ✅ | `ParmVarDecl`（含默认值） | shim 函数提供所有默认值组合的重载版本（展开为多个固定参数函数） |
| 004 | inline_functions | inline 函数 | ✅ | `FunctionDecl` + `inline` | 函数体内联写入 `hicc::cpp!` 块；`import_lib!` 照常声明 |
| 005 | variadic_functions | C 可变参数函数（va_list） | ✅ | `FunctionDecl`（va_list） | 直接映射 C 可变参数；Rust 侧使用 `unsafe` + `...` 参数（不同于 C++ 可变参数模板） |

---

### 7.3 类与对象（006–012）

| # | 示例 | C++ 特性 | 支持状态 | AST 节点 | 处理方式 |
|---|------|---------|---------|---------|---------|
| 006 | class_basic | 基础类 | ✅ | `CXXRecordDecl` | opaque pointer 模式：`class Foo;` 前向声明 + `import_class!` + `import_lib!` |
| 007 | class_constructor | 构造/析构 | ✅ | `CXXConstructorDecl` / `CXXDestructorDecl` | 生成 `*_new()` 和 `*_delete()` shim 函数 |
| 008 | class_copy | 拷贝构造 | ✅ | `CXXConstructorDecl`（copy） | 生成 `*_copy()` shim，接收 `const Foo*`，返回新 `Foo*` |
| 009 | class_move | 移动构造 | ✅ | `CXXConstructorDecl`（move） | 生成 `*_move()` shim，使用 `std::move()`；原指针逻辑上转移所有权 |
| 010 | class_static | 静态成员 | ✅ | `VarDecl`（static） | 生成 getter/setter shim 函数（`{class}_get_{field}` / `{class}_set_{field}`） |
| 011 | class_const | const 成员函数 | ✅ | `CXXMethodDecl`（const） | Rust 侧映射为 `fn method(&self)`（不可变引用），shim 参数为 `const Foo*` |
| 012 | class_volatile | volatile 成员函数 | ✅ | `CXXMethodDecl`（volatile） | shim 参数标注 `volatile Foo*`，Rust 侧使用 `*mut Foo` 并附注释 |

---

### 7.4 面向对象特性（013–018）

| # | 示例 | C++ 特性 | 支持状态 | AST 节点 | 处理方式 |
|---|------|---------|---------|---------|---------|
| 013 | inheritance_single | 单继承 | ✅ | `CXXBaseSpecifier` | 基类方法提升：子类的 `import_class!` 中同时列出继承来的方法 |
| 014 | inheritance_multiple | 多继承 | ✅ | `CXXBaseSpecifier`（多个） | 多条继承链展开，同名方法按 C++ 查找规则决定选用哪条链 |
| 015 | virtual_basic | 虚函数 | ✅ | `CXXMethodDecl`（virtual） | 通过 opaque pointer 调用虚函数；shim `{class}_{method}(self)` → `self->method()` |
| 016 | virtual_pure | 纯虚/抽象类 | ✅ | `CXXMethodDecl`（= 0） | 抽象类只生成 `class Foo;` 前向声明；子类实现方法正常生成 shim |
| 017 | virtual_override | override | ✅ | `CXXMethodDecl`（override） | override 语义透传；Rust 侧调用方式与普通虚函数相同 |
| 018 | virtual_diamond | 菱形继承（virtual 继承） | ✅ | `CXXBaseSpecifier`（virtual） | 共享基类通过 opaque pointer 统一访问；生成 `{class}_as_{base}()` 类型转换 shim |

---

### 7.5 运算符与类型（019–023）

| # | 示例 | C++ 特性 | 支持状态 | AST 节点 | 原因 & 处理方式 |
|---|------|---------|---------|---------|----------------|
| 019 | operator_overload | 运算符重载 | ⚠️ `[OP]` | `CXXMethodDecl`（operator） | **不能直接 ✅ 的原因**：C ABI 不支持 `operator+` 等符号名，FFI 边界只能传命名函数。**处理方案**：为每个运算符生成命名 shim（如 `number_add` / `number_sub`），写入 `hicc::cpp!` 和 `import_lib!`。追加内联 TODO：`// cpp2rust-todo[OP]: 可实现 impl std::ops::Add<Number> for Number` |
| 020 | friend_function | 友元函数 | ✅ | `FriendDecl` | 友元关系对 FFI 透明；shim 函数体内仍能访问私有成员，直接提取为普通函数写入 `import_lib!`，附注释说明友元身份 |
| 021 | explicit_ctor | explicit 构造函数 | ✅ | `CXXConstructorDecl`（explicit） | `explicit` 防止隐式转换，对 FFI 透明；生成 `*_new()` shim 与普通构造函数相同 |
| 022 | mutable_member | mutable 成员 | ✅ | `FieldDecl`（mutable） | `mutable` 允许 const 方法修改，对 FFI 透明；Rust 侧对应方法仍声明为 `&self` |
| 023 | typeid_rtti | typeid / RTTI / dynamic_cast | ✅ | `CXXTypeidExpr` / `CXXDynamicCastExpr` | 检测到 RTTI 使用场景时，在 `hicc::cpp!` 中注入整数枚举（`ShapeType { CIRCLE=0, ... }`）+ 虚函数（`getType()` / `getTypeName()`），完全绕过 `typeid`。Rust 侧 `match shape_getType(p)` 对应枚举值 |

---

### 7.6 模板实例化（024–028）

| # | 示例 | C++ 特性 | 支持状态 | AST 节点 | 原因 & 处理方式 |
|---|------|---------|---------|---------|----------------|
| 024 | template_function | 函数模板 | ✅ | `ClassTemplateSpecialization`（实例化） | 忽略模板声明（`FunctionTemplateDecl`）；只处理实际被调用的实例化版本（如 `max<int>`），每个具体类型版本生成独立 shim |
| 025 | template_class | 类模板 | ✅ | `ClassTemplateSpecialization` | 忽略模板声明（`ClassTemplateDecl`）；只处理实际实例化的具体类型（如 `Stack<int>`），按普通类生成 shim |
| 026 | template_specialization | 模板偏特化 | ✅ | `ClassTemplatePartialSpecialization` | 偏特化本身视为实例化路径之一；解析时只收集通过该特化路径实例化的类型 |
| 027 | template_instantiation | 显式模板实例化 | ✅ | `ClassTemplateSpecialization` | 显式实例化（`template class Foo<int>;`）直接在 AST 中可见，可直接提取 |
| 028 | variadic_template | 可变参数模板 | ⚠️ `[VA]` | `VariadicTemplate` / `CallExpr` | **不能直接 ✅ 的原因**：C++ 可变参数模板（`...Args`）是编译期展开，FFI 无法表达"任意数量参数"。**处理方案**：扫描 AST 中所有对该模板的 `CallExpr`，收集实际出现的参数数量和类型组合，为每种组合生成一个固定参数版本（如 `sum_2(a, b)`, `sum_3(a, b, c)`）。追加内联 TODO：`// cpp2rust-todo[VA]: 可变参数模板，已展开 N 个调用点，新增调用时需手动添加对应版本` |

---

### 7.7 智能指针与内存（029–033）

| # | 示例 | C++ 特性 | 支持状态 | AST 节点 | 处理方式 |
|---|------|---------|---------|---------|---------|
| 029 | unique_ptr | std::unique_ptr | ✅ | `ClassTemplateSpecialization` | 视为 opaque pointer；生成 `*_new()` 返回裸指针，调用方负责通过 `*_delete()` 释放；不尝试映射所有权语义 |
| 030 | shared_ptr | std::shared_ptr | ✅ | `ClassTemplateSpecialization` | 类似 unique_ptr；shim 管理引用计数的增减（通过 `*_clone()` / `*_delete()` shim） |
| 031 | custom_deleter | 自定义删除器 | ✅ | `FunctionDecl` | 删除器函数注入 `hicc::cpp!`；`*_delete()` shim 内部调用自定义删除器 |
| 032 | placement_new | Placement new | ✅ | `CXXNewExpr` | 生成 `*_placement_new(ptr, ...)` shim，在给定内存地址构造对象 |
| 033 | raii_pattern | RAII 模式 | ✅ | 构造/析构 | 析构函数生成 `*_delete()` shim；Rust 侧可实现 `Drop` trait 调用 `*_delete()` |

---

### 7.8 STL 容器（034–038）

| # | 示例 | C++ 特性 | 支持状态 | AST 节点 | 处理方式 |
|---|------|---------|---------|---------|---------|
| 034 | vector_basic | std::vector\<T\> | ✅ | `ClassTemplateSpecialization` | 视为 opaque pointer；生成 `vec_new`, `vec_delete`, `vec_push`, `vec_get`, `vec_size` 等 shim |
| 035 | map_basic | std::map\<K,V\> | ✅ | `ClassTemplateSpecialization` | 同上；生成 `map_insert`, `map_find`, `map_erase` 等 shim |
| 036 | string_basic | std::string | ✅ | `ClassTemplateSpecialization` | 生成 `str_new`, `str_delete`, `str_c_str`, `str_length` 等 shim；跨 FFI 传字符串用 `*const i8` |
| 037 | array_basic | std::array\<T,N\> | ✅ | `ClassTemplateSpecialization` | 固定大小数组，生成按索引访问 shim；N 在实例化时已知 |
| 038 | tuple_basic | std::tuple\<T...\> | ✅ | `ClassTemplateSpecialization` | 按元素位置生成 `tuple_get_0`, `tuple_get_1` 等 shim；元素类型在实例化时已知 |

---

### 7.9 函数对象（039–042）

| # | 示例 | C++ 特性 | 支持状态 | AST 节点 | 原因 & 处理方式 |
|---|------|---------|---------|---------|----------------|
| 039 | lambda_basic | Lambda 表达式 | ⚠️ `[LM]` | `LambdaExpr` | **不能直接 ✅ 的原因**：有状态 lambda（捕获外部变量）是匿名类型，FFI 无法直接表达捕获列表和闭包类型。**处理方案（双策略）**：① 无状态 lambda（空捕获 `[]`）→ 退化为 `extern "C" fn(...)` 函数指针，直接在 `import_lib!` 中声明为函数指针类型。② 有状态 lambda（含捕获）→ 生成 class wrapper + opaque pointer（`*_new(init)`, `*_call(self, arg)`, `*_delete(self)`）。追加内联 TODO：`// cpp2rust-todo[LM]: 有状态 lambda，内部捕获状态不透明，已封装为 class wrapper` |
| 040 | std_function | std::function\<Sig\> | ✅ | `ClassTemplateSpecialization` | `std::function<int(int)>` 视为有状态函数对象（同 lambda 有状态策略）；生成 class wrapper + opaque pointer |
| 041 | functional_bind | std::bind | ✅ | `CallExpr`（bind）/ `CXXRecordDecl` | `std::bind` 产物本质是函数对象（与有状态 lambda 机制相同），使用同一 class wrapper 策略完全覆盖 |
| 042 | exception_basic | C++ 异常处理 | ✅ | `CXXThrowExpr` / `CXXCatchStmt` | 在 shim 层捕获异常（`try { ... } catch (const std::exception& e) { ... }`）并转为错误码 + 错误消息字符串返回；FFI 边界不抛异常 |

---

### 7.10 其他高级特性（043–048）

| # | 示例 | C++ 特性 | 支持状态 | AST 节点 | 处理方式 |
|---|------|---------|---------|---------|---------|
| 043 | namespace_nested | 嵌套命名空间 | ✅ | `NamespaceDecl` | 命名空间前缀扁平化到 shim 函数名（`foo::bar::Baz::method` → `foo_bar_baz_method`）；`import_lib!` 中不存在命名空间概念 |
| 044 | enum_class | 强类型枚举（enum class） | ✅ | `EnumDecl`（scoped） | 枚举值导出为 Rust `const`（`pub const ERROR_NONE: i32 = 0;`）；Rust 侧建议手动实现 `enum` + `TryFrom<i32>` |
| 045 | union_basic | union（含匿名 union） | ✅ | `RecordDecl`（union） | 生成 opaque pointer + 按字段名的 getter/setter shim；Rust 侧用 `union` 类型或 opaque pointer |
| 046 | constexpr_basic | constexpr 常量/函数 | ✅ | `Expr`（constexpr） | 编译期常量直接读取 AST 中的 `IntegerLiteral` / `FloatingLiteral` 值，生成 Rust `const`；constexpr 函数按普通函数处理（运行期调用） |
| 047 | noexcept_basic | noexcept / noexcept(expr) | ✅ | `NoexceptSpec` | `noexcept` 语义对 FFI 透明；shim 函数本身不抛异常，Rust 侧对应函数不需要特殊处理 |
| 048 | summary | 综合 FFI 模式 | ✅ | — | 综合示例，以上所有策略的组合应用 |

---

### 7.11 ⚠️ 降级特性汇总（4 项）

| TAG | 示例 | C++ 特性 | 不能完全自动的根本原因 | 自动降级策略 | 用户剩余工作 |
|-----|------|---------|---------------------|------------|------------|
| `[OP]` | 019 | 运算符重载 | C ABI 无运算符符号；FFI 边界只能传命名函数 | 每个运算符生成命名 shim（`{class}_{add/sub/...}`），写入 `hicc::cpp!` + `import_lib!` | 可选：手动实现 `impl std::ops::Add<T> for T` 等 Rust trait |
| `[VA]` | 028 | 可变参数模板 | `...Args` 参数包是编译期展开，FFI 运行期无法表达"任意数量参数" | 扫描 AST 中所有调用点，为每种（参数数量 × 类型组合）生成一个固定参数版本 | 若需要新的参数数量组合，手动在 `hicc::cpp!` 和 `import_lib!` 中添加对应版本 |
| `[LM]` | 039 | 有状态 Lambda | 有状态 lambda 是匿名闭包类型，FFI 无法表达捕获列表；无法自动推断 `operator()` 的真实签名 | 无状态 lambda → 函数指针；有状态 lambda → class wrapper + opaque pointer 封装 | 若需要在 Rust 侧传递 Rust 闭包给 C++ 回调，需手动编写 trampoline |
| `[LM]` | 040 | std::function | 同有状态 lambda——类型擦除容器，签名可推断但捕获状态不透明 | 统一使用 class wrapper + opaque pointer 策略 | 可选：手动实现 Rust 闭包 → C++ std::function 适配层 |

---

## 8. 局限性及处理方案

### 8.1 局限性总览

| 限制 | 处理方案 |
|------|---------|
| 仅支持 Linux | Docker 容器方案 |
| 需要完整构建环境 | `--skip-failed` 跳过失败文件 |
| 模板实例化跨 TU | 合并多 AST 分析 |

---

## 9. 实现计划

### 9.1 Phase 顺序

| 阶段 | 内容 | 优先级 | 依赖 |
|------|------|--------|------|
| Phase 0 | Hook 机制（hook.cpp） | P0 | 新建 hook.cpp |
| Phase 1 | ast_parser.rs（C++ AST 解析） | P0 | Phase 0 |
| Phase 2 | 基础提取器（class/function/enum） | P0 | Phase 1 |
| Phase 3 | 模板实例化追踪器 | P0 | Phase 1 |
| Phase 4 | 后处理器（OP/FR/Lambda） | P1 | Phase 2 |
| Phase 5 | hicc 代码生成器 | P0 | Phase 2, 3 |
| Phase 6 | 局限性处理（Docker/增量） | P1 | Phase 1-5 |
| Phase 7 | 集成测试 | P1 | Phase 1-6 |

### 9.2 Phase 0-1 详细任务

**Phase 0 - Hook 机制（新建 `hook.cpp`）**：
1. [ ] 创建 `hook/hook.cpp`，合并以下逻辑：
   - 复制 `hook/hook.c` 的预处理捕获逻辑
   - 复制 `examples/cpp-hook/hook.cpp` 的 C++ 编译器检测逻辑
2. [ ] 修改编译器列表：`gcc, clang, cc` → `g++, clang++, c++`
3. [ ] 修改文件扩展名支持：`.c` → `.cpp, .cc, .cxx, .c++, .C, .cp`
4. [ ] 输出预处理文件（`.c2rust`），使用 `-E -C -P`
5. [ ] 复制 `src/capture.rs`
6. [ ] 复制 `hook/Makefile` 并适配

**Phase 1 - AST 解析**：
1. [ ] 实现 `ast_parser.rs`，使用 `clang` crate
2. [ ] 支持 `CXXRecordDecl`、`CXXMethodDecl` 等 C++ 节点
3. [ ] 支持 `ClassTemplateSpecialization` 模板实例化

**验收标准**：
```bash
# 使用 clang crate 解析宏展开后的 C++ 文件
echo 'class Foo { public: int getValue(); };' | g++ -E -x c++ - > foo.c2rust
cargo run -- parse foo.c2rust
# 应输出:
# - CXXRecordDecl: Foo
#   - CXXMethodDecl: getValue
```

---

## 10. 技术依赖

### 10.1 Rust Crates

```toml
[dependencies]
clang = "2"            # libclang 绑定（C++ AST 解析）
clap = "4"             # CLI
anyhow = "1"            # 错误处理
serde = { version = "1", features = ["derive"] }
serde_json = "1"
walkdir = "2"

[build-dependencies]
cc = "1"
```

### 10.2 系统依赖

```bash
apt-get install clang libclang-dev g++ libstdc++-dev
```

---

## 11. 风险评估

| 风险 | 影响 | 缓解措施 |
|------|------|---------|
| hook.cpp 创建复杂 | 合并两个源码容易出错 | 基于 hook.c 修改，保留核心逻辑 |
| clang crate API 变化 | 解析失败 | 锁定版本 |
| C++ AST 节点遗漏 | 功能缺失 | 扩展解析逻辑 |
| 隐式模板跨 TU | 类型缺失 | 合并多 AST 分析 |
| 系统库展开代码庞大 | 解析慢、文件大 | 接受现状，暂不优化 |
