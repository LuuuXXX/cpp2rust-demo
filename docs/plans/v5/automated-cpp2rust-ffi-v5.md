# C++ 到 Rust Safe FFI 自动化工具 - 方案 v5

## 1. 概述

### 1.1 核心思路

v5 通过 **LD_PRELOAD 编译拦截**机制，在真实编译过程中捕获 C++ 代码信息，生成 Rust FFI 脚手架。

**关键理念**：C++ 模板的价值在于**实例化结果**，而非模板本身。转换时只关注实际被实例化的具体类型（如 `std::vector<int>`），不关注模板声明。

### 1.2 参考实现来源

`references/c2rust-demo/` 目录包含完整的 c2rust-demo 参考项目，其核心文件如下：

| 文件 | 路径 | 用途 |
|------|------|------|
| `hook.c` | `references/c2rust-demo/hook/hook.c` | LD_PRELOAD 拦截逻辑（C 版本，参考结构） |
| `capture.rs` | `references/c2rust-demo/src/capture.rs` | Hook 编译 + LD_PRELOAD 执行逻辑 |
| `main.rs` | `references/c2rust-demo/src/main.rs` | CLI 入口（init / merge 两个命令） |

**cpp2rust-ffi 与 c2rust-demo 的关系**：
- 工具流程完全对应：`init -- <构建命令>` 一步完成 hook 编译、编译拦截、AST 解析、Rust 脚手架生成；`merge` 整理输出结构。
- 需新建 `hook/hook.cpp`，参考 `hook.c` 结构，将编译器列表和文件扩展名改为 C++ 版本（`gcc`/`clang` → `g++`/`clang++`）。
- 需编写 `src/capture.rs`，参考 c2rust-demo 同名文件，将环境变量和目录前缀从 `C2RUST_` / `.c2rust` 改为 `CPP2RUST_` / `.cpp2rust`。

### 1.3 版本定位

v5 是完全独立的新版本，所有输入都必须通过 LD_PRELOAD 编译拦截方式获取。

---

## 2. 快速开始

### 2.1 工作流程

与 c2rust-demo 完全对应，仅有**两个核心命令**：

```bash
# Step 1: 在目标 C++ 项目根目录执行 init
cd cpp-project/
cpp2rust-ffi init -- make -j4

# 示例
cpp2rust-ffi init -- make
cpp2rust-ffi init --feature core_lib -- make -j4
cpp2rust-ffi init -- g++ -c foo.cpp -I.
```

`init` 内部自动完成：
1. 编译 `hook/libhook.so`（若尚未编译）
2. 通过 `LD_PRELOAD` 注入构建过程，捕获 `.cpp2rust` 预处理文件
3. 交互式选择参与转换的文件（非交互环境自动全选）
4. 调用 clang crate 解析 AST，提取类/函数/模板实例化/枚举
5. 生成 `.cpp2rust/<feature>/rust/` 下的 Rust 脚手架及 `init-interface-report.md`

```bash
# Step 2: 合并按符号文件为按模块文件（可选）
cpp2rust-ffi merge
cpp2rust-ffi merge --feature core_lib
```

**`--feature` 使用场景**：大型 C++ 项目通常按模块拆分，`--feature` 允许分批处理，
每次只生成或合并某一模块的 FFI 代码，避免单次生成文件数量过多。省略时默认使用 `default`。

### 2.2 CLI 参数

#### `cpp2rust-ffi init`

| 参数 | 必填 | 说明 |
|------|------|------|
| `--feature <name>` | ❌ | feature 名称；省略时默认 `default` |
| `--skip-failed` | ❌ | 跳过 AST 解析失败的文件，继续处理剩余文件（默认遇到解析错误即终止） |
| `-- <BUILD_CMD>...` | ✅ | `--` 之后的所有参数原样作为构建命令传入 |

#### `cpp2rust-ffi merge`

| 参数 | 必填 | 说明 |
|------|------|------|
| `--feature <name>` | ❌ | 只合并指定 feature；省略则使用 `default` |

### 2.3 环境变量（可选覆盖）

| 变量 | 说明 |
|------|------|
| `CPP2RUST_CXX` | 覆盖默认 C++ 编译器名称（默认自动检测 g++/clang++/c++，支持带版本后缀） |
| `CPP2RUST_DEBUG` | 设置为非空时输出 hook 调试日志到 stderr |

> **与 c2rust-demo 的命名区别**：c2rust-demo 使用 `C2RUST_*` 前缀和 `.c2rust/<feature>` 目录；
> cpp2rust-ffi 有意采用 `CPP2RUST_*` 和 `.cpp2rust/<feature>` 以避免与系统上其他 c2rust 工具产生目录/变量冲突。

---

## 3. 技术架构

### 3.1 整体架构

```
cpp2rust-ffi tool (v5)
├── hook/                         # LD_PRELOAD Hook 库（与 c2rust-demo 保持一致，置于项目根）
│   ├── hook.cpp               # C++ 拦截器（新建）
│   └── Makefile
├── src/
│   ├── main.rs                    # CLI 入口 (init / merge)
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
│    LD_PRELOAD 注入 → 预处理捕获(g++ -E -C) → 编译            │
│    输出: c/*.cpp2rust（宏展开后的 C++ 代码）                  │
└─────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────┐
│ 2. AST 提取 (ast_parser.rs + extractor/)                   │
│    clang crate 解析 .cpp2rust → 类/函数/模板实例化/枚举       │
└─────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────┐
│ 3. 代码生成 (generator/)                                    │
│    hicc 宏格式 Rust 代码 → lib.rs + <unit>.rs               │
│    最小 shim 策略：方法用 import_class!，只为必要场景建 shim │
└─────────────────────────────────────────────────────────────┘
```

