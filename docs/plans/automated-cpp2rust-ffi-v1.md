# C++ 到 Rust Safe FFI 自动化工具 - v1 计划

## 1. 背景与目标

### 1.1 当前状态

当前 `examples/` 目录下的 48 个示例采用**手动编写**的方式实现 C++ 到 Rust FFI：

```
examples/043_namespace_nested/
├── cpp/
│   ├── namespace_nested.h        # C++ 头文件
│   └── namespace_nested.cpp      # C++ 实现
└── rust_hicc/
    ├── build.rs                  # 手动编写
    ├── Cargo.toml                # 手动编写
    └── src/main.rs               # 手动编写 hicc 宏
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
│   ├── generator/           # Rust 代码生成
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

## 3. 功能需求

### 3.1 核心功能

#### F1: C++ 头文件解析
- 使用 libclang 解析 C++ 头文件
- 支持的 C++ 特性：
  - [x] 类（class）
  - [x] 命名空间（namespace）
  - [x] 嵌套命名空间
  - [x] 枚举类（enum class）
  - [x] 结构体（struct）
  - [x] 函数声明
  - [x] 静态成员
  - [x] 构造/析构函数
  - [x] 类方法（const 方法）
  - [ ] 模板类（v2）
  - [ ] 虚函数（v2）

#### F2: Rust FFI 代码生成
- 为每个 C++ 类生成对应的 `import_class!` 宏调用
- 为每个函数生成对应的 `import_lib!` 宏调用
- 处理命名空间映射（`::` → `_` 或模块嵌套）
- 处理 opaque pointer 模式（嵌套命名空间类使用 void*）

#### F3: 项目脚手架生成
- 生成 `rust_hicc/build.rs`
- 生成 `rust_hicc/Cargo.toml`
- 生成 `rust_hicc/src/main.rs`

### 3.2 CLI 接口

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

### 3.3 输出示例

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

## 4. 实现计划

### 4.1 阶段划分

| 阶段 | 内容 | 时间 | 优先级 |
|------|------|------|--------|
| **Phase 1** | 项目脚手架 + CLI 入口 | 1 周 | P0 |
| **Phase 2** | Clang 解析器封装 | 1 周 | P0 |
| **Phase 3** | 类型提取（类/函数/枚举） | 1 周 | P0 |
| **Phase 4** | Rust FFI 代码生成器 | 1 周 | P0 |
| **Phase 5** | 项目模板生成 | 1 周 | P0 |
| **Phase 6** | 集成测试 + 48 示例验证 | 1 周 | P1 |

### 4.2 Phase 1 详细设计

**目标**：建立项目结构，实现 CLI 入口

**文件结构**：
```
cpp2rust-ffi/
├── Cargo.toml
├── src/
│   ├── main.rs          # CLI 解析
│   ├── lib.rs           # 库入口
│   ├── cmd/
│   │   ├── mod.rs
│   │   └── generate.rs  # generate 子命令
│   └── error.rs         # 错误处理
└── tests/
    └── integration_test.rs
```

**CLI 框架**：使用 `clap` 或 `structopt`

**关键依赖**：
```toml
[dependencies]
clap = "4"
anyhow = "1"
thiserror = "1"
```

### 4.3 Phase 2 详细设计

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
```

**Clang AST 映射**：
| Clang AST 节点 | JSON kind 值 | 解析目标 |
|----------------|--------------|----------|
| `CXXRecordDecl` | `"CXXRecordDecl"` | 类/结构体 |
| `FunctionDecl` | `"FunctionDecl"` | 函数声明 |
| `EnumDecl` | `"EnumDecl"` | 枚举 |
| `CXXMethodDecl` | `"CXXMethodDecl"` | 类方法 |
| `NamespaceDecl` | `"NamespaceDecl"` | 命名空间 |

### 4.4 Phase 3-4 详细设计

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

### 4.5 Phase 5 详细设计

**项目模板变量**：
```rust
pub struct ProjectTemplate {
    pub lib_name: String,        // 库名称
    pub module_name: String,     // Rust 模块名
    pub cpp_files: Vec<String>,  // C++ 源文件列表
    pub cpp_includes: Vec<String>, // #include 内容
}
```

## 5. 技术选型

### 5.1 核心依赖

| 依赖 | 用途 | 版本 |
|------|------|------|
| `clap` | CLI 参数解析 | 4.x |
| `anyhow` | 错误处理 | 1.x |
| `clang-rs` | libclang Rust 绑定 | 0.1.x |
| `tempfile` | 临时文件处理 | 3.x |
| `serde` | 序列化 | 1.x |
| `serde_json` | JSON 输出 | 1.x |

### 5.2 外部依赖

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

## 6. 测试计划

### 6.1 单元测试

| 测试项 | 覆盖内容 |
|--------|----------|
| `parser_tests` | Clang 解析各种 C++ 特性 |
| `generator_tests` | 代码生成逻辑 |
| `template_tests` | 模板渲染 |

### 6.2 集成测试

使用现有的 48 个 examples 作为测试集：

```bash
# 对每个 example 运行自动化工具
for dir in examples/*/; do
    cpp2rust-ffi -i "$dir/cpp" -o /tmp/rust_output
    # 对比生成的代码与现有代码
    diff -r "$dir/rust_hicc" /tmp/rust_output
done
```

### 6.3 验收标准

1. **编译通过**：生成的 rust_hicc 项目可以 `cargo build`
2. **运行正确**：生成的代码运行结果与手动编写一致
3. **覆盖完整**：48 个示例中至少 90% 可以自动化生成

## 7. 已知限制

### 7.1 v1 不支持的特性

| 特性 | 说明 | 预计版本 |
|------|------|----------|
| 模板类 | `template<typename T>` | v2 |
| 虚函数 | 纯虚函数/抽象类 | v2 |
| 继承 | 类继承关系 | v2 |
| 运算符重载 | `operator+` 等 | v2 |
| STL 容器 | `std::vector` 等 | v2 |
| 智能指针 | `std::unique_ptr` 等 | v2 |

### 7.2 临时解决方案

对于 v1 不支持的特性，工具应：
1. 生成注释标注 `# // TODO: v2 support`
2. 提供 fallback 到 `#[link(name = "...")]` 手动 extern 声明

## 8. 未来扩展（v2）

1. **模板支持**：解析和生成模板类
2. **虚函数表**：支持抽象类和接口
3. **继承关系**：映射到 Rust trait
4. **STL 容器**：集成 hicc-std
5. **智能指针**：生成 hicc::unique_ptr 等包装

## 9. 参考资料

- [c2rust-demo 工具](../references/c2rust-demo.md) - 自动化构建捕获流程参考
- [hicc 框架](../references/hicc.md) - FFI 宏使用手册
- [bindgen 文档](https://rust-lang.github.io/rust-bindgen/) - 类型解析参考
- [libclang 文档](https://clang.llvm.org/doxygen/group__CINDEX.html) - AST 解析 API
- [Clang AST 解析示例](./v1/clang-ast-examples.md) - 具体 AST 节点结构示例

## 10. 总结

本计划书描述了一个基于 Clang 解析 + hicc 宏生成的 C++ 到 Rust FFI 自动化工具 v1 版本。核心价值：

1. **减少重复工作**：自动生成 rust_hicc 项目结构
2. **保持一致性**：自动化生成的代码风格统一
3. **易于扩展**：模块化设计便于后续添加新特性

v1 版本覆盖 48 个示例中 90%+ 的场景（约 43 个），为后续版本奠定基础。
