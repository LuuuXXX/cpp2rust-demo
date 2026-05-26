# C++ 到 Rust Safe FFI 自动化工具 - 方案 v5

## 1. 概述

### 1.1 核心思路：LD_PRELOAD 编译拦截 + 编译时实例化捕获

v5 是完全重新设计的版本，借鉴 `c2rust-demo` 的构建捕获机制，通过 `LD_PRELOAD` 注入共享库拦截编译器进程，在**真实编译过程中**捕获转换所需的所有信息。

**核心理念**：C++ 模板的价值在于**实例化结果**，而非模板本身。转换过程中：
- 不需要分析模板声明
- 只需要捕获**实际被实例化的类型和符号**
- 生成 Rust FFI 时，直接使用实例化后的具体类型（如 `std::vector<int>` 而非 `std::vector<T>`）

### 1.2 编译拦截流程

```
真实编译命令: g++ -c foo.cpp -o foo.o
                   ↓
         LD_PRELOAD=libhook.so
                   ↓
     hook.so 拦截 g++ 调用
                   ↓
    1. 执行预处理，捕获展开后的代码
    2. 解析 AST，提取实例化信息
    3. 记录符号信息
                   ↓
        生成 .cpp2rust 预处理文件和元数据
```

### 1.3 版本演进

| 版本 | 核心突破 | 局限性 |
|------|---------|--------|
| v1 | 头文件解析 → 自动生成 hicc 脚手架 | 模板实例化无法处理 |
| v2 | AST 编译捕获 → 解决模板实例化声明 | 运算符/友元/lambda/RTTI 无法生成 |
| v3 | 后处理降级 → 5 类特性生成可编译代码 + TODO 清单 | 代码格式不对齐、TODO 系统过重 |
| v4 | 修正代码格式对齐 hicc 生态、改进 Lambda/RTTI 策略 | 静态分析无法捕获运行时信息 |
| **v5** | **LD_PRELOAD 编译拦截，捕获真实编译过程** | 仅支持 Linux |

---

## 2. 技术方案

### 2.1 整体架构

```
cpp2rust-ffi tool (v5)
├── src/
│   ├── main.rs                        # CLI 入口 (init / merge)
│   ├── hook/                          # LD_PRELOAD Hook 库
│   │   ├── hook.c                    # C 拦截器（基于 c2rust-demo 移植）
│   │   ├── Makefile                  # 编译 libhook.so
│   │   └── cpp_preprocessor.rs       # C++ 预处理捕获（Rust侧）
│   ├── compiler/
│   │   ├── ast_capturer.rs           # 从预处理文件解析扩展 AST
│   │   ├── instantiation_tracker.rs  # 模板实例化追踪
│   │   └── cursor_visitor.rs         # AST 遍历器
│   ├── extractor/
│   │   ├── class_extractor.rs
│   │   ├── function_extractor.rs
│   │   ├── template_extractor.rs     # 基于实例化结果提取
│   │   ├── vtable_extractor.rs
│   │   ├── lambda_extractor.rs
│   │   ├── macro_expander.rs         # 宏展开处理
│   │   └── enum_extractor.rs
│   ├── postprocessor/
│   │   ├── operator_handler.rs
│   │   ├── friend_handler.rs
│   │   ├── lambda_handler.rs
│   │   ├── rtti_handler.rs
│   │   └── variadic_handler.rs
│   ├── generator/
│   │   ├── hicc_codegen.rs
│   │   ├── class_generator.rs
│   │   ├── template_generator.rs
│   │   ├── vtable_generator.rs
│   │   └── project_generator.rs
│   └── todo_collector.rs
└── Cargo.toml
```

### 2.2 四阶段处理流程

```
1. 编译拦截 (hook/)
   └── LD_PRELOAD 注入，捕获完整编译过程
       ├── 预处理：生成 .cpp2rust 展开文件
       ├── AST 导出：生成 .ast.json（包含实际实例化）
       └── 符号记录：生成 .symbols.json

2. 提取 (extractor/)
   ├── 类/结构体/友元/运算符
   ├── 函数（含 operator 方法）
   ├── 模板实例化（只关注已实例化的具体类型）
   ├── 虚函数表
   ├── Lambda 表达式
   ├── 宏展开代码块（来自预处理器输出）
   └── 枚举

3. 后处理 (postprocessor/)
   ├── 运算符重载 → named shim 函数
   ├── 友元函数   → import_lib!
   ├── Lambda     → fn ptr / class wrapper
   ├── RTTI       → 枚举 + 虚函数
   └── 可变参数   → 固定元数展开

4. 生成 (generator/)
   └── hicc 宏格式 Rust 代码
```

