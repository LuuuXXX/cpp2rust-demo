# cpp2rust-demo

**C++ → Rust Safe FFI 自动化脚手架生成工具**。给定任意 C++ 项目，执行两条命令即可生成基于 [hicc](https://crates.io/crates/hicc) 的 Rust FFI 绑定层（`hicc::cpp!` / `hicc::import_class!` / `hicc::import_lib!` 三段式），并整理为可直接使用的 Rust 项目结构。

```bash
cpp2rust-demo init -- make -j4   # 捕获构建 + 生成 FFI 脚手架
cpp2rust-demo merge              # 备份并整理编译单元输出（可选）
```

> **工具定位**：cpp2rust-demo 负责生成 FFI **脚手架**（绑定声明 + 必要 C 桥接 shim），不处理业务逻辑、不重写 C++ 代码。生成产物开箱即可 `cargo check`，部分降级特性需人工补全后才能完整编译运行。

仓库同时包含 **48 个循序渐进的 C++ 特性示例**，每个示例都有对应的 C++ 源码和可运行的 Rust FFI 参考实现，覆盖从基础函数到模板、STL、虚继承等复杂场景。

**导航**：[工作原理](#工作原理) · [命令参考](#命令参考) · [快速开始](#快速开始) · [生成代码格式](#生成代码格式三段式) · [特性矩阵](#c-特性支持矩阵) · [降级特性](#降级特性详解6-项) · [测试体系](#测试体系) · [局限性](#局限性) · [未来计划](#未来开发计划) · [学习路径](#学习路径示例索引)

---

## 工作原理

工具通过 **三阶段流水线** 将 C++ 代码转换为 Rust FFI 脚手架：

```
┌──────────────────────────────────┐
│  阶段 1：编译拦截（hook.cpp）      │
│  LD_PRELOAD 注入 → g++ -E -C     │
│  → .cpp2rust（宏展开后的 C++ 代码）│
└──────────────┬───────────────────┘
               ↓
┌──────────────────────────────────┐
│  阶段 2：AST 提取                 │
│  libclang 解析 .cpp2rust         │
│  → CppAst（类/函数/枚举/模板）    │
│  → FfiSpec IR（extractor/）      │
└──────────────┬───────────────────┘
               ↓
┌──────────────────────────────────┐
│  阶段 3：代码生成（generator/）   │
│  FfiSpec → hicc 三段式 Rust 代码 │
│  → .cpp2rust/<feature>/rust/     │
│     lib.rs + <unit>.rs           │
└──────────────────────────────────┘
```

### 阶段 1：编译拦截（hook.cpp / LD_PRELOAD）

**为什么使用 LD_PRELOAD？**  
C++ 构建系统（Make、CMake 等）启动的 `g++` 进程繁多且命令行各异，`LD_PRELOAD` 是在不修改构建脚本的前提下拦截所有 `g++` 调用的最轻量方式。`hook.cpp` 编译为 `libhook.so`，在进程启动时自动注入，劫持 `execve`/`execvp` 等系统调用。

**`g++ -E -C` 的作用**  
hook 拦截到 `g++` 编译调用后，将其替换为等价的 **预处理调用**（`g++ -E -C`）：
- `-E`：只执行预处理，不进行编译、汇编或链接；所有 `#include` 内容被展开写入输出文件；宏调用被展开为实际 C++ 代码。
- `-C`：保留注释（包括文档注释），使后续 AST 解析能读取到内联函数体内的注释。

输出文件以 `.cpp2rust` 为扩展名（非标准，避免与正常构建产物冲突），内容是**宏完全展开、#include 全部内联**后的单文件 C++ 代码，是阶段 2 的输入。

**关键约束**：`hook.cpp` 源码已内嵌进工具 binary（`include_str!`），首次运行时自动解压到用户数据目录并编译，后续自动跳过重编译。生成的 Rust 项目本身（L2 编译 / L3 运行 / L5 符号验证）已在 Windows MSVC 和 MinGW 环境通过 CI 验证。Linux/macOS 依赖 `LD_PRELOAD` / `DYLD_INSERT_LIBRARIES`；Windows 通过 PATH 注入 `hook_shim.exe`（`hook/hook_shim.rs`）实现等价的编译拦截（同时支持 GNU/MinGW + MSVC 两种编译器）。

---

### 阶段 2：AST 提取（extractor/）

**libclang 解析预处理文件**  
`.cpp2rust` 文件本质是合法的 C++ 代码（已展开宏），工具通过 `libclang` crate 以 `-xc++ -std=c++17` 参数解析它，获得完整的 AST。

**行标记扫描（来源区分）**  
`g++ -E` 输出中每段来自不同文件的内容前都有 `# <line> "<file>"` 行标记，工具扫描这些标记，把哪些字节偏移范围属于**当前 `.cpp` 文件自身**（而非被 `#include` 引入的头文件）记录为 `cpp_byte_ranges`。AST 提取时只纳入来自当前文件（`is_from_current_file`）或显式 `extern "C"` 的函数/类，过滤掉第三方头文件中的符号。

**模板实例化的特殊处理**  
C++ 模板在 AST 中同时存在模板声明（`ClassTemplate`）和实例化节点（`ClassTemplateSpecialization`）。工具**只处理 `ClassTemplateSpecialization`**，即已经具有具体类型参数的实例化结果（如 `std::vector<int>`），跳过裸模板声明——这是工具能处理 STL 容器等复杂模板的根本原因。模板声明的实际类型参数在实例化后才可知，无法在编译期枚举，因此"实例化优先"是唯一可行的 FFI 提取策略。

**同名函数去重**  
同一函数可能在头文件和 `.cpp` 中各出现一次，工具对同名函数按以下规则去重：有函数体（`body_offset` 有值）的定义优先；否则非 `extern "C"` 版本优先，避免重复生成绑定。

**提取结果（CppAst）**

| 字段 | 含义 |
|------|------|
| `classes` | 类/结构体（含模板特化）|
| `functions` | 全局函数（含 `extern "C"` 和友元函数）|
| `enums` | 枚举和 `enum class` |
| `typedefs` | `typedef` 声明 |
| `template_class_ranges` | 模板类的源码范围（用于生成 `cpp!` 块内嵌）|

---

### FfiSpec 中间表示（IR）

`CppAst` 经由 `extractor/mod.rs` 转换为 **`FfiSpec`**，这是连接阶段 2 与阶段 3 的核心 IR。`FfiSpec` 不依赖 libclang 数据结构，描述"需要生成哪些 Rust 代码"，而不是"C++ 代码长什么样"：

```
FfiSpec
├── cpp_block_lines   ── hicc::cpp! 块内容（include 指令 + 必要 C++ shim）
├── class_specs[]     ── 每个类对应一个 ClassSpec
│   ├── methods[]     ──   成员方法（→ import_class! 块）
│   ├── associated_fns[] ─ 关联函数（ctor/dtor/factory → import_lib! 的 class 段）
│   └── destroy_fn    ──   析构 shim 名（→ #[cpp(class=..., destroy=...)]）
└── lib_spec          ── import_lib! 块整体
    ├── fwd_decls[]   ──   类前向声明
    └── fn_bindings[] ──   全局/自由函数绑定
```

每个 `MethodBinding` / `FnBinding` 均携带：C++ 原始签名（写入 `#[cpp(method/func = "...")]` 属性）、Rust 函数名（snake_case）、参数列表、返回类型、是否需要 `unsafe`。

---

### 阶段 3：代码生成（generator/）

**三段式输出**对应 hicc 的三个宏：

| 宏 | 来源 | 作用 |
|----|------|------|
| `hicc::cpp!` | `FfiSpec::cpp_block_lines` | 内联 C++ 代码（#include、必要 shim 函数体） |
| `hicc::import_class!` | `FfiSpec::class_specs` | 每个类的成员方法绑定（hicc 处理虚表 dispatch）|
| `hicc::import_lib!` | `FfiSpec::lib_spec` | 全局函数及 ctor/dtor/关联函数绑定 |

---

### 类型映射规则（C++ → Rust）

`extractor/type_mapper.rs` 中的 `cpp_to_rust()` 函数按以下规则将 libclang 返回的 C++ 类型字符串映射为 Rust FFI 类型：

| C++ 类型 | Rust 类型 | 说明 |
|---------|----------|------|
| `void` | `()` | 无返回值 |
| `bool` / `_Bool` | `bool` | |
| `char` / `signed char` | `i8` | C `char` 在 Rust FFI 中为 `i8` |
| `unsigned char` | `u8` | |
| `short` | `i16` | |
| `unsigned short` | `u16` | |
| `int` | `i32` | |
| `unsigned int` | `u32` | |
| `long` / `long long` | `i64` | Linux x86-64：`long` = 64 位 |
| `unsigned long` / `unsigned long long` | `u64` | |
| `float` | `f32` | |
| `double` / `long double` | `f64` | |
| `size_t` | `usize` | |
| `ptrdiff_t` / `intptr_t` | `isize` | |
| `int8_t` … `int64_t` | `i8` … `i64` | `<stdint.h>` 定长整型 |
| `uint8_t` … `uint64_t` | `u8` … `u64` | |
| `const char *` | `*const i8` | C 字符串 |
| `char *` | `*mut i8` | 可变 C 字符串 |
| `const T *` | `*const T_rust` | 指向 const 的指针 |
| `T *` | `*mut T_rust` | 可变指针 |
| `void *` | `*mut u8` | 不透明指针 |
| `const void *` | `*const u8` | |
| `T[N]` / `T[]` | `*mut T_rust` | 数组参数退化为指针 |
| `volatile T` 前缀 | 去掉 `volatile` 后递归映射 | Rust 无 `volatile` 概念 |
| `T *const` | `*mut T_rust` | 指针本身 `const` 在 Rust 无对应语义 |
| 未知 C++ 类类型 `T` | `T`（原样保留） | 由 hicc 宏处理 opaque pointer |

**引用类型**（`T&` / `const T&`）目前不做自动映射，由 hicc 的 `import_class!` 宏在方法签名中通过 `&self` / `&mut self` 处理。

**关键理念**：C++ 模板的价值在于**实例化结果**，不在于模板声明。工具只处理实际被实例化的具体类型（如 `std::vector<int>`），跳过未实例化的模板。

---

## 命令参考

工具提供两个子命令，覆盖"捕获 → 生成 → 整理"的完整工作流：

| 子命令 | 作用 | 典型用法 |
|--------|------|---------|
| `init` | 通过 LD_PRELOAD 拦截构建命令，捕获 C++ 预处理文件，解析 AST，生成 hicc 三段式 FFI 脚手架 | `cpp2rust-demo init -- make -j4` |
| `merge` | 将 `init` 生成的编译单元文件整理为按 C++ 目录结构组织的 Rust 项目，备份原始输出；支持多 feature 合并为带 `[features]` 的统一项目 | `cpp2rust-demo merge --feature default` |

### `init` — 捕获构建 + 生成 FFI 脚手架

```bash
# 单文件项目
cpp2rust-demo init -- g++ -shared -fPIC mylib.cpp -o libmylib.so

# Make 项目
cpp2rust-demo init -- make -j4

# 指定 feature 名称（多平台/多配置场景）
cpp2rust-demo init --feature linux_x86   -- make -j4
cpp2rust-demo init --feature arm_embedded -- arm-none-eabi-g++ -shared -fPIC mylib.cpp -o libmylib.so
```

`init` 自动完成以下步骤：

1. 首次运行时将内嵌的 `hook.cpp` 解压到用户数据目录并编译为 `libhook.so`（后续调用自动跳过）
2. 通过 LD_PRELOAD 注入构建过程，捕获 `.cpp2rust` 预处理文件
3. 交互式选择参与转换的文件（非交互/CI 环境自动全选）
4. libclang 解析 AST，提取类 / 函数 / 枚举 / 模板实例化
5. 生成 `.cpp2rust/<feature>/rust/` 下的 hicc Rust 脚手架

**参数说明：**

| 参数 | 必填 | 说明 |
|------|------|------|
| `-- <BUILD_CMD>...` | ✅ | `--` 后面的所有参数作为构建命令传入 |
| `--feature <name>` | ❌ | 构建目标名称（默认 `default`）；多平台构建使用不同名称，结果落在各自独立目录互不干扰 |

### `merge` — 整理输出结构（可选）

```bash
# 整理单个 feature 的输出（维持 C++ 目录结构）
cpp2rust-demo merge
cpp2rust-demo merge --feature linux_x86

# 多 feature 合并为支持 cargo build --features 的统一 Rust 项目
cpp2rust-demo merge --feature linux_x86 --feature arm_embedded
```

`merge` 将 `init` 的扁平输出整理为按 C++ 目录结构组织的 Rust 项目，并提供备份机制：

```
.cpp2rust/<feature>/rust/
    ├── src.1/   ← init 输出原始备份（首次运行时 rename from src）
    ├── src.2/   ← merge 输出（每次运行重写，维持子目录结构）
    └── src      ← symlink → src.2
```

多 feature 合并时，输出到 `.cpp2rust/<feat1>_<feat2>/rust/`，生成含 `[features]` 段的 `Cargo.toml` 和按 feature 条件编译的 `src/lib.rs`、`build.rs`：

```bash
cd .cpp2rust/linux_x86_arm_embedded/rust
cargo build --features linux_x86
cargo build --features arm_embedded
```

**参数说明：**

| 参数 | 必填 | 说明 |
|------|------|------|
| `--feature <name>` | ❌ | 要操作的构建目标（默认 `default`）；**可重复指定**，≥2 个时进入多 feature 合并模式 |

### 环境变量

| 变量 | 说明 |
|------|------|
| `CPP2RUST_CXX` | 覆盖默认 C++ 编译器（默认自动检测 g++/clang++/c++，支持带版本后缀如 g++-13） |
| `CPP2RUST_DEBUG` | 非空时输出 hook 调试日志到 stderr |

---

## 快速开始

### 安装依赖

```bash
# 系统依赖（Ubuntu/Debian）
sudo apt-get install clang libclang-dev g++ libstdc++-14-dev

# 从 GitHub 安装（无需克隆仓库）
cargo install --git https://github.com/LuuuXXX/cpp2rust-demo

# 或从本地源码安装（开发者）
cargo install --path .
```

> **注意**：`hook/hook.cpp` 已内嵌进 binary，无需额外文件。首次执行 `init` 时工具
> 自动将 hook 源码解压到 `~/.local/share/cpp2rust-demo/hook/`（Linux）或
> `~/Library/Application Support/cpp2rust-demo/hook/`（macOS）并编译；后续调用在 hook 库为最新版时自动跳过重编译。

### Step 1 — `init`：捕获构建 + 生成 FFI 脚手架

在目标 C++ 项目根目录执行：

```bash
cd /path/to/my-cpp-project

# 单文件项目
cpp2rust-demo init -- g++ -shared -fPIC mylib.cpp -o libmylib.so

# Make 项目
cpp2rust-demo init -- make -j4

# 指定 feature 名称以区分不同构建目标（如不同平台或构建配置）
# C++ 的 target 对应 Rust 的 feature：每个 feature 保存一次构建命令的产物
cpp2rust-demo init --feature linux_x86 -- make -j4
cpp2rust-demo init --feature arm_embedded -- arm-none-eabi-g++ -shared -fPIC mylib.cpp -o libmylib.so
```

> **多 target 场景**：同一 C++ 项目针对多个平台或构建配置，分别执行 `init` 并指定不同 `--feature`，
> 输出会落在各自独立的目录下，互不干扰：
>
> ```
> .cpp2rust/linux_x86/      ← linux_x86 构建命令对应的 Rust FFI 绑定
> .cpp2rust/arm_embedded/   ← arm_embedded 构建命令对应的 Rust FFI 绑定
> ```
>
> 完成各 feature 的 `init` + `merge` 之后，可通过 `merge --feature linux_x86 --feature arm_embedded`
> 将多个 feature 合并为一个支持 `cargo build --features <feature>` 按需编译的统一 Rust 项目。
> 详见下文 [Step 2b](#step-2b)。


`init` 自动完成：
1. 首次运行时将内嵌的 `hook.cpp` 解压到用户数据目录并编译为 `libhook.so`（后续调用自动跳过）
2. 通过 LD_PRELOAD 注入构建过程，捕获 `.cpp2rust` 预处理文件
3. 交互式选择参与转换的文件（非交互环境自动全选）
4. libclang 解析 AST，提取类/函数/枚举/模板实例化
5. 生成 `.cpp2rust/<feature>/rust/` 下的 hicc Rust 脚手架

输出示例：
```
=== cpp2rust-demo init ===
项目根目录   : /path/to/my-cpp-project
Feature    : default
构建命令   : make -j4
...
已捕获 3 个 .cpp2rust 文件
已为本 feature 选择 3 个文件

正在对选定文件运行 AST 解析与代码生成...
  mylib.cpp.cpp2rust → 2 个类、5 个函数、0 个枚举  [142 ms]

⚠ 降级特性（需要人工处理）：
  [OP] × 2 次
      utils/foo （2 次）
  → 在生成文件中搜索 'cpp2rust-todo' 可定位这些位置。

✓ cpp2rust-demo init 完成。

输出目录结构:
  .cpp2rust/default/
    ├── c/          （捕获的 .cpp2rust 文件，目录结构与 C++ 项目一致）
    ├── meta/       （build_cmd.txt、selected_files.json、init-report.md）
    └── rust/       （生成的 Rust 项目：Cargo.toml、src/lib.rs、src/**/*.rs）

已在 .cpp2rust/default/rust/src/ 生成 3 个单元文件
```

### Step 2 — `merge`：备份并整理编译单元输出（可选）

`merge` 将 `init` 生成的 `src/` 目录原地备份，并将整理后的输出写回同一 feature 目录，完整保留 C++ 项目的目录结构：

```bash
cpp2rust-demo merge --feature default
```

执行后在 `.cpp2rust/<feature>/rust/` 下生成：

```
.cpp2rust/default/rust/
    ├── src.1/   ← init 输出的原始备份（rename from src）
    ├── src.2/   ← merge 输出（维持 C++ 目录结构）
    └── src      ← symlink → src.2
```

- 首次运行：`src/` 重命名为 `src.1/`，输出写入 `src.2/`，建立 `src → src.2` symlink
- 重复运行：`src.1/` 保持不变，仅更新 `src.2/` 并重建 symlink

### Step 2b — `merge`（多 feature 合并）：生成统一 Rust 项目 <a name="step-2b"></a>

当项目拥有多个 feature（如多平台/多配置）时，可以在各 feature 完成单独的 `init` + `merge` 之后，
通过一次 `merge` 调用将它们合并为一个支持 `cargo build --features <feature>` 按需编译的统一 Rust 项目：

```bash
# 对每个 feature 先完成单独的 init + merge
cpp2rust-demo init --feature linux_x86 -- make -j4
cpp2rust-demo merge --feature linux_x86

cpp2rust-demo init --feature arm_embedded -- arm-none-eabi-g++ ...
cpp2rust-demo merge --feature arm_embedded

# 多 feature 合并：输出到 .cpp2rust/linux_x86_arm_embedded/rust/
cpp2rust-demo merge --feature linux_x86 --feature arm_embedded
```

执行后在 `.cpp2rust/<feat1>_<feat2>/rust/` 下生成组合项目：

```
.cpp2rust/linux_x86_arm_embedded/
└── rust/
    ├── Cargo.toml  ← package.name = "linux_x86_arm_embedded"，含 [features] 段
    ├── build.rs    ← 按 feature 条件编译各 feature 的 C++ shim
    └── src/
        ├── lib.rs              ← #[cfg(feature = "...")] pub mod ...;
        ├── linux_x86/          ← linux_x86 的 Rust 源文件
        │   └── mod.rs
        └── arm_embedded/       ← arm_embedded 的 Rust 源文件
            └── mod.rs
```

在生成的项目中按需编译特定 feature：

```bash
cd .cpp2rust/linux_x86_arm_embedded/rust
cargo build --features linux_x86
cargo build --features arm_embedded
cargo build --features linux_x86,arm_embedded
```

> **注意**：多 feature 合并**不影响**原有的单 feature 目录（`.cpp2rust/linux_x86/`、
> `.cpp2rust/arm_embedded/`），它们保持不变，可以继续独立使用。

### Step 3 — 手动完善降级特性

工具在 `init` 终端输出中会列出检测到的降级特性 TAG。参考下表按 TAG 说明手动完善，同时可在需要人工处理的位置手动添加 `// cpp2rust-todo[TAG]` 注释，`merge` 命令会汇总这些标记并统计总数：

| TAG | 原因 | 需手动操作 |
|-----|------|-----------|
| `[OP]` | 运算符重载（C ABI 无运算符符号） | 为生成的命名 shim（`{class}_add` 等）添加 Rust `std::ops::*` trait 实现 |
| `[VA]` | 可变参数模板（编译期展开，FFI 无法表达任意参数数） | 检查 wrapper 类展开的版本数量是否满足需求，按需手动添加新版本 |
| `[LM]` | 有状态 Lambda / std::function（捕获列表不透明） | 若需 Rust 闭包 → C++ 回调，手动编写 trampoline |
| `[CV]` | C 可变参数函数（`...` 参数，FFI 无法表达任意类型） | 在头文件中为所需参数组合补充固定参数 wrapper，再将其加入 `hicc::cpp!` + `import_lib!` |
| `[FP]` | 函数指针参数（`void (*)(...)` 类型，自动映射为 `Option<unsafe extern "C" fn(...)>`，生成 `cpp2rust-todo[FP]` 注释提示安全性要求） | 确认回调符合 `extern "C"` 调用约定；若需 Rust 闭包，手动编写 trampoline |
| `[VM]` | volatile this 成员函数（方法整体从 `import_class!` 移除） | 检查 `import_lib!` 中是否已有对应的 `volatile T*` C shim；如无则手动添加 |

> **完整命令参数说明**见 [命令参考](#命令参考) 章节。

### 进阶：对纯 C++ 库使用 shim 工作流

`cpp2rust-demo` 通过解析 C++ 预处理后的 AST 来提取 `extern "C"` 函数。对于**纯 C++ 库**（例如 rapidjson、Eigen、Abseil），其头文件和源文件中均无 `extern "C"` 声明，直接运行 `init` 只会生成 `hicc::cpp!` 头文件块，**不会生成 `import_lib!` FFI 绑定**。

这是预期行为，不是 bug。正确的做法是先编写一层 **C++ shim 文件**（`extern "C"` 不透明句柄包装层），再对 shim 文件运行 `cpp2rust-demo init`。

#### 推荐工作流

```
纯 C++ 库（如 rapidjson）
        │
        ▼
  ① 编写 C++ shim 文件
     （extern "C" 包装层，暴露必要的 API 为 C 函数）
        │
        ▼
  ② cpp2rust-demo init --feature <name> -- <编译 shim 的命令>
     （工具拦截 g++ 调用，提取 shim 中的 extern-C 函数）
        │
        ▼
  ③ cpp2rust-demo merge --feature <name>
        │
        ▼
  ④ 在生成的 Rust 项目中使用 import_lib! 绑定调用原始 C++ API
```

#### shim 文件示例

```cpp
// document_ffi.h — 暴露为 extern "C" 的不透明句柄 API
#ifdef __cplusplus
extern "C" {
#endif

typedef struct RapidDocument RapidDocument;

RapidDocument* rapid_document_new();
void           rapid_document_delete(RapidDocument* doc);
int            rapid_document_parse(RapidDocument* doc, const char* json);

#ifdef __cplusplus
}
#endif

// document_ffi.cpp — 实现（include header，g++ 编译时 extern-C 来自 header）
#include "document_ffi.h"
#include "rapidjson/document.h"

struct RapidDocument { rapidjson::Document inner; };

RapidDocument* rapid_document_new() { return new RapidDocument{}; }
void rapid_document_delete(RapidDocument* doc) { delete doc; }
int rapid_document_parse(RapidDocument* doc, const char* json) {
    doc->inner.Parse(json);
    return doc->inner.HasParseError() ? -1 : 0;
}
```

#### rapidjson 完整参考实现

本仓库已包含 rapidjson 的完整 shim 参考实现（10 个子系统），位于：

```
references/rapidjson-refactoring/rapidjson_sys/shim/
├── allocator_ffi.cpp / .h
├── document_ffi.cpp / .h
├── pointer_ffi.cpp / .h
├── reader_ffi.cpp / .h
├── stringbuffer_ffi.cpp / .h
├── value_ffi.cpp / .h
└── …（共 10 个子系统）
```

使用本地验证脚本体验完整流程：

```bash
# 自动定位本地 shim 文件并运行完整转换 + 验证
bash usage/verify-rapidjson-ffi.sh
```

> **生成 Cargo.toml 包含 `hicc-std` 依赖**：工具在生成的 `Cargo.toml` 中自动添加
> `hicc-std` 依赖（STL 容器绑定所需的辅助宏），无需手动添加。

---

## 生成代码格式（三段式）

工具输出标准的 hicc 三段式 Rust FFI 代码：

```rust
// ─── 段 1：C++ 实现内联（含必要 shim）───────────────────
hicc::cpp! {
    #include "foo.h"

    // ctor/dtor/operator/placement-new 等必要 shim
    Foo* foo_new(int value) { return new Foo(value); }
    void foo_delete(Foo* self) { delete self; }
}

// ─── 段 2：类方法绑定（每个类独立块）──────────────────────
hicc::import_class! {
    #[cpp(class = "Foo")]
    pub class Foo {
        #[cpp(method = "int getValue() const")]
        fn getValue(&self) -> i32;

        #[cpp(method = "void setValue(int v)")]
        fn setValue(&mut self, v: i32);
    }
}

// ─── 段 3：全局/关联函数绑定 ──────────────────────────────
hicc::import_lib! {
    #![link_name = "foo"]

    class Foo;

    #[cpp(func = "Foo* foo_new(int value)")]
    fn foo_new(value: i32) -> *mut Foo;

    #[cpp(func = "void foo_delete(Foo* self)")]
    unsafe fn foo_delete(self_: *mut Foo);
}
```

**最小 shim 策略**：成员方法直接通过 `import_class!` 绑定（由 hicc 处理虚表 dispatch），只有以下场景才生成 C shim 函数：
- 构造函数 / 析构函数（C 无 `new`/`delete`）
- 静态成员变量 getter/setter
- 运算符重载（C ABI 无运算符符号）
- placement new
- STL 容器 wrapper 类的 ctor/dtor

---

## C++ 特性支持矩阵

> 图例：✅ 完全自动生成可编译代码　⚠️ 降级生成 + 内联 TODO（代码仍可 `cargo check`）

| 示例 | 类别 | C++ 特性 | 状态 | FFI 策略 |
|------|------|---------|------|---------|
| [001_hello_world](examples/001_hello_world) | 基础函数 | extern "C" 函数 | ✅ | AST 直接提取 → `import_lib!` |
| [002_function_overload](examples/002_function_overload) | 基础函数 | 函数重载 | ✅ | 各重载名称加类型后缀区分（`_i32`/`_f64`）→ `import_lib!` |
| [003_default_args](examples/003_default_args) | 基础函数 | 默认参数 | ✅ | C++ 侧展开为多个固定参数重载，写入 `hicc::cpp!` |
| [004_inline_functions](examples/004_inline_functions) | 基础函数 | inline 函数 | ✅ | 函数体从 `.cpp2rust` 提取，内联写入 `hicc::cpp!` |
| [005_variadic_functions](examples/005_variadic_functions) | 基础函数 | C 可变参数（`...`） | ⚠️ `[CV]` | `...` 参数函数整体跳过；头文件中的固定参数 wrapper 直接 `extern "C"` 绑定 |
| [006_class_basic](examples/006_class_basic) | 类与对象 | 基础类 | ✅ | opaque pointer + `import_class!` + ctor/dtor shim |
| [007_class_constructor](examples/007_class_constructor) | 类与对象 | 构造/析构 | ✅ | `*_new()` / `*_delete()` 必要 shim |
| [008_class_copy](examples/008_class_copy) | 类与对象 | 拷贝构造 | ✅ | `*_copy(const Foo*)` 必要 shim |
| [009_class_move](examples/009_class_move) | 类与对象 | 移动构造 | ✅ | `*_move(Foo*)` shim（内部 `std::move()`） |
| [010_class_static](examples/010_class_static) | 类与对象 | 静态成员 | ✅ | `{class}_get_{field}` / `{class}_set_{field}` 必要 shim |
| [011_class_const](examples/011_class_const) | 类与对象 | const 成员函数 | ✅ | 直接 `import_class!`，映射为 `fn method(&self)` |
| [012_class_volatile](examples/012_class_volatile) | 类与对象 | volatile 成员函数 | ⚠️ `[VM]` | `volatile this` 方法从 `import_class!` 中整体移除；`extern "C"` shim（接收 `volatile T*`）仍进入 `import_lib!` |
| [013_inheritance_single](examples/013_inheritance_single) | 面向对象 | 单继承 | ✅ | 基类方法在子类 `import_class!` 中一并提升，无 shim |
| [014_inheritance_multiple](examples/014_inheritance_multiple) | 面向对象 | 多继承 | ✅ | 多条继承链展开，所有方法通过 `import_class!` 直接绑定 |
| [015_virtual_basic](examples/015_virtual_basic) | 面向对象 | 虚函数 | ✅ | opaque pointer 调用，hicc 宏负责虚表 dispatch |
| [016_virtual_pure](examples/016_virtual_pure) | 面向对象 | 纯虚/抽象类 | ✅ | 抽象类只生成前向声明；子类通过 `import_class!` 绑定 |
| [017_virtual_override](examples/017_virtual_override) | 面向对象 | override | ✅ | override 语义透传，与普通虚函数相同 |
| [018_virtual_diamond](examples/018_virtual_diamond) | 面向对象 | 菱形继承（virtual 继承） | ✅ | 为每条继承方法生成命名 shim（`d_getAValue(D*)`），避免指针调整 |
| [019_operator_overload](examples/019_operator_overload) | 运算符/类型 | 运算符重载 | ⚠️ `[OP]` | 自动生成命名 shim（`{class}_add` 等）写入 `hicc::cpp!` + `import_lib!`；Rust `ops::*` trait 需手动实现 |
| [020_friend_function](examples/020_friend_function) | 运算符/类型 | 友元函数 | ✅ | 友元函数提取为普通函数写入 `import_lib!` |
| [021_explicit_ctor](examples/021_explicit_ctor) | 运算符/类型 | explicit 构造函数 | ✅ | `explicit` 对 FFI 透明，与普通构造相同 |
| [022_mutable_member](examples/022_mutable_member) | 运算符/类型 | mutable 成员 | ✅ | `mutable` 对 FFI 透明，直接 `import_class!` |
| [023_typeid_rtti](examples/023_typeid_rtti) | 运算符/类型 | typeid/RTTI/dynamic_cast | ✅ | 注入整数枚举 + 虚函数 `getType()`，完全绕过 `typeid` |
| [024_template_function](examples/024_template_function) | 模板实例化 | 函数模板 | ✅ | 忽略模板声明，为每个实例化版本生成命名 C 包装函数 |
| [025_template_class](examples/025_template_class) | 模板实例化 | 类模板 | ✅ | 只处理实际实例化的具体类型，按普通类生成 |
| [026_template_specialization](examples/026_template_specialization) | 模板实例化 | 模板偏特化 | ✅ | 偏特化视为实例化路径之一，收集通过该路径实例化的类型 |
| [027_template_instantiation](examples/027_template_instantiation) | 模板实例化 | 显式模板实例化 | ✅ | 显式实例化在 AST 中直接可见，按普通类处理 |
| [028_variadic_template](examples/028_variadic_template) | 模板实例化 | 可变参数模板 | ⚠️ `[VA]` | 生成 wrapper 类 + 按参数数量展开的静态方法；超出范围的组合需手动添加 |
| [029_unique_ptr](examples/029_unique_ptr) | 智能指针/内存 | std::unique_ptr | ✅ | opaque pointer；`*_new()` 返回裸指针，调用方 `*_delete()` 释放 |
| [030_shared_ptr](examples/030_shared_ptr) | 智能指针/内存 | std::shared_ptr | ✅ | `*_clone()` shim 增加引用计数，`*_delete()` 减少；其余方法直接绑定 |
| [031_custom_deleter](examples/031_custom_deleter) | 智能指针/内存 | 自定义删除器 | ✅ | 删除器函数注入 `hicc::cpp!`，`*_delete()` shim 内部调用自定义删除器 |
| [032_placement_new](examples/032_placement_new) | 智能指针/内存 | placement new | ✅ | 生成 `*_placement_new(ptr, ...)` 必要 shim |
| [033_raii_pattern](examples/033_raii_pattern) | 智能指针/内存 | RAII 模式 | ✅ | 析构函数生成 `*_delete()` shim，Rust 侧可实现 `Drop` trait |
| [034_vector_basic](examples/034_vector_basic) | STL 容器 | std::vector\<T\> | ✅ | 薄 wrapper 类 `IntVector`（`VectorImpl<int>` 封装）→ `import_class!` |
| [035_map_basic](examples/035_map_basic) | STL 容器 | std::map\<K,V\> | ✅ | 薄 wrapper 类 `StringIntMap`（`MapImpl<string,int>` 封装）→ `import_class!` |
| [036_string_basic](examples/036_string_basic) | STL 容器 | std::string | ✅ | string wrapper，`c_str()`/`length()` 等通过 `import_class!` 绑定 |
| [037_array_basic](examples/037_array_basic) | STL 容器 | std::array\<T,N\> | ✅ | 数组 wrapper（N 在实例化时已知）→ `import_class!` |
| [038_tuple_basic](examples/038_tuple_basic) | STL 容器 | std::tuple\<T...\> | ✅ | tuple wrapper，按位置 `get<N>()` 通过 `import_class!` 绑定 |
| [039_lambda_basic](examples/039_lambda_basic) | 函数对象 | Lambda 表达式 | ⚠️ `[LM]` | 无状态 lambda → 函数指针；有状态 lambda → class wrapper + `call()` 方法；C 函数指针参数自动映射为 `Option<unsafe extern "C" fn(...)>`（加 `[FP]` 注释） |
| [040_std_function](examples/040_std_function) | 函数对象 | std::function\<Sig\> | ⚠️ `[LM]` | 类型擦除容器，统一用 class wrapper + opaque pointer；C 函数指针参数自动映射为 `Option<unsafe extern "C" fn(...)>` |
| [041_functional_bind](examples/041_functional_bind) | 函数对象 | std::bind | ✅ | 产物本质是函数对象，同有状态 lambda 策略，`import_class!` 完全覆盖 |
| [042_exception_basic](examples/042_exception_basic) | 函数对象 | C++ 异常处理 | ✅ | shim 层 `try/catch` 捕获异常，转为错误码 + 错误消息字符串返回 |
| [043_namespace_nested](examples/043_namespace_nested) | 高级特性 | 嵌套命名空间 | ✅ | `void*` opaque pointer + raw `extern "C"` 绑定，函数名前缀扁平化（`foo::bar::Baz` → `foo_bar_baz_*`） |
| [044_enum_class](examples/044_enum_class) | 高级特性 | 强类型枚举（enum class） | ✅ | 枚举值导出为 Rust `const`，建议手动实现 `enum` + `TryFrom<i32>` |
| [045_union_basic](examples/045_union_basic) | 高级特性 | union | ✅ | opaque pointer + 按字段名 getter/setter shim；Rust 侧用 `#[repr(C)] union` |
| [046_constexpr_basic](examples/046_constexpr_basic) | 高级特性 | constexpr 常量/函数 | ✅ | 编译期常量读取 AST `IntegerLiteral` 值，生成 Rust `const`；constexpr 函数按普通函数处理 |
| [047_noexcept_basic](examples/047_noexcept_basic) | 高级特性 | noexcept | ✅ | `noexcept` 语义对 FFI 透明，直接处理 |
| [048_summary](examples/048_summary) | 高级特性 | 综合 FFI 模式 | ✅ | 以上所有策略的组合应用 |

> **STL 容器核心策略**：先在 `hicc::cpp!` 中生成薄 wrapper 类（如 `IntVector` 封装 `std::vector<int>`），再对 wrapper 类做 `import_class!` 绑定，规避模板方法签名复杂度。

---

## 降级特性详解（6 项）

| TAG | 示例 | C++ 特性 | 无法完全自动的根本原因 | 自动降级策略 | 用户剩余工作 |
|-----|------|---------|---------------------|------------|------------|
| `[OP]` | 019 | 运算符重载 | C ABI 无运算符符号；FFI 边界只能传命名函数 | 为每个运算符生成命名 shim（`{class}_add/sub/...`），写入 `hicc::cpp!` + `import_lib!` | 可选：手动实现 `impl std::ops::Add<T> for T` 等 |
| `[VA]` | 028 | 可变参数模板 | `...Args` 是编译期展开，FFI 无法表达"任意数量参数" | 生成 wrapper 类，按参数数量和类型组合分别封装为静态方法（`sum_1`/`sum_2` 等） | 若需要新的参数数量/类型组合，在 `hicc::cpp!` 中手动添加对应方法和包装函数 |
| `[LM]` | 039 | 有状态 Lambda | 匿名闭包类型，FFI 无法表达捕获列表 | 无状态 lambda → 函数指针；有状态 lambda → class wrapper + opaque pointer | 若需 Rust 闭包 → C++ 回调，手动编写 trampoline |
| `[LM]` | 040 | std::function | 类型擦除容器，捕获状态不透明 | 统一使用 class wrapper + opaque pointer | 可选：手动实现 Rust 闭包 → `std::function` 适配层 |
| `[CV]` | 005 | C 可变参数函数 | C 的 `...` 参数在运行时按 `va_list` 访问，Rust FFI 要求精确的静态类型列表，无法表达可变元 | 含 `...` 的函数（`is_variadic = true`）整体跳过，不生成任何绑定 | 在头文件中为每种实际调用的参数组合提供固定参数 wrapper，工具自动绑定这些 wrapper |
| `[FP]` | 039, 040 | 函数指针参数 | C++ 成员函数指针（`int (Cls::*)()` 等）无法映射为 Rust FFI 类型 | C 函数指针（`void (*)(int)` 等）自动映射为 `Option<unsafe extern "C" fn(i32)>`，标记 `is_unsafe = true` 并在绑定前加 `// cpp2rust-todo[FP]:` 注释；C++ 成员函数指针仍整体跳过 | 确认回调符合 `extern "C"` 调用约定；若需 Rust 闭包，手动编写 trampoline |
| `[VM]` | 012 | volatile 成员函数 | hicc 通过方法指针类型进行检查，`volatile this` 修饰的方法指针（`R (T::*)() volatile`）在 Rust 无对应语义，导致类型不匹配 | `is_volatile = true` 的方法从 `import_class!` 中整体移除；`extern "C"` shim 若以 `volatile T*` 为第一参数，则仍进入 `import_lib!` | 检查 `import_lib!` 中是否已有对应 `volatile T*` C shim；若无，在头文件中手动添加 `void foo_read(volatile Foo* self)` 并重新运行 `init` |

### 降级特性代码示例

#### `[OP]` — 运算符重载

**根本原因**：C ABI 没有运算符符号的概念，FFI 边界只能传递命名函数，无法直接映射 `operator+` 等语法。

```cpp
// C++ 运算符重载 —— 无法直接 FFI 导出
class Number {
public:
    Number operator+(const Number& other) const;
    Number operator-(const Number& other) const;
    Number operator*(const Number& other) const;
    Number& operator+=(const Number& other);
    bool operator==(const Number& other) const;
};

// 工具自动生成的命名 shim（写入 hicc::cpp! + import_lib!）
Number number_add(const Number* self, const Number* other);
Number number_sub(const Number* self, const Number* other);
Number number_mul(const Number* self, const Number* other);
Number* number_add_assign(Number* self, const Number* other);
bool number_eq(const Number* self, const Number* other);
```

生成结果：所有 `operator*` 从 `import_class!` 中移除；对应命名 shim 进入 `import_lib!`，可直接在 Rust 侧调用 `number_add(a, b)` 等。

**用户操作**：可选在 Rust 侧手动实现 `impl std::ops::Add<Number> for Number` 等 trait，内部调用自动生成的 `number_add`，恢复 `+` 运算符语法。

---

#### `[VA]` — 可变参数模板

**根本原因**：C++ `...Args` 是编译期展开，FFI 无法在运行时表达"任意数量、任意类型参数"的函数签名。

```cpp
// 可变参数模板 —— 无法直接 FFI 导出
template<typename... Args>
int sum(Args... args);   // ← 模板，编译期展开，跳过

// 工具生成 wrapper 类，按参数数量分别封装为静态方法
class SumWrapper {
public:
    static int sum_1(int a);
    static int sum_2(int a, int b);
    static int sum_3(int a, int b, int c);
    static int sum_4(int a, int b, int c, int d);
    static int sum_5(int a, int b, int c, int d, int e);
};
```

生成结果：原始 `sum<Args...>` 不出现；`sum_1` / `sum_2` / … / `sum_5` 作为静态方法进入 `import_class!`（或对应的 `import_lib!` 自由函数）。

**用户操作**：若需要新的参数数量或类型组合（如 `sum_6` 或 `float` 版本），在 `hicc::cpp!` 中手动添加对应 wrapper 方法，重新运行 `cpp2rust-demo init`。

---

#### `[LM]` — 有状态 Lambda / std::function

**根本原因**：Lambda 是匿名闭包类型，捕获列表信息在 FFI 边界无法表达；`std::function` 对捕获状态做了类型擦除，同样无法直接传递。

```cpp
// 无状态 lambda → 可转为函数指针，工具正常绑定
auto add = [](int a, int b) { return a + b; };     // ← 无捕获，映射为 fn(i32, i32) -> i32

// 有状态 lambda —— 工具生成 class wrapper + opaque pointer [LM]
int offset = 10;
auto add_offset = [offset](int a) { return a + offset; };  // ← 捕获 offset，跳过直接绑定

// 工具自动生成 opaque class wrapper（写入 hicc::cpp!）
class LambdaWrapper {
    std::function<int(int, int)> fn;
public:
    LambdaWrapper(int (*fn_ptr)(int, int));
    int call(int a, int b);
};

// std::function 参数 —— 同样通过 class wrapper 处理
struct Processor {
    std::function<int(int)> callback;
    // 工具生成：processor_set_callback(self, fn_ptr)
    // 工具生成：processor_process(self, value)
};
```

生成结果：无状态 lambda 对应的函数指针参数直接出现在 `import_lib!`；有状态 lambda 和 `std::function` 通过 `LambdaWrapper` / `Processor` 等 opaque class 暴露，在 `import_class!` 中提供 `new`、`call`、`set_callback`、`process` 等命名方法。

**用户操作**：若需要将 Rust 闭包传入 C++ 回调，手动编写 trampoline（将 Rust `fn` 指针 + 用户数据包装为 `extern "C"` 函数），再通过 `set_callback` 注入；或将捕获状态改为全局/线程局部变量，降级为无状态 lambda。

---

#### `[CV]` — C 可变参数函数

**根本原因**：Rust 的 FFI 接口要求每个参数都有精确的静态类型，而 C 的 `...` 只在运行时通过 `va_list` 访问参数，无法在编译期表达参数数量与类型。

```c
// 头文件中的 C 可变参数函数 —— 工具跳过此函数
int sum(int count, ...);            // ← is_variadic=true，整体跳过

// 手动提供的固定参数 wrapper —— 工具正常绑定
int sum_3(int a, int b, int c);
int sum_5(int a, int b, int c, int d, int e);
```

生成结果：`sum` 不出现；`sum_3` / `sum_5` 正常进入 `import_lib!`。

**用户操作**：若现有 wrapper 数量不足（例如需要 `sum_4`），在头文件和实现文件中手动添加，重新运行 `cpp2rust-demo init`。

---

#### `[FP]` — 函数指针参数

**C 函数指针**（`int (*op)(int, int)` 形式）自动映射为 `Option<unsafe extern "C" fn(i32, i32) -> i32>`，函数标记 `is_unsafe = true`，并在 `#[cpp(func = "...")]` 前自动插入：

```rust
// cpp2rust-todo[FP]: 含函数指针参数，需确保回调符合 extern "C" 调用约定
#[cpp(func = "int apply_operation(int, int, int (*)(int, int))")]
unsafe fn apply_operation(a: i32, b: i32, op: Option<unsafe extern "C" fn(i32, i32) -> i32>) -> i32;
```

**C++ 成员函数指针**（`int (Cls::*)() const` 形式）仍无法映射为合法 Rust FFI 类型，含此类参数的函数整体跳过。

生成结果：`apply_operation`、`lambda_wrapper_new` 等含 C 函数指针参数的函数现在出现在 `import_lib!` 中；`add_impl`、`make_add_lambda` 等普通函数不受影响。

**用户操作**：确认传入的回调函数符合 `extern "C"` 调用约定（无 Rust 闭包捕获、无 panic）；若需 Rust 闭包 → C++ 回调，手动编写 trampoline。

---

#### `[VM]` — volatile 成员函数

**根本原因**：hicc 通过方法指针类型（`R (T::*)() volatile`）绑定成员方法，而 Rust 的 `fn` 签名中没有 `volatile this` 的概念，类型不匹配导致编译失败。工具因此将 volatile 方法从 `import_class!` 中整体移除。

```cpp
class HardwareDevice {
public:
    void init();                             // 普通方法 —— 进入 import_class!
    uint32_t readStatus() volatile; // volatile 方法 —— 从 import_class! 移除 [VM]
};

// extern "C" shim（接收 volatile T* 作为第一参数）—— 仍进入 import_lib!
uint32_t hardware_device_read_status(volatile HardwareDevice* self);
```

生成结果：`HardwareDevice` 的 `import_class!` 中只有 `init`；`readStatus` 被跳过；但 `hardware_device_read_status(volatile HardwareDevice*)` 作为自由函数进入 `import_lib!`（标注 `unsafe`）。

**用户操作**：优先在 `extern "C"` 头文件中提供 `volatile T*` 参数的 C shim，工具即可自动生成 `import_lib!` 绑定。若头文件中无对应 shim，手动在 `hicc::cpp!` 中添加并声明到 `import_lib!`。

---

## 测试体系

测试分五层，位于 `tests/` 目录：

| 层 | 文件 | 验证内容 | 当前状态 |
|----|------|---------|---------|
| **L1** 黄金文件测试 | `l1_golden_tests.rs` | 工具生成的 hicc 脚手架与 `rust_hicc/src/main.rs` 中对应块一致 | ✅ **49/49 通过** |
| **L2** 编译测试 | `l2_compile_tests.rs` | 仓库中现有的 `rust_hicc/` 能通过 `cargo build` | ✅ **48/48 通过** |
| **L3** 运行测试 | `l3_run_tests.rs` | `cargo run` 输出与各示例 README 中"运行结果"一致 | ✅ **48/48 通过** |
| **L4** E2E 测试 | `rapidjson_e2e_test.rs` | 对 rapidjson shim 文件（`references/rapidjson-refactoring/rapidjson_sys/shim/`）执行完整 init + merge 转换，验证生成真实 `import_lib!` FFI 绑定（而非仅 `cpp!` 块） | ✅ 通过 |
| **L5** 符号验证测试 | `l5_nm_symbol_tests.rs` | 用 `nm` 双向验证 C++ 导出符号均已链接进 Rust FFI 二进制 | ✅ 通过 |

### 测试命令

```bash
# 运行 L1 黄金文件测试（须单线程：clang 全局状态竞争）
cargo test --test l1_golden_tests -- --include-ignored --test-threads=1

# 运行 L2 编译测试
cargo test --test l2_compile_tests

# 运行 L3 运行测试
cargo test --test l3_run_tests -- --include-ignored --test-threads=1

# 运行 L4 rapidjson E2E 测试（须单线程：避免并行磁盘操作冲突）
cargo test --test rapidjson_e2e_test -- --test-threads=1

# 显式运行需要 libgtest-dev 的 unittest 测试（非 CI 环境）
# cargo test --test rapidjson_e2e_test -- --ignored --test-threads=1

# 运行 L5 nm 符号验证测试
cargo test --test l5_nm_symbol_tests -- --include-ignored

# 运行某个示例的全部测试
cargo test 006_class_basic

# 更新黄金文件（工具输出有意变更时使用）
cargo test --test l1_golden_tests update_all_goldens -- --include-ignored
```

---

## 局限性

| 场景 | 说明 |
|------|------|
| **捕获阶段 Windows** | Linux/macOS 使用 `LD_PRELOAD` / `DYLD_INSERT_LIBRARIES`；Windows 通过 PATH 注入 `hook_shim.exe` 实现等价拦截（支持 GNU/MinGW + MSVC），L2/L3/L5 已在 Windows MSVC & MinGW 通过 CI |
| **命名空间类** | extern "C" 函数签名中含 `::` 类型或 `void*` opaque 指针时，会压制 `import_class!`/`import_lib!` 块（仅生成空 `cpp!`），需手动绑定 |
| **运算符重载** | 生成命名 shim + `[OP]` TODO，Rust 运算符 trait 需手动实现 |
| **有状态 Lambda / std::function** | 生成 class wrapper，若需 Rust 闭包回调需手动编写 trampoline |
| **可变参数模板** | 按调用点展开有限版本，超出范围的参数组合需手动添加 |
| **业务逻辑** | 工具只生成 FFI 绑定层（`lib.rs`），`fn main()` 和业务代码需手动编写 |
| **跨翻译单元模板** | 每个 `.cpp2rust` 独立解析，跨文件模板实例化可能遗漏（`merge` 阶段部分缓解） |

---

## 未来开发计划

以下按优先级从高到低排列（P1 最高，P3 最低）：

| 优先级 | 方向 | 现状 | 目标 |
|--------|------|------|------|
| ~~**P1**~~ | ~~**Windows 编译拦截**~~ ✅ | 已通过 `hook/hook_shim.rs` 实现：PATH 注入 `hook_shim.exe` 替代 `LD_PRELOAD`，同时支持 GNU/MinGW 和 MSVC 两种编译器 | — |
| ~~**P1**~~ | ~~**函数指针参数自动绑定 `[FP]`**~~ ✅ | 已实现：C 函数指针自动映射为 `Option<unsafe extern "C" fn(...)>`，标记 `is_unsafe = true` 并加 `cpp2rust-todo[FP]` 注释；C++ 成员函数指针仍跳过 | — |
| **P2** | **跨翻译单元模板合并** | 每个 `.cpp2rust` 独立解析，跨文件模板实例化可能遗漏；`merge` 阶段已部分缓解 | 在 `merge` 阶段实现跨文件模板实例化聚合，消除遗漏 |
| **P2** | **更多真实项目 E2E 验证** | 已有 rapidjson 完整参考（10 个子系统）+ L4 E2E 测试 | 扩展至更多主流 C++ 开源库（如 Eigen、Abseil、{fmt}），验证工具在复杂项目上的覆盖率和鲁棒性 |
| **P3** | **L3 运行测试本地化** | L3 运行测试主要在 CI 环境验证，本地运行步骤较繁琐 | 补充本地快速运行脚本，降低开发者验证门槛 |

---

## 学习路径（示例索引）

每个示例目录下包含：
- `cpp/`：C++ 源码（工具的输入）
- `rust_hicc/src/main.rs`：包含 hicc FFI 脚手架 + 手写 `fn main()` 的完整可运行参考文件
- `README.md`：特性说明 + 运行结果

```
入门阶段：001_hello_world → 002_function_overload → 003_default_args
         → 004_inline_functions → 005_variadic_functions → 006_class_basic

类与对象：007_class_constructor → 008_class_copy → 009_class_move
         → 010_class_static → 011_class_const → 012_class_volatile

面向对象：013_inheritance_single → 014_inheritance_multiple
         → 015_virtual_basic → 016_virtual_pure → 017_virtual_override → 018_virtual_diamond

运算符/类型：019_operator_overload → 020_friend_function → 021_explicit_ctor
            → 022_mutable_member → 023_typeid_rtti

模板：024_template_function → 025_template_class → 026_template_specialization
     → 027_template_instantiation → 028_variadic_template

内存管理：029_unique_ptr → 030_shared_ptr → 031_custom_deleter
         → 032_placement_new → 033_raii_pattern

STL：034_vector_basic → 035_map_basic → 036_string_basic
    → 037_array_basic → 038_tuple_basic

函数对象：039_lambda_basic → 040_std_function → 041_functional_bind → 042_exception_basic

高级特性：043_namespace_nested → 044_enum_class → 045_union_basic
         → 046_constexpr_basic → 047_noexcept_basic → 048_summary
```

### 运行单个示例

```bash
cd examples/001_hello_world

# 编译 C++ 共享库
cd cpp && g++ -shared -fPIC hello_world.cpp -o libhello_world.so && cd ..

# 编译并运行 Rust FFI
cd rust_hicc && cargo run
```

---

## 依赖

- Linux（LD_PRELOAD 必需）
- C++ 编译器：g++ 或 clang++（C++11 或更高）
- Rust 工具链：rustc / cargo（1.82+）
- libclang（用于 AST 解析）：`libclang-dev`
- [`hicc`](https://crates.io/crates/hicc) `0.2` 和 [`hicc-build`](https://crates.io/crates/hicc-build) `0.2`

---

## 仓库结构

```
cpp2rust-demo/
├── hook/              # LD_PRELOAD 拦截器（hook.cpp + Makefile）
├── src/               # 工具源码（Rust）
├── examples/          # 48 个示例，每个含 cpp/ 和 rust_hicc/ 子目录
├── tests/             # 三层测试体系（L1/L2/L3）
├── docs/
│   ├── plans/v5/      # 完整方案文档（automated-cpp2rust-ffi-v5.md）
│   └── references/    # hicc、c2rust-demo 等参考文档
└── references/
    └── c2rust-demo/   # C 语言版参考实现（同架构）
```

---

## 许可

MIT