---

## 4. Hook 机制

### 4.1 新建 hook.cpp（必须）

| 现有文件 | 问题 | 解决方案 |
|---------|------|---------|
| `hook/hook.c`（c2rust-demo 参考） | 只支持 C 编译器（`gcc`/`clang`/`cc`），只检测 `.c` 文件 | 参考结构新建 hook.cpp，换为 C++ 编译器列表和 C++ 文件扩展名 |

**新 hook.cpp 特性**：

| 特性 | 值 |
|------|---|
| 编译器检测 | `g++`, `clang++`, `c++`（支持 versioned：`g++-13`、`clang++-17`） |
| 文件扩展名 | `.cpp`, `.cc`, `.cxx`, `.c++`, `.C`, `.cp` |
| 预处理参数 | `-E -C`（保留注释；保留行号信息供 clang 定位原始位置） |
| 输出后缀 | `.cpp2rust`（区别于 c2rust 的 `.c2rust`） |

### 4.2 预处理方案（参考 c2rust）

c2rust 使用 `cc -E` 对 `.c` 文件做宏展开，生成 `.i` 文件；cpp2rust 完全对应地使用 C++ 编译器：

```bash
# hook.cpp 中的预处理调用（类比 c2rust 的 cc -E 方案）
g++ -E -C \
    -std=c++17 \          # 从原始编译命令中继承
    -I<inc_paths> \       # 从原始编译命令中继承
    -D<defs> \            # 从原始编译命令中继承
    foo.cpp \
    -o foo.cpp.cpp2rust
```

**与 c2rust 方案的对比**：

| 项目 | c2rust（参考） | cpp2rust（本工具） |
|------|--------------|-----------------|
| 编译器 | `cc` / `gcc` / `clang` | `c++` / `g++` / `clang++` |
| 输入文件 | `.c` | `.cpp` / `.cc` / `.cxx` 等 |
| 输出后缀 | `.c2rust` | `.cpp2rust` |
| 预处理标志 | `-E -C` | `-E -C`（相同；不加 `-P` 以保留行号） |
| 额外 flag | — | `-std=c++17`（从编译命令继承） |

> **注意**：不加 `-P` 是刻意保留行号 marker（`# line "file"`），这样后续 `clang` crate 解析
> `.cpp2rust` 文件时仍能把 AST 节点对应到原始源文件位置，便于调试和报错。

### 4.3 输出目录结构

与 c2rust-demo 的 `.c2rust/<feature>/` 结构完全对应：

```
.cpp2rust/<feature>/                 # CPP2RUST_FEATURE_ROOT 指向此目录
├── c/                               # 预处理文件（.cpp2rust 后缀）
│   └── src/
│       ├── foo.cpp.cpp2rust         # 宏展开后的 C++ 代码
│       └── bar.cpp.cpp2rust
├── meta/                            # 构建元数据
│   ├── build_cmd.txt                # 原始构建命令
│   └── init-interface-report.md    # 初始化报告
├── rust/                            # 生成的 Rust 项目
│   └── src/
│       ├── lib.rs
│       └── mod_<name>/
└── targets.list                     # 链接目标列表
```

**使用示例**（feature 名为 `core_lib`）：
```bash
# init 时通过 --feature 指定；工具内部自动管理 .cpp2rust/ 目录
cpp2rust-ffi init --feature core_lib -- make -j4
# 产物在 .cpp2rust/core_lib/c/src/*.cpp2rust

# 省略 --feature 时默认使用 default
cpp2rust-ffi init -- make -j4
# 产物在 .cpp2rust/default/c/src/*.cpp2rust
```

---

## 5. AST 解析

### 5.1 clang crate 解析