### 2.3 编译拦截流程详解

#### 2.3.1 Hook 机制（基于 c2rust-demo 移植）

```c
// hook/hook.cpp（支持 g++/clang++）
#define _GNU_SOURCE
#include <dlfcn.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <sys/wait.h>

// 支持 C++ 编译器
static const char* cxx_compilers[] = {
    "g++", "clang++", "c++", "gcc", "clang", NULL
};

__attribute__((constructor))
void init() {
    // 读取 /proc/self/cmdline 获取实际调用
    // 解析参数，识别 -c, -o, 源文件等
    // Fork 执行预处理 + AST 导出
}
```

**与 c2rust-demo hook.c 的差异**：
- 原版仅支持 C (gcc/clang)，v5 扩展支持 C++ (g++/clang++)
- 增加 AST JSON 导出功能
- 支持模板实例化信息提取

#### 2.3.2 预处理捕获

```bash
# 对每个 .cpp 文件执行
clang++ -E -C -P -I<inc_paths> -D<defs> foo.cpp > foo.cpp2rust
```

生成的 `.cpp2rust` 文件：
- 宏已完全展开
- 条件编译已裁剪（仅保留实际采用的分支）
- `#line` 指令保留（可选，用于定位）

#### 2.3.3 AST 实例化捕获

```bash
# 导出完整 AST（包含实例化细节）
clang++ -Xclang -ast-dump=json -fsyntax-only foo.cpp > foo.ast.json
```

关键：模板实例化信息在 AST 中体现为 `ClassTemplateSpecializationDecl` 节点，包含：
- 模板参数实际类型（如 `int` 而非 `T`）
- 成员 specialization 信息
- 实例化来源位置

### 2.4 输出目录结构

```
.c2rust/<feature>/
├── hook/                           # Hook 库
│   └── libhook.so
├── capture/                         # 编译拦截产物
│   ├── foo.cpp.cpp2rust           # 预处理后的 C++ 代码
│   ├── foo.cpp.ast.json           # AST 导出（含实例化）
│   ├── foo.cpp.symbols.json       # 符号信息
│   └── targets.list               # 链接目标列表
├── meta/
│   ├── build_env.txt              # 捕获时的环境变量
│   ├── build_cmd.txt              # 实际构建命令
│   ├── selected_files.json
│   └── init-interface-report.md
└── rust/                           # 生成的 Rust 项目
    ├── Cargo.toml
    └── src/
        ├── lib.rs
        └── mod_xxx.rs
```

---

## 3. 核心设计

### 3.1 模板实例化：只关注结果，不关注模板

**核心理念**：
- 模板只是生成代码的"配方"，真正重要的是**实例化后的具体类型**
- `std::vector<int>` 和 `std::vector<std::string>` 是两个完全不同的类型
- 转换时只需要捕获 `std::vector<int>` 的完整接口，不需要知道它是模板实例化出来的

**实例化发现流程**：
```
编译拦截 → AST 解析 → 遍历 ClassTemplateSpecializationDecl
                            ↓
                    检查是否由当前翻译单元实例化
                            ↓
                    收集 template_args（实际类型）
                            ↓
                    生成模板实例化记录
```

**实例化与 AST 节点对应**：
```json
// foo.ast.json 中的模板实例化节点
{
  "id": 42,
  "kind": "ClassTemplateSpecialization",
  "name": "vector",
  "templateArgs": [
    {"kind": "Type", "type": "int"},
    {"kind": "Allocator", "type": "std::allocator<int>"}
  ],
  "specializationKind": "Implicit",
  "location": {"file": "foo.cpp", "line": 15, "col": 10},
  "methods": [
    {"name": "push_back", "signature": "void (int&&)", "mangled": "_ZNSt6vectorIiSaIiEE9push_backEOi"},
    {"name": "size", "signature": "size_t () const", "mangled": "_ZNSt6vectorIiSaIiEE4sizeEv"}
  ]
}
```

### 3.2 数据结构

