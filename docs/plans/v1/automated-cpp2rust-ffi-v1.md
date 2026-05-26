# C++ 到 Rust Safe FFI 自动化工具 - v1 计划

## 1. 背景与目标

### 1.1 当前状态

当前 `examples/` 目录下的 48 个示例采用**手动编写**的方式实现 C++ 到 Rust FFI：

```
examples/043_namespace_nested/
├── cpp/
│   ├── namespace_nested.h        # C++ 头文件
│   └── namespace_nested.cpp     # C++ 实现
└── rust_hicc/
    ├── build.rs                 # 手动编写
    ├── Cargo.toml               # 手动编写
    └── src/main.rs              # 手动编写 hicc 宏
```

### 1.2 问题

1. **重复性工作**：每个示例需要手动编写对应的 Rust FFI 代码
2. **维护成本**：C++ API 变更时需要同步修改 Rust 代码
3. **学习曲线**：需要了解 hicc 宏的详细用法

### 1.3 目标

开发一个**自动化工具**，输入是 C++ 头文件，输出是完整的 rust_hicc 项目结构（包括 build.rs、Cargo.toml、src/main.rs）。

## 2. 技术方案

### 2.1 现有工具分析

| 工具 | 原理 | 输入 | 输出 | 自动化程度 |
|------|------|------|------|------------|
| **c2rust-demo** | LD_PRELOAD hook 捕获构建过程 | C 项目 + make | Rust scaffold | 完全自动 |
| **hicc** | 过程宏内联 C++ | Rust 文件中手写 cpp! | 直接编译 | 手动 |
| **bindgen** | Clang 解析 C 头文件 | .h 文件 | Rust FFI 类型 | 完全自动 |

### 2.2 自动化方案

参考 `bindgen` 的头文件解析能力，设计一个针对 hicc 的自动化工具。

**核心思路**：
1. 使用 **Clang** 解析 C++ 头文件，提取类型、函数、类信息
2. 使用 **hicc 宏模板**生成 Rust FFI 代码
3. 生成完整的 rust_hicc 项目结构

### 2.3 架构设计

```
cpp2rust-ffi tool
├── src/
│   ├── main.rs              # CLI 入口
│   ├── parser/              # C++ 头文件解析
│   │   ├── mod.rs
│   │   ├── clang_wrapper.rs # libclang 封装
│   │   └── type_analyzer.rs # 类型分析
│   ├── generator/          # Rust 代码生成
│   │   ├── mod.rs
│   │   ├── class_generator.rs
│   │   ├── func_generator.rs
│   │   └── project_generator.rs
│   └── template/            # 项目模板
│       ├── build_rs.tpl
│       ├── cargo_toml.tpl
│       └── main_rs.tpl
├── Cargo.toml
└── README.md
```

## 3. Examples C++ 特性覆盖详情

### 3.1 特性分类总表

根据 `./examples/` 中 48 个示例的 C++ 特性：

#### 基础类型与函数 (1-5)

| 示例 | 特性 | AST 节点 | v1 支持 |
|------|------|----------|---------|
| 001_hello_world | extern "C" 函数 | `FunctionDecl` | ✅ |
| 002_function_overload | 函数重载 | `FunctionDecl` (多个同名) | ✅ |
| 003_default_args | 默认参数 | `ParmVarDecl` (带默认值) | ✅ |
| 004_inline_functions | 内联函数 | `FunctionDecl` + `inline` | ✅ |
| 005_variadic_functions | 可变参数函数 | `FunctionDecl` (可变参数) | ✅ |

#### 类与对象 (6-12)

| 示例 | 特性 | AST 节点 | v1 支持 |
|------|------|----------|---------|
| 006_class_basic | 基础类 | `CXXRecordDecl` | ✅ |
| 007_class_constructor | 构造/析构函数 | `CXXConstructorDecl`, `CXXDestructorDecl` | ✅ |
| 008_class_copy | 拷贝构造函数 | `CXXConstructorDecl` (copy) | ✅ |
| 009_class_move | 移动构造函数 | `CXXConstructorDecl` (move) | ✅ |
| 010_class_static | 静态成员 | `VarDecl` (static) | ✅ |
| 011_class_const | const 成员函数 | `CXXMethodDecl` (const) | ✅ |
| 012_class_volatile | volatile 成员函数 | `CXXMethodDecl` (volatile) | ✅ |