```rust
// ast_parser.rs（输入文件为 .cpp2rust，即 g++ -E -C 输出）
use clang::Clang;

pub fn parse_preprocessed(file: &Path) -> Result<CppAst> {
    let clang = Clang::new()?;
    let index = Index::new(&clang, false, false);

    // 必须传 -x c++：.cpp2rust 扩展名非标准，clang 默认按 C 解析
    let tu = index.parser(file)
        .arguments(&["-xc++", "-std=c++17"])
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

**支持（参与 FFI 生成）**：

| 节点类型 | v5 用途 |
|---------|--------|
| `CXXRecordDecl` | 类/结构体定义 |
| `CXXMethodDecl` | 成员函数 |
| `FunctionDecl` | 全局函数 |
| `CXXConstructorDecl` | 构造函数 |
| `CXXDestructorDecl` | 析构函数 |
| `ClassTemplateSpecialization` | 模板实例化（只处理具体类型） |
| `ClassTemplatePartialSpecialization` | 偏特化路径（同上，只收集实例化结果） |
| `NamespaceDecl` | 命名空间（前缀扁平化到符号名） |
| `EnumDecl` | 枚举定义 |
| `FieldDecl` | 成员字段（用于生成 getter/setter shim） |
| `VarDecl`（static） | 静态成员变量（生成 getter/setter shim） |
| `ParmVarDecl` | 函数参数（含默认值信息） |
| `CXXBaseSpecifier` | 继承关系（用于方法提升） |
| `FriendDecl` | 友元声明（友元函数提取为普通函数） |

**不支持（不参与 FFI 生成）**：

| 节点类型 | 不支持原因 |
|---------|----------|
| `IfStmt` / `ForStmt` / `WhileStmt` / `SwitchStmt` / `DoStmt` | 控制流语句，属于函数体实现细节，对 FFI 签名无帮助 |
| `CompoundStmt` / `ReturnStmt` / `BreakStmt` / `ContinueStmt` | 语句块与跳转，同上 |
| `BinaryOperator` / `UnaryOperator` / `ConditionalOperator` | 表达式节点，属于函数体实现，不影响 FFI 边界 |
| `CallExpr` / `MemberExpr` / `DeclRefExpr` | 函数调用与成员访问，属于函数体实现 |
| `CXXThisExpr` | `this` 指针表达式，函数体内部细节，FFI 边界不需要 |
| `AccessSpecDecl` | `public` / `private` / `protected` 访问控制关键字，FFI 只暴露 public 接口，访问控制在 AST 过滤阶段已处理 |
| `UsingDirectiveDecl` | `using namespace`，只影响名称查找，不产生符号 |
| `UsingDecl` | `using Base::method`，基类方法的引入通过 `CXXBaseSpecifier` 展开处理，此节点无需单独处理 |
| `StaticAssertDecl` | 编译期断言，无运行时行为，对 FFI 无意义 |
| `ClassTemplate` / `FunctionTemplate`（纯模板声明） | 模板声明本身不产生符号；只处理实际被实例化的 `ClassTemplateSpecialization` |
| `TypedefDecl` / `TypeAliasDecl` | 类型别名本身不产生符号，只在涉及 public API 参数类型时间接处理，不单独遍历 |
| `CXXBindTemporaryExpr` / `MaterializeTemporaryExpr` | 临时对象生命周期管理，C++ 编译器内部节点，跨 FFI 边界无意义 |
| `NullStmt` / `LabelStmt` / `GotoStmt` | 空语句与跳转标签，函数体细节，对 FFI 无意义 |

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

### 6.3 hicc::cpp! 代码重建策略（分离定义处理）

C++ 源码中类方法常以**分离定义**形式出现（`.h` 声明 + `.cpp` 实现），而 `hicc::cpp!` 块要求方法**全部内联写入 class 体内**。生成器处理流程如下：

1. **提取声明**：从 AST 的 `CXXRecordDecl` 获取 class 骨架和方法签名列表
2. **提取实现**：从 AST 的 `CXXMethodDecl` 节点的 `get_definition()` 找到方法体，读取对应 SourceRange 内的原始文本（`.cpp2rust` 文件中的行范围）
3. **重组内联**：将提取的方法体文本嵌入 class 体内；若方法体在系统头中（`is_in_system_header()` 为 true）则不提取实现，只保留声明
4. **shim 函数**：ctor/dtor 等必要 shim 函数直接追加在 class 定义之后，写入同一 `hicc::cpp!` 块

> **注意**：若源码已是头文件内联定义（方法体直接在 `.h` 中），则 SourceRange 已包含完整实现，无需跨文件拼接。

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
| 001 | hello_world | extern "C" 函数 | ✅ | `FunctionDecl` | AST 直接提取 → `import_lib!` 条目，无 shim |
| 002 | function_overload | 函数重载 | ✅ | `FunctionDecl`（多个同名） | 每个重载签名不同，AST 中各重载独立可见；各重载名称加后缀区分（如 `_i32`, `_f64`），生成 `import_lib!` 条目 |
| 003 | default_args | 默认参数 | ✅ | `ParmVarDecl`（含默认值） | 默认参数在 C++ 侧展开为多个固定参数重载函数写入 `hicc::cpp!`；`import_lib!` 声明各版本 |
| 004 | inline_functions | inline 函数 | ✅ | `FunctionDecl` + `inline` | 函数体从 `.cpp2rust` 中提取，内联写入 `hicc::cpp!`；`import_lib!` 照常声明 |
| 005 | variadic_functions | C 可变参数函数（va_list） | ✅ | `FunctionDecl`（va_list） | C 可变参数（`...`）无法直接跨 FFI 绑定；在 C++ 侧已存在固定参数 wrapper（如 `sum_3`/`sum_5`），工具检测到这些 wrapper 后直接用裸 `extern "C"` 绑定，**不**通过 hicc 宏；**不**生成 `hicc::cpp!` 块（函数体不内联，直接链接现有符号） |

---

### 7.3 类与对象（006–012）

| # | 示例 | C++ 特性 | 支持状态 | AST 节点 | 处理方式 |
|---|------|---------|---------|---------|---------|
| 006 | class_basic | 基础类 | ✅ | `CXXRecordDecl` | opaque pointer 模式：`class Foo;` 前向声明 + `import_class!`（方法直接绑定）+ `import_lib!`（仅 ctor/dtor shim） |
| 007 | class_constructor | 构造/析构 | ✅ | `CXXConstructorDecl` / `CXXDestructorDecl` | **必要 shim**：生成 `*_new()` 和 `*_delete()` 包装函数（C 没有 `new`/`delete`） |
| 008 | class_copy | 拷贝构造 | ✅ | `CXXConstructorDecl`（copy） | **必要 shim**：生成 `*_copy()` shim，接收 `const Foo*`，返回新 `Foo*` |
| 009 | class_move | 移动构造 | ✅ | `CXXConstructorDecl`（move） | **必要 shim**：生成 `*_move()` shim，使用 `std::move()`；原指针逻辑上转移所有权 |
| 010 | class_static | 静态成员 | ✅ | `VarDecl`（static） | **必要 shim**：生成 getter/setter shim（`{class}_get_{field}` / `{class}_set_{field}`） |
| 011 | class_const | const 成员函数 | ✅ | `CXXMethodDecl`（const） | 直接 `import_class!`，无 shim；Rust 侧映射为 `fn method(&self)`（不可变引用） |
| 012 | class_volatile | volatile 成员函数 | ✅ | `CXXMethodDecl`（volatile） | 直接 `import_class!`，无 shim；`method` 属性中注明 `volatile`，Rust 侧使用 `*mut Foo` 并附注释 |

---

### 7.4 面向对象特性（013–018）

| # | 示例 | C++ 特性 | 支持状态 | AST 节点 | 处理方式 |
|---|------|---------|---------|---------|---------|
| 013 | inheritance_single | 单继承 | ✅ | `CXXBaseSpecifier` | 基类方法在子类的 `import_class!` 中一并列出（方法提升），无 shim |
| 014 | inheritance_multiple | 多继承 | ✅ | `CXXBaseSpecifier`（多个） | 多条继承链展开，同名方法按 C++ 查找规则决定选用哪条链；所有方法通过 `import_class!` 直接绑定，无 shim |
| 015 | virtual_basic | 虚函数 | ✅ | `CXXMethodDecl`（virtual） | 通过 opaque pointer 调用虚函数；直接 `import_class!`，hicc 宏负责虚表 dispatch，无 shim |
| 016 | virtual_pure | 纯虚/抽象类 | ✅ | `CXXMethodDecl`（= 0） | 抽象类只生成 `class Foo;` 前向声明；子类方法通过 `import_class!` 直接绑定，无 shim |
| 017 | virtual_override | override | ✅ | `CXXMethodDecl`（override） | override 语义透传；Rust 侧调用方式与普通虚函数相同，直接 `import_class!` |
| 018 | virtual_diamond | 菱形继承（virtual 继承） | ✅ | `CXXBaseSpecifier`（virtual） | 共享基类通过 opaque pointer 统一访问；**不**生成 `as_base()` 类型转换 shim；而是为每个需要从 Rust 侧调用的继承方法生成独立命名 shim（如 `d_getAValue(D*)`），在 shim 内部通过派生类指针直接调用，无需指针调整 |

---

### 7.5 运算符与类型（019–023）

| # | 示例 | C++ 特性 | 支持状态 | AST 节点 | 原因 & 处理方式 |
|---|------|---------|---------|---------|----------------|
| 019 | operator_overload | 运算符重载 | ⚠️ `[OP]` | `CXXMethodDecl`（operator） | **不能直接 ✅ 的原因**：C ABI 不支持 `operator+` 等符号名，FFI 边界只能传命名函数。**处理方案**：为每个运算符生成命名 shim（如 `number_add` / `number_sub`），写入 `hicc::cpp!` 和 `import_lib!`。追加内联 TODO：`// cpp2rust-todo[OP]: 可实现 impl std::ops::Add<Number> for Number` |
| 020 | friend_function | 友元函数 | ✅ | `FriendDecl` | 友元关系对 FFI 透明；友元函数在 AST 中可直接提取为普通函数写入 `import_lib!`，附注释说明友元身份，无 shim |
| 021 | explicit_ctor | explicit 构造函数 | ✅ | `CXXConstructorDecl`（explicit） | `explicit` 防止隐式转换，对 FFI 透明；生成 `*_new()` shim 与普通构造函数相同（必要 shim） |
| 022 | mutable_member | mutable 成员 | ✅ | `FieldDecl`（mutable） | `mutable` 允许 const 方法修改，对 FFI 透明；直接 `import_class!`，Rust 侧对应方法仍声明为 `&self`，无 shim |
| 023 | typeid_rtti | typeid / RTTI / dynamic_cast | ✅ | `CXXTypeidExpr` / `CXXDynamicCastExpr` | 检测到 RTTI 使用场景时，在 `hicc::cpp!` 中注入整数枚举（`ShapeType { CIRCLE=0, ... }`）+ 虚函数（`getType()` / `getTypeName()`），完全绕过 `typeid`；Rust 侧通过 `import_class!` 直接调用 `getType()`，无需额外 shim |