```rust
/// 编译捕获结果
pub struct CaptureResult {
    /// 预处理后的 C++ 源代码
    pub preprocessed_source: PathBuf,
    /// AST JSON 文件路径
    pub ast_json: PathBuf,
    /// 符号映射
    pub symbols: SymbolMap,
    /// 实际实例化的模板列表
    pub instantiations: Vec<TemplateInstantiation>,
    /// 宏展开的代码块（位置 → 展开内容）
    pub macro_expansions: HashMap<SourceLocation, String>,
}

/// 模板实例化信息（关注实例化结果，而非模板本身）
pub struct TemplateInstantiation {
    /// 实例化后的类型名（如 `std::vector<int>`）
    pub instantiated_name: String,
    /// 模板参数（展开后的实际类型）
    pub template_args: Vec<String>,
    /// 实例化发生的文件位置
    pub location: SourceLocation,
    /// 生成的 mangled 符号名
    pub mangled_name: Option<String>,
    /// 实例化的成员函数列表
    pub methods: Vec<InstantiatedMethod>,
}

pub struct InstantiatedMethod {
    pub name: String,
    pub signature: String,
    pub mangled: String,
}

/// 符号映射
pub struct SymbolMap {
    /// C++ 符号名 → Rust link_name
    pub cpp_to_rust: HashMap<String, String>,
    /// mangled → demangled
    pub mangled_to_demangled: HashMap<String, String>,
}
```

### 3.3 宏展开处理

#### 3.3.1 宏分类

| 宏类型 | 处理策略 | 示例 |
|--------|---------|------|
| 对象宏（Object-like） | 捕获展开值，生成 `const` 常量 | `#define MAX_SIZE 1024` |
| 函数宏（Function-like） | 保留定义，生成 `macro_rules!` | `#define SWAP(a, b) do { ... } while(0)` |
| 条件宏 | 捕获实际分支，丢弃未选中分支 | `#ifdef DEBUG_LOGGING` |
| 编译器特定宏 | 标记为 `#[cfg()]` 条件 | `__GNUC__`, `__clang__` |

#### 3.3.2 宏展开代码块

```rust
// 从 .cpp2rust 文件中识别宏展开区域
hicc::cpp! {
    // ===== 宏展开区域（来自 foo.cpp:42） =====
    int composite_key_impl(int a, int b) {
        return ((a) * 1000 + (b));  // 宏已展开
    }
}
```

---

## 4. C++ 特性支持详情

### 4.1 总览

| 类别 | 特性数 | ✅ 完全自动 | ⚠️ 降级生成（内联 TODO） | ❌ 不支持 |
|------|--------|------------|------------------------|---------|
| 基础类型与函数 | 5 | 5 | 0 | 0 |
| 类与对象 | 7 | 7 | 0 | 0 |
| 面向对象特性 | 6 | 6 | 0 | 0 |
| 运算符与类型 | 5 | 3 | 2 | 0 |
| 模板实例化 | 5 | 5 | 0 | 0 |
| 智能指针与内存 | 5 | 5 | 0 | 0 |
| STL 容器 | 5 | 5 | 0 | 0 |
| 函数对象 | 4 | 3 | 1 | 0 |
| 其他高级特性 | 6 | 6 | 0 | 0 |
| **总计** | **48** | **45** | **3** | **0** |

> **v5 改进**：相比 v4（44 直接 + 4 降级），v5 通过编译时捕获将模板实例化从 ⚠️ 升级为 ✅，实现 45 直接 + 3 降级。

### 4.2 后处理特性详情（3 个 ⚠️）

| 特性 | 示例 | 处理方式 | 内联 TODO tag |
|------|------|---------|--------------|
| 运算符重载 | 019 | named shim + `[OP]` 提示可实现 std::ops trait | `[OP]` |
| 友元函数 | 020 | 直接入 import_lib! + `[FR]` 标注 | `[FR]` |
| typeid/RTTI | 023 | 枚举注入 + `[RTTI]` 标注 | `[RTTI]` |

### 4.3 详细特性表

#### 基础类型与函数 (1-5)

| 示例 | 特性 | AST 节点 | 支持 | 处理方式 |
|------|------|----------|------|---------|
| 001_hello_world | extern "C" 函数 | `FunctionDecl` | ✅ | 直接生成 import_lib! |
| 002_function_overload | 函数重载 | `FunctionDecl` (多个同名) | ✅ | 每个重载生成独立条目 |
| 003_default_args | 默认参数 | `ParmVarDecl` (带默认值) | ✅ | shim 包装默认值 |
| 004_inline_functions | 内联函数 | `FunctionDecl` + `inline` | ✅ | 内联到 hicc::cpp! 块 |
| 005_variadic_functions | 可变参数函数 | `FunctionDecl` (va_list) | ✅ | C variadic 直接映射 |

#### 类与对象 (6-12)