#### 面向对象特性 (13-18)

| 示例 | 特性 | AST 节点 | v1 支持 | 不支持原因 |
|------|------|----------|---------|------------|
| 013_inheritance_single | 单继承 | `CXXBaseSpecifier` | ✅ | |
| 014_inheritance_multiple | 多继承 | `CXXBaseSpecifier` (多个) | ✅ | |
| 015_virtual_basic | 虚函数基础 | `CXXMethodDecl` (virtual) | ✅ | |
| 016_virtual_pure | 纯虚函数/抽象类 | `CXXMethodDecl` (= 0) | ⚠️ | 可解析声明，但无法生成 Rust trait 接口 |
| 017_virtual_override | override 说明符 | `CXXMethodDecl` (override) | ✅ | |
| 018_virtual_diamond | 菱形继承 | `CXXBaseSpecifier` (virtual) | ⚠️ | 可解析继承关系，但 vtable 映射复杂 |

#### 运算符与类型 (19-23)

| 示例 | 特性 | AST 节点 | v1 支持 | 不支持原因 |
|------|------|----------|---------|------------|
| 019_operator_overload | 运算符重载 | `CXXMethodDecl` (operator) | ❌ | `operator+` 等需要映射到 Rust trait（如 `Add`），语义复杂 |
| 020_friend_function | 友元函数 | `FriendDecl` | ❌ | 友元函数不是类的成员，但可访问私有成员，FFI 映射困难 |
| 021_explicit_ctor | explicit 构造函数 | `CXXConstructorDecl` (explicit) | ✅ | |
| 022_mutable_member | mutable 成员 | `FieldDecl` (mutable) | ✅ | |
| 023_typeid_rtti | typeid 与 RTTI | `CXXTypeidExpr` | ❌ | 需要运行时类型信息，纯解析无法获取 |

#### 模板 (24-28)

| 示例 | 特性 | AST 节点 | v1 支持 | 不支持原因 |
|------|------|----------|---------|------------|
| 024_template_function | 函数模板 | `FunctionTemplateDecl` | ✅ | |
| 025_template_class | 类模板 | `ClassTemplateDecl` | ⚠️ | 可解析模板声明（如 `template<class T> class Foo`），但无法获取具体实例化（如 `Foo<int>`） |
| 026_template_specialization | 模板偏特化 | `ClassTemplatePartialSpecialization` | ❌ | 偏特化涉及复杂的模板参数匹配逻辑 |
| 027_template_instantiation | 模板显式实例化 | `ClassTemplateSpecialization` | ⚠️ | 只能解析声明，无法捕获编译时实例化结果 |
| 028_variadic_template | 可变参数模板 | `VariadicTemplate` | ❌ | `Args...` 语法涉及参数包展开，语义分析复杂 |

#### 智能指针与内存 (29-33)

| 示例 | 特性 | AST 节点 | v1 支持 | 不支持原因 |
|------|------|----------|---------|------------|
| 029_unique_ptr | std::unique_ptr | `CXXNewExpr`, `TypeRef` | ✅ | |
| 030_shared_ptr | std::shared_ptr | `CXXNewExpr`, `TypeRef` | ✅ | |
| 031_custom_deleter | 自定义删除器 | `FunctionDecl` | ✅ | |
| 032_placement_new | Placement new | `CXXNewExpr` | ✅ | |
| 033_raii_pattern | RAII 模式 | 构造/析构函数 | ✅ | |

#### STL 容器 (34-38)

| 示例 | 特性 | AST 节点 | v1 支持 | 不支持原因 |
|------|------|----------|---------|------------|
| 034_vector_basic | std::vector | `ClassTemplateSpecialization` | ⚠️ | 只能解析为 opaque pointer，无法获取具体模板参数 `std::vector<int>` |
| 035_map_basic | std::map | `ClassTemplateSpecialization` | ⚠️ | 同上 |
| 036_string_basic | std::string | `ClassTemplateSpecialization` | ⚠️ | 同上 |
| 037_array_basic | std::array | `ClassTemplateSpecialization` | ⚠️ | 同上 |
| 038_tuple_basic | std::tuple | `ClassTemplateSpecialization` | ⚠️ | 同上 |

#### 函数对象 (39-42)