---

### 7.6 模板实例化（024–028）

| # | 示例 | C++ 特性 | 支持状态 | AST 节点 | 原因 & 处理方式 |
|---|------|---------|---------|---------|----------------|
| 024 | template_function | 函数模板 | ✅ | `FunctionTemplateDecl` + `ClassTemplateSpecialization` | 忽略模板声明；在 `hicc::cpp!` 中为每个实例化版本生成命名 C 包装函数（如 `swap_int(int*,int*)`），调用对应模板实例（`do_swap<int>`）；再在 `import_lib!` 中绑定这些包装函数。**包装函数本身属于必要 shim**（模板实例化的 C 兼容导出层） |
| 025 | template_class | 类模板 | ✅ | `ClassTemplateSpecialization` | 忽略模板声明；只处理实际实例化的具体类型（如 `Stack<int>`），按普通类生成 ctor/dtor shim + `import_class!` 方法绑定 |
| 026 | template_specialization | 模板偏特化 | ✅ | `ClassTemplatePartialSpecialization` | 偏特化本身视为实例化路径之一；解析时只收集通过该特化路径实例化的类型 |
| 027 | template_instantiation | 显式模板实例化 | ✅ | `ClassTemplateSpecialization` | 显式实例化（`template class Foo<int>;`）直接在 AST 中可见 |
| 028 | variadic_template | 可变参数模板 | ⚠️ `[VA]` | `VariadicTemplate` / `CallExpr` | **不能直接 ✅ 的原因**：C++ 可变参数模板（`...Args`）是编译期展开，FFI 无法表达"任意数量参数"。**处理方案**：在 `hicc::cpp!` 中生成 wrapper 类（如 `SumCalculator`），将每种参数数量版本封装为独立静态方法（`calculate_1`/`calculate_2` 等），再生成 C 兼容包装函数（`sum_1`/`sum_2` 等）；`import_lib!` 绑定各包装函数。追加内联 TODO：`// cpp2rust-todo[VA]: 可变参数模板，已按调用点展开 N 个版本，新增参数组合时需手动添加对应版本` |