| 示例 | 特性 | AST 节点 | 支持 | 处理方式 |
|------|------|----------|------|---------|
| 006_class_basic | 基础类 | `CXXRecordDecl` | ✅ | opaque ptr + import_class! |
| 007_class_constructor | 构造/析构 | `CXXConstructorDecl` | ✅ | `*_new()` / `*_delete()` shim |
| 008_class_copy | 拷贝构造 | `CXXConstructorDecl` (copy) | ✅ | `*_copy()` shim |
| 009_class_move | 移动构造 | `CXXConstructorDecl` (move) | ✅ | `*_move()` shim |
| 010_class_static | 静态成员 | `VarDecl` (static) | ✅ | 静态访问 shim |
| 011_class_const | const 成员函数 | `CXXMethodDecl` (const) | ✅ | `&self` 绑定 |
| 012_class_volatile | volatile 成员函数 | `CXXMethodDecl` (volatile) | ✅ | 透传 volatile 语义 |

#### 面向对象特性 (13-18)

| 示例 | 特性 | AST 节点 | 支持 | 处理方式 |
|------|------|----------|------|---------|
| 013_inheritance_single | 单继承 | `CXXBaseSpecifier` | ✅ | 基类方法提升到派生类 import_class! |
| 014_inheritance_multiple | 多继承 | `CXXBaseSpecifier` (多个) | ✅ | 多继承链展开 |
| 015_virtual_basic | 虚函数 | `CXXMethodDecl` (virtual) | ✅ | vtable shim |
| 016_virtual_pure | 纯虚/抽象类 | `CXXMethodDecl` (= 0) | ✅ | Rust trait 接口 |
| 017_virtual_override | override | `CXXMethodDecl` (override) | ✅ | override 透传 |
| 018_virtual_diamond | 菱形继承 | `CXXBaseSpecifier` (virtual) | ✅ | virtual 继承展开 |

#### 运算符与类型 (19-23)

| 示例 | 特性 | AST 节点 | 支持 | 处理方式 |
|------|------|----------|------|---------|
| 019_operator_overload | 运算符重载 | `CXXMethodDecl` (operator) | ⚠️ | named shim + `[OP]` TODO |
| 020_friend_function | 友元函数 | `FriendDecl` | ⚠️ | 直接入 import_lib! + `[FR]` TODO |
| 021_explicit_ctor | explicit 构造 | `CXXConstructorDecl` (explicit) | ✅ | explicit 标记保留 |
| 022_mutable_member | mutable 成员 | `FieldDecl` (mutable) | ✅ | `&mut self` 访问函数 |
| 023_typeid_rtti | typeid/RTTI | `CXXTypeidExpr` | ⚠️ | 枚举注入 + `[RTTI]` TODO |

#### 模板实例化 (24-28)

| 示例 | 特性 | AST 节点 | 支持 | 处理方式 |
|------|------|----------|------|---------|
| 024_template_function | 函数模板 | `FunctionTemplateDecl` → 实例化 | ✅ | 捕获实际实例化 |
| 025_template_class | 类模板 | `ClassTemplateDecl` → 实例化 | ✅ | 捕获 `ClassTemplateSpecialization` |
| 026_template_specialization | 模板偏特化 | `ClassTemplatePartialSpecialization` | ✅ | 捕获偏特化实例化 |
| 027_template_instantiation | 显式实例化 | `ClassTemplateSpecialization` | ✅ | 捕获显式实例化声明 |
| 028_variadic_template | 可变参数模板 | `VariadicTemplate` | ✅ | 捕获固定元数展开调用点 |

> **v5 核心优势**：v4 的模板处理基于静态分析推断实例化，v5 通过编译拦截直接捕获实际实例化结果，更准确。

#### 智能指针与内存 (29-33)

| 示例 | 特性 | AST 节点 | 支持 | 处理方式 |
|------|------|----------|------|---------|
| 029_unique_ptr | std::unique_ptr | `CXXNewExpr` | ✅ | hicc-smart-ptr 封装 |
| 030_shared_ptr | std::shared_ptr | `CXXNewExpr` | ✅ | hicc-smart-ptr 封装 |
| 031_custom_deleter | 自定义删除器 | `FunctionDecl` | ✅ | 删除器函数注入 |
| 032_placement_new | Placement new | `CXXNewExpr` | ✅ | placement new shim |
| 033_raii_pattern | RAII 模式 | 构造/析构 | ✅ | Drop trait 模式 |

#### STL 容器 (34-38)