| 示例 | 特性 | AST 节点 | v1 支持 | 不支持原因 |
|------|------|----------|---------|------------|
| 039_lambda_basic | Lambda 表达式 | `LambdaExpr` | ❌ | Lambda 是匿名函数对象，涉及闭包语义，需要完整的语义分析 |
| 040_std_function | std::function | `ClassTemplateSpecialization` | ⚠️ | 模板实例化问题（同 STL 容器） |
| 041_functional_bind | std::bind | `CallExpr` | ❌ | 绑定器涉及参数绑定语义，较为复杂 |
| 042_exception_basic | 异常处理 | `CXXThrowExpr`, `CXXCatchStmt` | ✅ | |

#### 其他高级特性 (43-48)

| 示例 | 特性 | AST 节点 | v1 支持 | 不支持原因 |
|------|------|----------|---------|------------|
| 043_namespace_nested | 嵌套命名空间 | `NamespaceDecl` (嵌套) | ✅ | |
| 044_enum_class | 强类型枚举 | `EnumDecl` (scoped) | ✅ | |
| 045_union_basic | 共用体 | `RecordDecl` (union) | ✅ | |
| 046_constexpr_basic | constexpr | `Expr` (constexpr) | ✅ | |
| 047_noexcept_basic | noexcept | `NoexceptSpec` | ✅ | |
| 048_summary | FFI 模式总结 | - | ✅ | |

**图例**：✅ 完全支持 ⚠️ 部分支持（仅声明解析） ❌ 不支持

**核心限制**：v1 基于头文件解析（语法分析），无法触发 C++ 编译器的**语义分析阶段**，因此：
- 模板实例化（`std::vector<int>`）不会发生
- 虚函数表绑定在语义分析阶段完成
- 运行时类型信息（RTTI）无法获取

## 4. 功能需求

### 4.1 核心功能

#### F1: C++ 头文件解析
- 使用 libclang 解析 C++ 头文件
- 支持的 C++ 特性（见上表 v1 支持列）
- **不支持**：模板实例化、Lambda、运算符重载、友元函数、typeid

#### F2: Rust FFI 代码生成
- 为每个 C++ 类生成对应的 `import_class!` 宏调用
- 为每个函数生成对应的 `import_lib!` 宏调用
- 处理命名空间映射（`::` → `_` 或模块嵌套）
- 处理 opaque pointer 模式（嵌套命名空间类使用 void*）

#### F3: 项目脚手架生成
- 生成 `rust_hicc/build.rs`
- 生成 `rust_hicc/Cargo.toml`
- 生成 `rust_hicc/src/main.rs`

### 4.2 CLI 接口

```bash
# 基本用法
cpp2rust-ffi --input ./cpp --output ./rust_hicc

# 指定头文件
cpp2rust-ffi -i ./example.h -o ./rust

# 启用详细输出
cpp2rust-ffi -i ./cpp -o ./rust --verbose

# 指定库名
cpp2rust-ffi -i ./cpp -o ./rust --lib-name "mylib"
```

### 4.3 输出示例

**输入** (`cpp/example.h`):
```cpp
#pragma once

namespace foo {
    namespace bar {
        class ConfigManager {
        public:
            ConfigManager();
            ~ConfigManager();
            void set_value(const char* key, int value);
            int get_value(const char* key) const;
        };
    }
}
```

**输出** (`rust_hicc/src/main.rs`):
```rust
hicc::cpp! {
    #include "example.h"

    namespace foo { namespace bar {

    class ConfigManager {
    public:
        ConfigManager();
        ~ConfigManager();
        void set_value(const char* key, int value);
        int get_value(const char* key) const;
    };

    }}
}

type ConfigManager = *mut std::ffi::c_void;

#[link(name = "example")]
unsafe extern "C" {
    fn config_manager_new() -> ConfigManager;
    fn config_manager_delete(p: ConfigManager);
    fn config_manager_set_value(p: ConfigManager, key: *const i8, value: i32);
    fn config_manager_get_value(p: ConfigManager, key: *const i8) -> i32;
}
```

## 5. 实现计划

### 5.1 阶段划分

| 阶段 | 内容 | 时间 | 优先级 |
|------|------|------|--------|
| **Phase 1** | 项目脚手架 + CLI 入口 | 1 周 | P0 |
| **Phase 2** | Clang 解析器封装 | 1 周 | P0 |
| **Phase 3** | 类型提取（类/函数/枚举） | 1 周 | P0 |
| **Phase 4** | Rust FFI 代码生成器 | 1 周 | P0 |
| **Phase 5** | 项目模板生成 | 1 周 | P0 |
| **Phase 6** | 集成测试 + 48 示例验证 | 1 周 | P1 |