---

### 7.7 智能指针与内存（029–033）

| # | 示例 | C++ 特性 | 支持状态 | AST 节点 | 处理方式 |
|---|------|---------|---------|---------|---------|
| 029 | unique_ptr | std::unique_ptr | ✅ | `ClassTemplateSpecialization` | opaque pointer；`*_new()` 返回裸指针（必要 shim），调用方通过 `*_delete()` 释放；不尝试映射所有权语义 |
| 030 | shared_ptr | std::shared_ptr | ✅ | `ClassTemplateSpecialization` | 类似 unique_ptr；`*_clone()` shim 增加引用计数，`*_delete()` shim 减少；其他方法通过 `import_class!` 直接绑定 |
| 031 | custom_deleter | 自定义删除器 | ✅ | `FunctionDecl` | 删除器函数在 AST 中可见；注入 `hicc::cpp!`；`*_delete()` shim 内部调用自定义删除器 |
| 032 | placement_new | Placement new | ✅ | `CXXNewExpr` | 生成 `*_placement_new(ptr, ...)` shim（必要，因为 placement new 不是普通函数调用） |
| 033 | raii_pattern | RAII 模式 | ✅ | 构造/析构 | 析构函数生成 `*_delete()` shim（必要）；Rust 侧可实现 `Drop` trait 调用 `*_delete()` |

---

### 7.8 STL 容器（034–038）

> **核心策略**：STL 容器的模板方法签名复杂，不能直接通过 `import_class!` 绑定 `std::vector<T>` 等。
> 必须先在 `hicc::cpp!` 中生成**薄 wrapper 类**（如 `IntVector` 封装 `std::vector<int>`），
> 再对 wrapper 类做 `import_class!` 绑定，并为 ctor/dtor 生成必要 shim。

| # | 示例 | C++ 特性 | 支持状态 | AST 节点 | 处理方式 |
|---|------|---------|---------|---------|---------|
| 034 | vector_basic | std::vector\<T\> | ✅ | `ClassTemplateSpecialization` | **生成 wrapper 类**（如 `IntVector` 封装 `VectorImpl<int>`）；`vec_new`/`vec_delete` 是必要 shim（ctor/dtor）；`push_back`/`get`/`size` 等通过 `import_class!` 绑定 wrapper 类方法 |
| 035 | map_basic | std::map\<K,V\> | ✅ | `ClassTemplateSpecialization` | **生成 wrapper 类**（如 `StringIntMap` 封装 `MapImpl<string,int>`）；`map_new`/`map_delete` 是必要 shim；`insert`/`find`/`erase` 等通过 `import_class!` 绑定 wrapper 类方法 |
| 036 | string_basic | std::string | ✅ | `ClassTemplateSpecialization` | **生成 wrapper 类**封装 `std::string`；`str_new`/`str_delete` 是必要 shim；`c_str()`/`length()` 等通过 `import_class!` 绑定 wrapper 类方法；跨 FFI 传字符串用 `*const i8` |
| 037 | array_basic | std::array\<T,N\> | ✅ | `ClassTemplateSpecialization` | **生成 wrapper 类**封装 `std::array<T,N>`；ctor/dtor 是必要 shim；按索引访问通过 `import_class!` 绑定 wrapper 类方法；N 在实例化时已知 |
| 038 | tuple_basic | std::tuple\<T...\> | ✅ | `ClassTemplateSpecialization` | **生成 wrapper 类**封装 `std::tuple<T...>`；ctor/dtor 是必要 shim；按位置 `get<N>()` 通过 `import_class!` 绑定；元素类型在实例化时已知 |

---

### 7.9 函数对象（039–042）

| # | 示例 | C++ 特性 | 支持状态 | AST 节点 | 原因 & 处理方式 |
|---|------|---------|---------|---------|----------------|
| 039 | lambda_basic | Lambda 表达式 | ⚠️ `[LM]` | `LambdaExpr` | **不能直接 ✅ 的原因**：有状态 lambda（捕获外部变量）是匿名类型，FFI 无法直接表达捕获列表。**处理方案（双策略，遵循 hicc 用法）**：① 无状态 lambda（空捕获 `[]`）→ 退化为函数指针，直接在 `import_lib!` 中声明为函数指针类型，无 shim。② 有状态 lambda（含捕获）→ `hicc::cpp!` 中生成 class wrapper（`LambdaWrapper`），通过 ctor/call/dtor shim 暴露；Rust 侧通过 `import_class!` 调用 `call()` 方法。追加内联 TODO：`// cpp2rust-todo[LM]: 有状态 lambda，内部捕获状态不透明，已封装为 class wrapper` |
| 040 | std_function | std::function\<Sig\> | ⚠️ `[LM]` | `ClassTemplateSpecialization` | **同有状态 lambda**——类型擦除容器，签名可推断但捕获状态不透明；统一使用 class wrapper + opaque pointer 策略；通过 `import_class!` 调用 `call()` 方法；追加内联 TODO：`// cpp2rust-todo[LM]: std::function，已封装为 class wrapper` |
| 041 | functional_bind | std::bind | ✅ | `CallExpr`（bind）/ `CXXRecordDecl` | `std::bind` 产物本质是函数对象（与有状态 lambda 机制相同），使用同一 class wrapper 策略，通过 `import_class!` 完全覆盖 |
| 042 | exception_basic | C++ 异常处理 | ✅ | `CXXThrowExpr` / `CXXCatchStmt` | 在 `hicc::cpp!` shim 层捕获异常（`try { ... } catch (const std::exception& e) { ... }`），转为错误码 + 错误消息字符串返回；FFI 边界不跨越异常；Rust 侧通过 `import_lib!` 调用 shim |