| 示例 | 特性 | AST 节点 | 支持 | 处理方式 |
|------|------|----------|------|---------|
| 034_vector_basic | std::vector | `ClassTemplateSpecialization` | ✅ | hicc-std 封装 |
| 035_map_basic | std::map | `ClassTemplateSpecialization` | ✅ | hicc-std 封装 |
| 036_string_basic | std::string | `ClassTemplateSpecialization` | ✅ | hicc-std 封装 |
| 037_array_basic | std::array | `ClassTemplateSpecialization` | ✅ | hicc-std 封装 |
| 038_tuple_basic | std::tuple | `ClassTemplateSpecialization` | ✅ | hicc-std 封装 |

#### 函数对象 (39-42)

| 示例 | 特性 | AST 节点 | 支持 | 处理方式 |
|------|------|----------|------|---------|
| 039_lambda_basic | Lambda | `LambdaExpr` | ✅ | 无状态 fn ptr / 有状态 class wrapper |
| 040_std_function | std::function | `ClassTemplateSpecialization` | ✅ | hicc-std 封装 |
| 041_functional_bind | std::bind | `CallExpr` | ✅ | class wrapper 模式 |
| 042_exception_basic | 异常处理 | `CXXThrowExpr` | ✅ | 异常框架透传 |

#### 其他高级特性 (43-48)

| 示例 | 特性 | AST 节点 | 支持 | 处理方式 |
|------|------|----------|------|---------|
| 043_namespace_nested | 嵌套命名空间 | `NamespaceDecl` | ✅ | Rust mod 嵌套 |
| 044_enum_class | 强类型枚举 | `EnumDecl` (scoped) | ✅ | Rust enum |
| 045_union_basic | union | `RecordDecl` (union) | ✅ | Rust union |
| 046_constexpr_basic | constexpr | `Expr` (constexpr) | ✅ | const 常量 |
| 047_noexcept_basic | noexcept | `NoexceptSpec` | ✅ | 透传 noexcept |
| 048_summary | FFI 模式总结 | — | ✅ | 综合示例 |

**图例**：✅ 完全自动  ⚠️ 降级生成（含内联 TODO）

---

## 5. 局限性及处理方案

### 5.1 局限性总览

| 限制 | 说明 | 影响 | 处理方案 |
|------|------|------|---------|
| 仅支持 Linux | `LD_PRELOAD` 是 Linux 特有机制 | 无法在 macOS/Windows 上使用 | 提供 Docker 容器方案 |
| 需要完整构建环境 | 依赖项目能成功编译 | 某些代码可能无法通过编译 | `--skip-failed` 跳过失败文件 |
| 模板实例化依赖编译 | 如果某模板从未实例化，不会出现在捕获中 | 某些模板类型可能缺失 | 提供 `--instantiate-all` 扫描所有实例化 |
| 宏展开后代码膨胀 | 预处理后代码量可能很大 | 处理时间增加 | 支持增量处理，只处理变更文件 |
| 无法捕获运行时信息 | 只捕获编译时信息 | 某些泛型代码无法完全转换 | 降级为手动补全 |

### 5.2 局限性详细处理方案

#### 5.2.1 Linux only

**问题**：`LD_PRELOAD` 是 Linux 特有的环境变量，macOS 使用 `DYLD_INSERT_LIBRARIES`，Windows 没有等效机制。

**处理方案**：
1. 提供 Docker 容器镜像，包含完整的 Linux 构建环境
2. macOS 用户可在 Docker 中运行
3. Windows 用户使用 WSL2

```bash
# Docker 使用方式
docker run --rm -v $(pwd):/project \
    -e C2RUST_FEATURE_ROOT=/project/.c2rust \
    cpp2rust-ffi:v5 bash -c "make -j4"
```

#### 5.2.2 需要完整构建环境

**问题**：如果项目有编译错误，LD_PRELOAD 拦截会失败。

**处理方案**：
1. 提供 `--skip-failed` 选项，跳过编译失败的文件
2. 记录失败文件列表，生成报告供用户手动处理
3. 支持 `--ignore-errors` 选项，强制捕获成功的文件

```bash
# 跳过编译失败的文件，继续处理成功的文件
cpp2rust-ffi init -i .c2rust/v5/capture -o ./rust_hicc --skip-failed

# 生成失败文件报告
# → meta/failed-builds.txt
```

#### 5.2.3 模板实例化依赖编译

**问题**：只有被实际使用的模板实例化才会出现在 AST 中。如果某实例化只在头文件声明但未使用，不会被捕获。