### 5.2 Phase 1-5 详细设计

#### Phase 1: 项目脚手架

**目标**：建立项目结构，实现 CLI 入口

**文件结构**：
```
cpp2rust-ffi/
├── Cargo.toml
├── src/
│   ├── main.rs          # CLI 解析
│   ├── lib.rs          # 库入口
│   ├── cmd/
│   │   ├── mod.rs
│   │   └── generate.rs # generate 子命令
│   └── error.rs        # 错误处理
└── tests/
    └── integration_test.rs
```

**CLI 框架**：使用 `clap` 或 `structopt`

#### Phase 2: Clang 解析器封装

**目标**：封装 libclang，提供 C++ 头文件解析能力

**核心接口**：
```rust
pub struct ClangParser {
    // 解析指定头文件，返回所有类型和函数
    pub fn parse_header(&self, path: &Path) -> Result<ParseResult>;
}

pub struct ParseResult {
    pub classes: Vec<ClassInfo>,
    pub functions: Vec<FunctionInfo>,
    pub enums: Vec<EnumInfo>,
    pub namespaces: Vec<NamespaceInfo>,
}

pub struct ClassInfo {
    pub name: String,
    pub full_name: String,       // e.g., "foo::bar::ConfigManager"
    pub methods: Vec<MethodInfo>,
    pub is_in_namespace: bool,
    pub namespace_depth: usize,   // 用于判断是否需要 void* 模式
}

pub struct MethodInfo {
    pub name: String,
    pub signature: String,       // e.g., "void (const char*, int)"
    pub is_constructor: bool,
    pub is_destructor: bool,
    pub is_const: bool,
    pub is_virtual: bool,
    pub is_pure_virtual: bool,
    pub is_override: bool,
}
```

**Clang AST 映射**：
| Clang AST 节点 | 解析目标 |
|----------------|----------|
| `CXXRecordDecl` | 类/结构体 |
| `FunctionDecl` | 函数声明 |
| `EnumDecl` | 枚举 |
| `CXXMethodDecl` | 类方法 |
| `NamespaceDecl` | 命名空间 |

#### Phase 3: 类型提取

**类型提取逻辑**：
```rust
impl ClassInfo {
    // 判断是否需要 opaque pointer 模式
    pub fn needs_opaque_pointer(&self) -> bool {
        // 嵌套命名空间深度 >= 2 使用 void*
        self.namespace_depth >= 2
    }
}
```

#### Phase 4: Rust FFI 代码生成

**FFI 代码生成模板**：
```rust
// 对于 foo::bar::ConfigManager 类
// 生成：

// 1. cpp! 块 - 包含类定义
hicc::cpp! {
    namespace foo { namespace bar {
    class ConfigManager { /* ... */ };
    }}
}

// 2. opaque pointer 类型别名
type ConfigManager = *mut std::ffi::c_void;

// 3. extern "C" 函数声明
#[link(name = "libname")]
unsafe extern "C" {
    fn config_manager_new() -> ConfigManager;
    fn config_manager_delete(p: ConfigManager);
    fn config_manager_set_value(p: ConfigManager, key: *const i8, value: i32) -> i32;
}
```

#### Phase 5: 项目模板生成

**项目模板变量**：
```rust
pub struct ProjectTemplate {
    pub lib_name: String,        // 库名称
    pub module_name: String,      // Rust 模块名
    pub cpp_files: Vec<String>,  // C++ 源文件列表
    pub cpp_includes: Vec<String>, // #include 内容
}
```

## 6. 技术选型

### 6.1 核心依赖

| 依赖 | 用途 | 版本 |
|------|------|------|
| `clap` | CLI 参数解析 | 4.x |
| `anyhow` | 错误处理 | 1.x |
| `clang-rs` | libclang Rust 绑定 | 0.1.x |
| `tempfile` | 临时文件处理 | 3.x |
| `serde` | 序列化 | 1.x |
| `serde_json` | JSON 输出 | 1.x |

### 6.2 外部依赖