---

### 7.10 其他高级特性（043–048）

| # | 示例 | C++ 特性 | 支持状态 | AST 节点 | 处理方式 |
|---|------|---------|---------|---------|---------|
| 043 | namespace_nested | 嵌套命名空间 | ✅ | `NamespaceDecl` | 命名空间前缀扁平化到函数名（`foo::bar::Baz::method` → `foo_bar_baz_method`）；方法通过 `import_class!` 直接绑定，命名空间在 `import_lib!` 中不存在 |
| 044 | enum_class | 强类型枚举（enum class） | ✅ | `EnumDecl`（scoped） | 枚举值导出为 Rust `const`（`pub const ERROR_NONE: i32 = 0;`）；Rust 侧建议手动实现 `enum` + `TryFrom<i32>` |
| 045 | union_basic | union（含匿名 union） | ✅ | `RecordDecl`（union） | opaque pointer + 按字段名的 getter/setter shim（必要，因 union 字段无独立符号）；Rust 侧用 opaque pointer |
| 046 | constexpr_basic | constexpr 常量/函数 | ✅ | `Expr`（constexpr） | 编译期常量直接读取 AST 中的 `IntegerLiteral` / `FloatingLiteral` 值，生成 Rust `const`；constexpr 函数按普通函数处理，AST 直接提取 |
| 047 | noexcept_basic | noexcept / noexcept(expr) | ✅ | `NoexceptSpec` | `noexcept` 语义对 FFI 透明；hicc 宏生成的函数本身不抛异常，Rust 侧不需要特殊处理 |
| 048 | summary | 综合 FFI 模式 | ✅ | — | 综合示例，以上所有策略的组合应用 |

---

### 7.11 ⚠️ 降级特性汇总（4 项）

| TAG | 示例 | C++ 特性 | 不能完全自动的根本原因 | 自动降级策略 | 用户剩余工作 |
|-----|------|---------|---------------------|------------|------------|
| `[OP]` | 019 | 运算符重载 | C ABI 无运算符符号；FFI 边界只能传命名函数 | 每个运算符生成命名 shim（`{class}_{add/sub/...}`），写入 `hicc::cpp!` + `import_lib!` | 可选：手动实现 `impl std::ops::Add<T> for T` 等 Rust trait |
| `[VA]` | 028 | 可变参数模板 | `...Args` 参数包是编译期展开，FFI 运行期无法表达"任意数量参数" | 在 `hicc::cpp!` 中生成 wrapper 类（如 `SumCalculator`），按参数数量和类型组合分别封装为静态方法，再生成 C 兼容命名包装函数（`sum_1`/`sum_2` 等）；`import_lib!` 绑定各包装函数 | 若需要新的参数数量或类型组合，手动在 `hicc::cpp!` 中为 wrapper 类添加对应静态方法和包装函数，并在 `import_lib!` 中声明 |
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

### 9.1 开发策略：测试驱动

> **原则**：先搭建测试基础设施，再开发功能。每个 Phase 的完成标准是"相关测试全部通过"，而非"代码可编译"。

```
Phase T  →  Phase 0  →  Phase 1  →  Phase 2 & 3  →  Phase 4  →  Phase 5  →  Phase 6
(测试框架)  (hook.cpp)  (AST 解析)   (提取器)       (后处理)    (代码生成)   (局限性处理)
    ↑_______________每个 Phase 开发完立即运行测试，不通过不进入下一 Phase_______________↑
```

---

### 9.2 Phase 顺序

| 阶段 | 内容 | 优先级 | 依赖 |
|------|------|--------|------|
| **Phase T** | **测试基础设施（黄金文件 + 编译 + 运行）** | **P0（最先）** | 无 |
| Phase 0 | Hook 机制（hook.cpp，输出 `.cpp2rust`；`init` 命令集成 hook 编译） | P0 | Phase T |
| Phase 1 | ast_parser.rs（C++ AST 解析 `.cpp2rust`） | P0 | Phase 0 |
| Phase 2 | 基础提取器（class/function/enum） | P0 | Phase 1 |
| Phase 3 | 模板实例化追踪器 | P0 | Phase 1 |
| Phase 4 | 后处理器（OP/FR/Lambda） | P1 | Phase 2 |
| Phase 5 | hicc 代码生成器（最小 shim 策略） | P0 | Phase 2, 3 |
| Phase 6 | merge 命令 + 局限性处理（增量/--feature） | P1 | Phase 1–5 |

---

### 9.3 Phase T - 测试基础设施（最高优先级）

#### 9.3.1 设计原则

每个 `examples/NNN_*/` 目录已包含**完整的参考资料**：
- `cpp/` → 工具的**输入**（C++ 源码）
- `rust_hicc/src/main.rs` → 包含 FFI 脚手架 + 手写 `fn main()` 的**可运行参考文件**
- `rust_hicc/build.rs` + `Cargo.toml` → 编译/运行验证所需的构建配置

> **重要区分**：工具只能生成 FFI 脚手架（`hicc::cpp!` / `hicc::import_class!` / `hicc::import_lib!` 三段），**不能也不应**生成 `fn main()`。`main.rs` 中的 `fn main()` 是手写的演示逻辑，**不属于工具输出范围**。
>
> 因此 L1 黄金文件测试**不比对 `main.rs` 全文**，而是只比对其中的 FFI 脚手架段落（`hicc::cpp!`、`hicc::import_class!`、`hicc::import_lib!` 块）。工具实际输出的目标文件为 `lib.rs`（无 `fn main()`）。