**处理方案**：
1. 提供 `--instantiate-all` 选项，触发所有可见的模板实例化
2. 通过代码覆盖分析，识别未被调用的实例化
3. 生成 `TODO: 未实例化的模板` 报告

```bash
# 强制实例化常用标准库模板
cpp2rust-ffi init -i .c2rust/v5/capture -o ./rust_hicc \
    --instantiate-templates=std::vector,std::map,std::string
```

#### 5.2.4 宏展开后代码膨胀

**问题**：预处理后的 `.cpp2rust` 文件可能比原始文件大数倍，包含大量展开的宏代码。

**处理方案**：
1. 支持增量处理，只处理变更的源文件
2. 提供 `--prune-macros` 选项，只保留实际被引用的宏展开
3. 缓存机制，避免重复处理未变更的文件

```bash
# 增量处理（只处理变更文件）
cpp2rust-ffi init -i .c2rust/v5/capture -o ./rust_hicc --incremental

# 只保留被使用的宏展开
cpp2rust-ffi init -i .c2rust/v5/capture -o ./rust_hicc --prune-macros
```

#### 5.2.5 无法捕获运行时信息

**问题**：某些 C++ 特性（如 `decltype`、运行时多态）无法通过编译时捕获完全转换。

**处理方案**：
1. 降级为 opaque pointer 模式
2. 生成 `TODO: 需要手动补全` 注释
3. 提供 `--runtime-analysis` 辅助分析

```rust
// 降级处理示例
hicc::import_lib! {
    // TODO: decltype 返回值类型需要手动确认
    #[cpp(func = "auto deduce_type(...) -> ???")]
    fn deduce_type(...) -> /* TODO */;
}
```

---

## 6. 工作流程

### 6.1 完整流程

```bash
# Step 1: 编译拦截（复用项目构建系统）
cd cpp-project/
C2RUST_FEATURE_ROOT=.c2rust/v5 \
C2RUST_CC=g++ \
C2RUST_LD=g++ \
LD_PRELOAD=/path/to/libhook.so \
    make -j4

# Step 2: 初始化（基于捕获结果生成 Rust 脚手架）
cpp2rust-ffi init -i .c2rust/v5/capture -o ./rust_hicc

# Step 3: 合并（如需要）
cpp2rust-ffi merge -i ./rust_hicc
```

### 6.2 高级选项

```bash
# 多构建配置支持
cpp2rust-ffi init --merge-configs \
    --inputs .c2rust/v5-debug/capture \
             .c2rust/v5-release/capture \
    -o ./rust_hicc

# 强制实例化指定模板
cpp2rust-ffi init -i .c2rust/v5/capture -o ./rust_hicc \
    --instantiate-templates=std::vector,std::map,std::string

# 跳过编译失败的文件
cpp2rust-ffi init -i .c2rust/v5/capture -o ./rust_hicc --skip-failed
```

---

## 7. 实现计划

| 阶段 | 内容 | 优先级 | 依赖 |
|------|------|--------|------|
| Phase 0 | 移植 hook.c → hook.cpp（支持 C++ 编译器） | P0 | c2rust-demo hook.c |
| Phase 1 | AST 编译引擎（基于拦截输入） | P0 | Phase 0 |
| Phase 2 | 模板实例化追踪器（`instantiation_tracker.rs`） | P0 | Phase 1 |
| Phase 3 | 宏展开处理器（`macro_expander.rs`） | P1 | Phase 1 |
| Phase 4 | hicc 宏格式代码生成器 | P0 | Phase 2, 3 |
| Phase 5 | 基础提取器（类/函数/枚举/虚函数表） | P0 | Phase 1 |
| Phase 6 | 运算符/友元/Lambda/RTTI 后处理 | P1 | Phase 5 |
| Phase 7 | 多构建配置合并支持 | P2 | Phase 1-6 |
| Phase 8 | 局限性处理方案（Docker/增量/跳过失败） | P1 | Phase 1-7 |
| Phase 9 | 集成测试 | P1 | Phase 1-8 |

---

## 8. 技术依赖

```toml
[dependencies]
clang = "2"            # libclang 绑定
clap = "4"             # CLI
anyhow = "1"           # 错误处理
serde = { version = "1", features = ["derive"] }
serde_json = "1"       # AST JSON 解析
tempfile = "3"         # 临时文件管理
walkdir = "2"          # 目录遍历

[build-dependencies]
cc = "1"               # C++ 编译器调用
```

```bash
# 系统依赖
apt-get install clang libclang-dev
apt-get install g++ libstdc++-dev
```