| 依赖 | 安装方式 | 用途 |
|------|----------|------|
| `libclang` | 系统包管理器 | Clang AST 解析 |
| `clang` | 系统包管理器 | C++ 编译器 |

**安装命令**：
```bash
# Ubuntu/Debian
apt-get install clang libclang-dev

# macOS
brew install llvm

# Arch Linux
pacman -S clang
```

## 7. 测试计划

### 7.1 单元测试

| 测试项 | 覆盖内容 |
|--------|----------|
| `parser_tests` | Clang 解析各种 C++ 特性 |
| `generator_tests` | 代码生成逻辑 |
| `template_tests` | 模板渲染 |

### 7.2 集成测试

使用现有的 48 个 examples 作为测试集：

```bash
# 对每个 example 运行自动化工具
for dir in examples/*/; do
    cpp2rust-ffi -i "$dir/cpp" -o /tmp/rust_output
    # 对比生成的代码与现有代码
    diff -r "$dir/rust_hicc" /tmp/rust_output
done
```

### 7.3 验收标准

1. **编译通过**：生成的 rust_hicc 项目可以 `cargo build`
2. **运行正确**：生成的代码运行结果与手动编写一致
3. **覆盖完整**：48 个示例中至少 90% 可以自动化生成

## 8. 已知限制

### 8.1 v1 不支持的特性

| 特性 | 说明 | 预计版本 |
|------|------|----------|
| 模板实例化 | `std::vector<int>` 等 | v2 |
| 运算符重载 | `operator+` 等 | v3 |
| 友元函数 | `FriendDecl` | v3 |
| typeid/RTTI | 运行时类型识别 | v3 |
| Lambda 表达式 | 匿名函数对象 | v2 |
| 虚函数表映射 | 抽象类到 trait | v2 |

### 8.2 部分支持的特性

| 特性 | v1 支持程度 | 说明 |
|------|-------------|------|
| 模板类声明 | ⚠️ | 可解析声明，但无法实例化 |
| 纯虚函数 | ⚠️ | 可解析，但无法生成 Rust trait |
| STL 容器 | ⚠️ | 只能解析为 opaque pointer |

### 8.3 临时解决方案

对于 v1 不支持的特性，工具应：
1. 生成注释标注 `# // TODO: v2 support`
2. 提供 fallback 到 `#[link(name = "...")]` 手动 extern 声明

**示例**：
```rust
// TODO: v2 support - 模板实例化
hicc::cpp! {
    #include <vector>
    typedef std::vector<int> IntVector; // TODO: 需要手动实例化
}

type IntVector = *mut std::ffi::c_void; // Fallback: opaque pointer
```

## 9. 未来扩展

### 9.1 v2 计划（见 `v2/automated-cpp2rust-ffi-v2.md`）

1. **AST 编译捕获**：通过 libclang 编译源文件，捕获模板实例化
2. **STL 容器支持**：自动识别并生成 hicc-std 包装
3. **虚函数表映射**：支持抽象类到 Rust trait

### 9.2 v3 计划

1. **运算符重载**：解析 `operator+` 等，映射到 Rust trait
2. **友元函数**：支持 `FriendDecl`
3. **typeid/RTTI**：运行时类型识别

## 10. 参考资料

- [c2rust-demo 工具](../references/c2rust-demo.md) - 自动化构建捕获流程参考
- [hicc 框架](../references/hicc.md) - FFI 宏使用手册
- [bindgen 文档](https://rust-lang.github.io/rust-bindgen/) - 类型解析参考
- [libclang 文档](https://clang.llvm.org/doxygen/group__CINDEX.html) - AST 解析 API
- [Clang AST 解析示例](./v1/clang-ast-examples.md) - 具体 AST 节点结构示例

## 11. 总结

本计划书描述了一个基于 Clang 解析 + hicc 宏生成的 C++ 到 Rust FFI 自动化工具 v1 版本。

**核心价值**：
1. **减少重复工作**：自动生成 rust_hicc 项目结构
2. **保持一致性**：自动化生成的代码风格统一
3. **易于扩展**：模块化设计便于后续添加新特性

**v1 覆盖范围**：
- 48 个示例中约 40 个可以完全自动化
- 约 8 个需要 v2 支持模板实例化
- 约 4 个需要 v3 支持运算符重载等

**下一步**：参见 `v2/automated-cpp2rust-ffi-v2.md` 了解 v2 计划。
