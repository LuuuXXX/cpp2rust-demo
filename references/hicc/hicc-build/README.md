# hicc-build

## 简介

`hicc-build` 是 hicc 生态中的**构建辅助库**，用于在 `build.rs` 中自动解析 Rust 源文件中的 hicc 宏（`hicc::cpp!`、`hicc::import_lib!`、`hicc::import_class!`），并生成对应的 C++ 适配代码，最终将其编译为静态库供 Rust 链接。

开发者无需手动编写 C++ 适配代码——只需在 Rust 文件中使用 hicc 宏声明 C++ 接口，`hicc-build` 会在构建时自动完成所有 C++ 代码的生成与编译。

## 核心功能

### `Build` 结构体

`hicc_build::Build` 是主入口，封装了 `cc::Build` 并添加了 hicc 专有的代码生成逻辑：

```rust
pub struct Build {
    build: cc::Build,
}
```

实现了 `Deref<Target = cc::Build>`，可以直接调用 `cc::Build` 的所有方法（如 `include()`、`flag()` 等）。

### 主要方法

#### `rust_file(src: P) -> &mut Self`

解析指定 Rust 文件中的 hicc 宏，生成 C++ 适配代码并添加到编译列表：

```rust
hicc_build::Build::new()
    .rust_file("src/main.rs")
    .rust_file("src/module.rs")  // 支持多个文件
    .compile("example");
```

**处理流程**：
1. 读取 Rust 源文件并用 `syn` 解析为 AST
2. 遍历所有宏调用，识别 `hicc::import_class!`、`hicc::import_lib!`、`hicc::cpp!`
3. 调用 `hicc-autogen` 将每个宏转换为对应的 C++ 代码
4. 将生成的 C++ 代码写入 `$OUT_DIR/<文件名>.cpp`
5. 将该 `.cpp` 文件添加到 `cc::Build` 的编译列表

#### `cpp_header(src: P, hdr: P) -> &mut Self`

将 hicc 宏转换为 C++ 头文件（不编译），供其他 C++ 文件 `#include`：

```rust
hicc_build::Build::new()
    .cpp_header("src/interface.rs", "include/interface.hpp")
    .rust_file("src/main.rs")
    .compile("example");
```

## 使用方式

### 基本用法

在项目的 `build.rs` 中：

```rust
fn main() {
    hicc_build::Build::new()
        .rust_file("src/main.rs")
        .compile("example");

    println!("cargo::rustc-link-lib=example");
    println!("cargo::rustc-link-lib=stdc++");
    println!("cargo::rerun-if-changed=src/main.rs");
}
```

### 多文件项目

```rust
fn main() {
    hicc_build::Build::new()
        .rust_file("src/ffi/types.rs")
        .rust_file("src/ffi/methods.rs")
        .compile("mylib_ffi");

    println!("cargo::rustc-link-lib=mylib_ffi");
    println!("cargo::rustc-link-lib=stdc++");
    println!("cargo::rerun-if-changed=src/ffi/types.rs");
    println!("cargo::rerun-if-changed=src/ffi/methods.rs");
}
```

### 添加额外的 C++ 源文件或包含路径

由于 `Build` 实现了 `DerefMut<Target = cc::Build>`，可以直接调用 `cc::Build` 的方法：

```rust
fn main() {
    hicc_build::Build::new()
        .include("third_party/include")
        .file("src/cpp/helper.cpp")   // 额外的 C++ 源文件
        .flag("-std=c++17")
        .rust_file("src/main.rs")
        .compile("example");

    println!("cargo::rustc-link-lib=example");
    println!("cargo::rustc-link-lib=stdc++");
}
```

### 生成头文件（供 C++ 侧使用）

```rust
fn main() {
    hicc_build::Build::new()
        .cpp_header("src/api.rs", "generated/api.hpp")
        .rust_file("src/main.rs")
        .compile("example");
}
```

## 自动包含路径

`hicc-build` 在初始化时（`init()`）自动配置以下包含路径：

- `DEP_HICC_INCLUDE`：hicc 核心库的 C++ 头文件目录（由 hicc 的 `build.rs` 通过 `cargo:include=` 注入）
- `DEP_HICC_STD_INCLUDE`：hicc-std 的 C++ 头文件目录（如果项目依赖 hicc-std）
- `.`：当前目录

因此，使用者在 C++ 代码中可以直接 `#include <hicc/hicc.hpp>` 而无需手动配置路径。

## 生成代码位置

生成的 C++ 适配文件位于 `$OUT_DIR`（Cargo 构建输出目录）：

```text
$OUT_DIR/
└── src-main.rs.cpp    # 由 src/main.rs 生成的 C++ 适配代码
```

文件名由 Rust 源文件路径转换而来（将 `/`、`\` 替换为 `-`，并追加 `.cpp`）。

## 支持的宏类型

| 宏 | 处理逻辑 |
|-----|---------|
| `hicc::import_class!` / `import_class!` | 生成 C++ 类适配代码（方法包装等） |
| `hicc::import_lib!` / `import_lib!` | 生成 C++ 全局函数适配代码 |
| `hicc::cpp!` / `cpp!` | 将 C++ 代码块直接输出到生成文件 |
| 其他宏 | 忽略 |

## 依赖关系

- `hicc-autogen`：代码生成逻辑（宏解析 → C++ 代码）
- `cc`：C++ 编译驱动（`cc::Build`）
- `syn`：Rust 源文件解析

## 注意事项

1. **C++ 标准**：生成的 C++ 代码依赖 C++11 或更高版本，请确保编译器支持
2. **`cargo::rerun-if-changed`**：需手动添加 `rerun-if-changed`，否则修改 Rust 源文件后不会重新生成 C++ 代码
3. **链接 C++ 标准库**：必须在 `build.rs` 中显式链接 `stdc++`（Linux）或 `c++`（macOS）
4. **Windows 路径**：`normalize_windows_path` 工具函数处理 Windows 的 `\\?\` 扩展路径前缀
