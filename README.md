# cpp2rust-demo

`cpp2rust-demo` 通过 `LD_PRELOAD` 钩子捕获 C++ 构建过程，为每个翻译单元导出 Clang AST JSON，并在 `.cpp2rust/<feature>/` 目录下生成基于 hicc 的 Rust FFI 脚手架代码。

## 安装

```bash
cargo build --release
# 二进制文件：target/release/cpp2rust-demo
```

**环境要求**：Linux、`g++`/`clang++`、`make`，以及支持 JSON AST 导出的 `clang++`。

## 使用方法

```bash
# 捕获构建过程并逐翻译单元生成模块
cpp2rust-demo init --feature demo -- sh -c "make clean && make"

# 将重复声明合并到第二棵源码树
cpp2rust-demo merge --feature demo
```

生成的目录结构：

```text
.cpp2rust/<feature>/
├── ast/          # 每个翻译单元的 Clang AST JSON
├── meta/         # 运算符垫片等辅助文件
└── rust/
    ├── Cargo.toml
    ├── build.rs
    └── src/      # 生成的 Rust FFI 脚手架
```

## 工作流程

```text
C++ 项目目录
   │
   ├─ cpp2rust-demo init --feature <name> -- <构建命令>
   │    ├─ 编译 hook/libhook.so（LD_PRELOAD 钩子）
   │    ├─ 注入构建过程，为每个翻译单元捕获 AST JSON
   │    ├─ 解析 AST，生成 hicc 脚手架（import_class! / import_lib!）
   │    └─ 输出 .cpp2rust/<feature>/rust/
   │
   └─ cpp2rust-demo merge --feature <name>
        ├─ 合并跨翻译单元的重复声明
        ├─ 生成统一的 lib.rs 入口
        └─ 生成 build.rs 及 operator_shims.hpp
```

## 支持的 C++ 特性

| # | 特性 | 生成策略 |
|---|---|---|
| 01 | 基础类型 | 原生类型映射 + 全局 `import_lib!` |
| 02 | 指针与引用 | 指针/引用启发式映射为 Rust 引用/裸指针 |
| 03 | 基础类 | `import_class!` + 构造函数工厂 |
| 04 | 继承 | 基类列表渲染为 `class Derived: Base` |
| 05 | 虚函数与多态 | 纯虚函数 => `#[interface]` trait |
| 06 | 运算符重载 | 在 `hicc::cpp!` 中生成垫片辅助函数 |
| 07 | 函数模板 | 仅处理显式实例化的 `FunctionDecl` |
| 08 | 类模板 | 处理显式实例化/特化的 record 声明 |
| 09 | 命名空间 | 保留完整限定名 |
| 10 | STL 容器 | 生成 `hicc_std` 提示注释 |
| 11 | 智能指针 | `unique_ptr`/`shared_ptr` 映射 |
| 12 | 移动语义 | `&&` 方法映射为 `self` 接收者 |
| 13 | Lambda/函数式 | `std::function` -> `hicc::Function` |
| 14 | 类型转换 | 生成 dynamic-cast 辅助提示 |
| 15 | 异常处理 | 可能抛出的方法返回 `hicc::Exception<T>` |
| 16 | 静态成员 | 静态方法生成到 `import_lib!` 中 |
| 17 | 友元函数 | 友元声明作为全局函数处理 |
| 18 | const 正确性 | `&self` vs `&mut self` 精确映射 |
| 19 | 内存管理 | 生成 placement-new 辅助提示 |
| 20 | 模板特化 | 显式特化的 record/函数在有声明时保留 |

## 已知限制

| 领域 | 限制 | 原因 |
|---|---|---|
| 模板 | 深度元编程不自动展开 | AST 驱动的脚手架只生成显式声明 |
| 容器 | 复杂 STL API 可能需要手动编写 `hicc_std` 封装 | 安全的生命周期和迭代器需要领域审查 |
| RTTI | dynamic-cast 辅助只是提示，非完整合成 | 运行时意图无法总是从声明中恢复 |
| 异常 | throw 检测是启发式的（检查源码范围中的 `throw`） | JSON AST 不直接暴露高级别的 Rust 策略 |
| 运算符 | 生成的垫片只覆盖常见运算符 | 重载语法需要逐签名适配 |
| 构建系统 | 仅支持 Linux/`LD_PRELOAD` | 钩子依赖 Unix 预加载语义 |

## 参考项目

| 项目 | 说明 | 路径 |
|------|------|------|
| hicc | C++ 转 Rust FFI 核心库（含 hicc-build、hicc-std 等子库） | `references/hicc/` |
| c2rust-demo | 面向 C 项目的构建捕获与脚手架生成工具 | `references/c2rust-demo/` |
| rapidjson | 高性能 C++ JSON 解析库（用于验证 AST 解析能力） | `references/rapidjson/` |

## 示例

`examples/` 目录包含 20 个覆盖主要 C++ 特性的示例，每个示例包含：
- `main.cpp`：演示特性的 C++ 源码
- `ast.json`：Clang AST JSON 导出
- `Makefile`：C++ 编译配置
- `README.md`：特性说明与 hicc 处理方式

详见 [examples/README.md](examples/README.md)。

## 验证

```bash
cargo build
cargo test
bash scripts/validate-rapidjson.sh
```