测试框架分三层，逐层递进：

| 层次 | 名称 | 验证内容 | 触发时机 |
|------|------|---------|---------|
| L1 | **黄金文件测试** | 工具生成的 FFI 脚手架段落与 `rust_hicc/src/main.rs` 中对应段落一致 | 每次提交 |
| L2 | **编译测试** | 仓库中现有的 `rust_hicc/` 能通过 `cargo build`（基线验证） | 每次提交 |
| L3 | **运行测试** | `cargo run` 输出与 README 中"运行结果"一致 | 合并前 |

#### 9.3.2 测试目录结构

```
cpp2rust-ffi/
├── tests/
│   ├── common/
│   │   ├── mod.rs              # 共用工具：run_tool(), diff_golden(), cargo_build()
│   │   └── golden.rs           # 黄金文件读取 & 规范化（去空行/注释比对）
│   ├── l1_golden_tests.rs      # L1：001–048 黄金文件对比
│   ├── l2_compile_tests.rs     # L2：001–048 cargo build 验证
│   └── l3_run_tests.rs         # L3：001–048 cargo run + stdout 对比
└── Cargo.toml
```

#### 9.3.3 L1 黄金文件测试（最核心）

**核心原则**：从 `rust_hicc/src/main.rs` 中提取 `hicc::cpp!`、`hicc::import_class!`、`hicc::import_lib!` 三种块作为黄金片段，与工具生成的 `lib.rs` 对应块进行比对，忽略 `fn main()` 和注释差异。

```rust
// tests/l1_golden_tests.rs
// 工具生成的 lib.rs 中的 FFI 脚手架段落，与 rust_hicc/src/main.rs 中同类段落对比

macro_rules! golden_test {
    ($name:ident, $example:literal) => {
        #[test]
        fn $name() {
            let example_dir = concat!("../examples/", $example);
            // 1. 用工具生成 lib.rs（只含 hicc 三段，无 fn main）
            let generated = run_tool_on(example_dir);
            // 2. 从黄金文件中提取 hicc 块（跳过 fn main 及其他手写代码）
            let golden_raw = read_golden(example_dir, "rust_hicc/src/main.rs");
            let golden = extract_hicc_blocks(&golden_raw);
            // 3. 规范化后对比（忽略空白行和注释差异）
            assert_eq!(normalize(&generated), normalize(&golden),
                "FFI scaffold mismatch for {}", $example);
        }
    };
}

/// 从文件内容中提取所有 hicc::cpp! / hicc::import_class! / hicc::import_lib! 块
fn extract_hicc_blocks(src: &str) -> String {
    // 按行扫描，提取 hicc:: 开头的块直到对应的闭合 }
    // 跳过 fn main() { ... } 及其他非 hicc 代码
    todo!("实现块提取逻辑")
}

golden_test!(test_001_hello_world,            "001_hello_world");
golden_test!(test_002_function_overload,      "002_function_overload");
golden_test!(test_003_default_args,           "003_default_args");
// ... 003–048 全部列出
golden_test!(test_048_summary,               "048_summary");
```

**初始状态**：工具尚未实现时，所有测试均为 **FAIL（预期行为）**。每完成一个 Phase，相关测试变为 PASS。

#### 9.3.4 L2 编译测试

```rust
// tests/l2_compile_tests.rs
// 直接对仓库中现有的 rust_hicc/ 目录运行 cargo build，
// 验证黄金文件本身可编译（保证基线正确）

#[test]
fn compile_001_hello_world() {
    let status = Command::new("cargo")
        .args(["build"])
        .current_dir("../examples/001_hello_world/rust_hicc")
        .status().unwrap();
    assert!(status.success());
}
// ... 001–048
```

> **注意**：L2 测试在 Phase T 完成时就应全部通过（黄金文件本身必须可编译）。

#### 9.3.5 L3 运行测试

```rust
// tests/l3_run_tests.rs
// cargo run 后比对 stdout 与 README 中的"运行结果"章节

fn expected_output(example: &str) -> String {
    // 从 README.md 中提取 "## 运行结果" 代码块
    parse_readme_run_result(&format!("../examples/{}/README.md", example))
}

#[test]
fn run_001_hello_world() {
    let output = cargo_run("../examples/001_hello_world/rust_hicc");
    assert_eq!(output.trim(), expected_output("001_hello_world").trim());
}
```

#### 9.3.6 测试执行命令

```bash
# 运行全部测试
cargo test

# 只运行 L1 黄金文件测试
cargo test --test l1_golden_tests

# 只运行某个示例的全部测试
cargo test 006_class_basic

# 查看当前通过率（CI 友好）
cargo test 2>&1 | grep -E "^test.*FAILED|^test result"
```

#### 9.3.7 Phase T 完成标准

| 检查项 | 验收条件 |
|--------|---------|
| hicc crate 可用 | `hicc = "0.2"` 和 `hicc-build = "0.2"` 可从 crates.io 安装（**前置条件**：若未发布，需在工具项目中通过 `[patch.crates-io]` 或 path 依赖先解决此问题再进行 L2/L3） |
| L2 编译基线确立 | 至少 001–010 示例 `cargo build` 成功；其余已知可编译示例纳入基线，剩余可在后续 Phase 中补充 |
| L3 运行基线确立 | 至少 001–010 示例 `cargo run` 输出与 README 一致 |
| L1 测试框架就绪 | 框架代码完整（含 `extract_hicc_blocks()` 实现），48 个 L1 测试均可运行（初始均 FAIL，符合预期） |
| CI 集成 | `cargo test --test l1_golden_tests` 在 GitHub Actions 中运行 |

