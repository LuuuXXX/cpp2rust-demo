# C++ 到 Rust Safe FFI 自动化工具 - 方案 v5

## 1. 概述

### 1.1 核心思路：LD_PRELOAD 编译拦截

v5 是完全重新设计的版本，借鉴 `c2rust-demo` 的构建捕获机制，通过 `LD_PRELOAD` 注入共享库拦截编译器进程，在**真实编译过程中**捕获转换所需的所有信息：

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

### 1.2 为什么需要编译拦截

静态分析（v4 方案）**无法捕获以下信息**：

| 信息类型 | 静态分析局限 | 编译时捕获优势 |
|---------|--------------|---------------|
| 模板实例化 | 只能看到模板声明，不知道哪些实例化会发生 | 实际编译触发哪些实例化，捕获完整的 `ClassTemplateSpecialization` |
| 宏展开 | 只能看到宏定义，不知道实际展开结果 | 捕获展开后的预处理代码，包括条件编译分支 |
| Include 依赖 | 只能分析头文件引用，无法知道运行时包含 | 实际被编译的代码行，反映真实依赖 |
| 编译符号 | 只能看到声明，不知道链接符号 | 捕获实际的 mangled/demangled 符号名 |
| 条件编译 | 无法知道哪个 `#ifdef` 分支被采用 | 捕获实际采用的代码路径 |

### 1.3 版本定位

v5 是完全独立的新版本，不保留 v4 的静态分析回退机制。所有输入都必须通过 LD_PRELOAD 编译拦截方式获取。

### 1.4 版本演进

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
│   │   ├── instantiation_tracker.rs  # 模板实例化追踪（新增）
│   │   └── cursor_visitor.rs         # AST 遍历器
│   ├── extractor/
│   │   ├── class_extractor.rs
│   │   ├── function_extractor.rs
│   │   ├── template_extractor.rs     # 基于实例化追踪重写
│   │   ├── vtable_extractor.rs
│   │   ├── lambda_extractor.rs
│   │   ├── macro_expander.rs         # 宏展开处理（新增）
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
   ├── 模板实例化（基于实际编译触发的实例化）
   ├── 虚函数表
   ├── Lambda 表达式
   ├── 宏展开代码块（新增：来自预处理器输出）
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
- 模板参数实际类型
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

### 3.1 数据结构

```rust
/// 编译捕获结果（v5）
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

/// 模板实例化信息
pub struct TemplateInstantiation {
    /// 模板名称（如 `std::vector`）
    pub template_name: String,
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

### 3.2 宏展开处理

#### 3.2.1 宏分类

| 宏类型 | 处理策略 | 示例 |
|--------|---------|------|
| 对象宏（Object-like） | 捕获展开值，生成 `const` 常量 | `#define MAX_SIZE 1024` |
| 函数宏（Function-like） | 保留定义，生成 `macro_rules!` | `#define SWAP(a, b) do { ... } while(0)` |
| 条件宏 | 捕获实际分支，丢弃未选中分支 | `#ifdef DEBUG_LOGGING` |
| 编译器特定宏 | 标记为 `#[cfg()]` 条件 | `__GNUC__`, `__clang__` |

#### 3.2.2 宏展开代码块

```rust
// 从 .cpp2rust 文件中识别宏展开区域
// 生成hicc::cpp! 块时标注宏来源
hicc::cpp! {
    // ===== 宏展开区域（来自 foo.cpp:42） =====
    // #define COMPOSITE_KEY(a, b) ((a) * 1000 + (b))
    // 展开为:
    int composite_key_impl(int a, int b) {
        return ((a) * 1000 + (b));
    }
}
```

### 3.3 模板实例化追踪

#### 3.3.1 实例化发现流程

```
编译拦截 → AST 解析 → 遍历 ClassTemplateSpecializationDecl
                            ↓
                    检查是否由当前翻译单元实例化
                            ↓
                    收集 template_args（实际类型）
                            ↓
                    生成模板实例化记录
```

#### 3.3.2 实例化与 AST 节点对应

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

---

## 4. 模块差异（v4 → v5）

### 4.1 新增模块

| 模块 | 功能 | 输入 | 输出 |
|------|------|------|------|
| `hook/` | LD_PRELOAD 拦截器 | 编译命令 | `.cpp2rust`, `.ast.json`, `.symbols.json` |
| `instantiation_tracker.rs` | 模板实例化追踪 | AST JSON | `TemplateInstantiation` 列表 |
| `macro_expander.rs` | 宏展开处理 | 预处理文件 | 宏常量 + `macro_rules!` 定义 |

### 4.2 重写模块

| 模块 | v4 行为 | v5 行为 |
|------|--------|--------|
| `ast_compiler.rs` | libclang 单独编译 | LD_PRELOAD 拦截，复用项目构建 |
| `template_extractor.rs` | 基于声明推断实例化 | 基于实际 AST 实例化节点 |
| `function_extractor.rs` | 解析函数声明 | 解析预处理后代码 + 符号映射 |

---

## 5. 工作流程

### 5.1 完整流程

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

### 5.2 多构建配置支持

```bash
# Debug 构建捕获
C2RUST_FEATURE_ROOT=.c2rust/v5-debug \
    make config=debug

# Release 构建捕获
C2RUST_FEATURE_ROOT=.c2rust/v5-release \
    make config=release

# 合并多配置结果
cpp2rust-ffi init --merge-configs \
    --inputs .c2rust/v5-debug/capture \
             .c2rust/v5-release/capture \
    -o ./rust_hicc
```

---

## 6. 实现计划

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
| Phase 8 | 集成测试 | P1 | Phase 1-7 |

---

## 7. 技术依赖

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

---

## 8. 局限性

| 限制 | 说明 | 规避方案 |
|------|------|---------|
| 仅支持 Linux | `LD_PRELOAD` 是 Linux 特有机制 | 未来可扩展到 macOS (DYLD_INSERT_LIBRARIES) 或 Windows |
| 需要完整构建环境 | 依赖项目能成功编译 | 预留 `--skip-build` 降级模式 |
| 模板实例化依赖编译 | 如果某模板从未实例化，不会出现在捕获中 | 可通过 `--instantiate-all` 强制实例化常用模板 |
| 宏展开后代码膨胀 | 预处理后代码量可能很大 | 支持增量处理，只处理变更文件 |