> **说明**：Phase T 不要求全部 48 个示例一次性全部通过 L2/L3，避免单点阻塞。以已知可编译的示例作为基线，后续 Phase 中每完成一批功能，相应示例的 L2/L3 顺带验证通过。

---

### 9.4 Phase 0 - Hook 机制（新建 `hook.cpp`）

**任务**：
1. [ ] 参考 `references/c2rust-demo/hook/hook.c`，创建 `hook/hook.cpp`
2. [ ] 修改编译器列表：`gcc, clang, cc` → `g++, clang++, c++`（支持 versioned 如 `g++-13`）
3. [ ] 修改文件扩展名支持：`.c` → `.cpp, .cc, .cxx, .c++, .C, .cp`
4. [ ] 输出预处理文件（`.cpp2rust`），使用 `-E -C`（保留注释和行号 marker）
5. [ ] 参考 `references/c2rust-demo/src/capture.rs`，编写 `src/capture.rs` 并做以下适配：
   - 将环境变量 `C2RUST_*` 前缀改为 `CPP2RUST_*`
   - 将目录 `.c2rust` 改为 `.cpp2rust`
   - Hook 构建集成到 `init` 命令流程中（`init` 调用时自动编译 libhook.so）
6. [ ] 创建 `hook/Makefile`

**Phase 0 完成标准**：
```bash
# 在 001_hello_world 上运行 init，验证 .cpp2rust 文件产出
cd examples/001_hello_world/cpp
cpp2rust-ffi init -- make
# 应在 .cpp2rust/default/c/ 下产出 hello_world.cpp.cpp2rust
```

---

### 9.5 Phase 1 - AST 解析

**任务**：
1. [ ] 实现 `ast_parser.rs`，使用 `clang` crate
2. [ ] 输入文件为 `.cpp2rust`（`g++ -E -C` 产物）
3. [ ] 支持 `CXXRecordDecl`、`CXXMethodDecl` 等 C++ 节点
4. [ ] 支持 `ClassTemplateSpecialization` 模板实例化
5. [ ] **实现系统头过滤**：使用 `cursor.location().is_in_system_header()` 跳过系统头展开的节点，只保留用户代码节点（`g++ -E -C` 处理含 `#include <vector>` 的文件会产生数万行系统头代码，不过滤将导致大量无关 `std::allocator` 等模板特化被误提取）

**Phase 1 完成标准**：
```bash
# 解析宏展开后的 C++ 文件，应输出 AST 结构
echo 'class Foo { public: int getValue(); };' | g++ -E -C -x c++ - > foo.cpp2rust
cargo run -- parse foo.cpp2rust
# 应输出:
# - CXXRecordDecl: Foo
#   - CXXMethodDecl: getValue
```

---

### 9.6 Phase 进度追踪（L1 测试通过率）

> 每完成一个 Phase，对应的 L1 测试应从 FAIL → PASS。

| L1 测试范围 | 对应 Phase |
|------------|-----------|
| 001–005（基础函数） | Phase 2 基础提取器 + Phase 5 代码生成 |
| 006–012（类与对象） | Phase 2 + Phase 5 |
| 013–018（OOP 特性） | Phase 2 + Phase 5 |
| 019–023（运算符与类型） | Phase 4 后处理 + Phase 5 |
| 024–028（模板实例化） | Phase 3 实例化追踪器 + Phase 5 |
| 029–033（智能指针） | Phase 2 + Phase 5 |
| 034–038（STL 容器） | Phase 3 + Phase 5 |
| 039–042（函数对象） | Phase 4 + Phase 5 |
| 043–048（高级特性） | Phase 2 + Phase 5 |

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

> **关于 hicc crate**：`examples/*/rust_hicc/` 中的黄金文件依赖 `hicc = "0.2"` 和 `hicc-build = "0.2"`（运行时/构建时宏库）。这两个 crate **属于目标用户项目的依赖**，不是本工具本身的依赖——本工具只负责生成引用 hicc 的 Rust 代码，自身不引入 hicc。进入 Phase T 前需确认 `hicc = "0.2"` 已发布到 crates.io 或通过 workspace 路径依赖可用，否则 L2/L3 测试无法编译。

### 10.2 系统依赖

```bash
apt-get install clang libclang-dev g++ libstdc++-dev
```

---

## 11. 风险评估

| 风险 | 影响 | 缓解措施 |
|------|------|---------|
| hook.cpp 创建复杂 | C++ 编译器检测容易出错 | 参考 c2rust-demo 的 hook.c 结构，改编 C++ 编译器检测部分 |
| clang crate API 变化 | 解析失败 | 锁定版本 |
| C++ AST 节点遗漏 | 功能缺失 | 扩展解析逻辑；参考示例黄金文件补充缺失的节点处理 |
| 隐式模板跨 TU | 类型缺失 | 合并多 AST 分析 |
| 系统库展开代码庞大 | 解析慢、文件大 | 接受现状；过滤系统头路径 |
| --feature 范围界定 | feature 边界模糊 | feature 对应 `.cpp2rust/<name>/` 目录，由用户决定分组 |
| 分离定义方法体提取 | hicc::cpp! 块内容不完整或报错 | 通过 `CXXMethodDecl::get_definition()` + SourceRange 读取原始文本；头文件内联定义无此问题 |
| hicc crate 不可用 | Phase T L2/L3 编译失败 | Phase T 前先确认 hicc/hicc-build 可从 crates.io 安装；必要时通过 workspace path 依赖提供 |
