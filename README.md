# cpp2rust-demo

[![CI](https://github.com/LuuuXXX/cpp2rust-demo/actions/workflows/ci.yml/badge.svg)](https://github.com/LuuuXXX/cpp2rust-demo/actions/workflows/ci.yml)

**C++ → Rust Safe FFI 自动化脚手架生成工具**。给定任意 C++ 项目，执行两条命令即可生成基于 [hicc](https://crates.io/crates/hicc) 的 Rust FFI 绑定层（`hicc::cpp!` / `hicc::import_class!` / `hicc::import_lib!` 三段式），并整理为可直接使用的 Rust 项目结构。

```bash
cpp2rust-demo init -- make -j4   # 捕获构建 + 生成 FFI 脚手架
cpp2rust-demo merge              # 备份并整理编译单元输出（可选）
```

> **工具定位**：cpp2rust-demo 负责生成 FFI **脚手架**（绑定声明 + 必要 C 桥接 shim），不处理业务逻辑、不重写 C++ 代码。生成产物开箱即可 `cargo check`，部分降级特性需人工补全后才能完整编译运行。

**主要特性**：

- 🔗 **跨平台编译拦截**：Linux/macOS 使用 `LD_PRELOAD` / `DYLD_INSERT_LIBRARIES`；Windows 通过 PATH 注入 `hook_shim.exe`，同时支持 GNU/MinGW 和 MSVC
- 🔍 **libclang AST 解析**：精确提取类、函数、枚举、模板实例化；行标记扫描自动区分项目源文件与 `#include` 引入的头文件
- 📦 **hicc 三段式代码生成**：`hicc::cpp!`（C++ shim 内联）/ `hicc::import_class!`（类方法绑定）/ `hicc::import_lib!`（全局函数绑定）
- 🏷️ **多 feature 支持**：`--feature <name>` 将不同平台或构建配置的产物隔离到各自目录，`merge` 命令可将多个 feature 合并为带 `[features]` 段的统一 Rust 项目
- 🤖 **CI / 非交互环境自动全选**：stdin 非 TTY 时自动全选所有捕获到的 `.cpp2rust` 文件，无需人工干预
- 🧪 **五层测试体系**：L1 黄金文件比对 / L2 编译测试 / L3 运行输出验证 / L4 真实项目 E2E 转换（rapidjson + tinyxml2 / pugixml / sqlite3 / nlohmann-json / fmtlib） / L5 `nm` 符号双向验证
- ⚠️ **降级特性内联提示**：无法完全自动化的 C++ 特性（运算符重载、可变参数模板、有状态 Lambda 等）自动降级并在生成代码中插入 `// cpp2rust-todo[TAG]` 注释，精确定位待手动完善的位置

仓库同时包含 **48 个循序渐进的 C++ 特性示例**，每个示例都有对应的 C++ 源码和可运行的 Rust FFI 参考实现，覆盖从基础函数到模板、STL、虚继承等复杂场景。

**导航**：[工作原理](#工作原理) · [命令参考](#命令参考) · [快速开始](#快速开始) · [生成代码格式](#生成代码格式三段式) · [特性矩阵](#c-特性支持矩阵) · [降级特性](#降级特性详解7-项) · [测试体系](#测试体系) · [局限性](#局限性) · [学习路径](#学习路径示例索引)

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

各阶段技术细节（LD_PRELOAD 原理、libclang AST 解析策略、FfiSpec IR 结构、类型映射规则等）见 [docs/INTRODUCTION.md](docs/INTRODUCTION.md)。

---

## 命令参考

工具提供两个子命令，覆盖"捕获 → 生成 → 整理"的完整工作流：

| 子命令 | 作用 | 典型用法 |
|--------|------|---------|
| `init` | 通过 LD_PRELOAD 拦截构建命令，捕获 C++ 预处理文件，解析 AST，生成 hicc 三段式 FFI 脚手架 | `cpp2rust-demo init -- make -j4` |
| `merge` | 将 `init` 生成的编译单元文件整理为按 C++ 目录结构组织的 Rust 项目，备份原始输出；生成 `api-manifest.md` API 对账清单；支持多 feature 合并为带 `[features]` 的统一项目；可配合 `--output-dir` 在 merge 完成后同时导出到任意目录 | `cpp2rust-demo merge --feature default` |

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

1. 首次运行时将内嵌的 `hook.cpp` 解压到用户数据目录并编译为 `libhook.so`（Linux）/ `libhook.dylib`（macOS）（后续调用自动跳过）
2. 通过 `LD_PRELOAD`（Linux）/ `DYLD_INSERT_LIBRARIES`（macOS）注入构建过程，捕获 `.cpp2rust` 预处理文件
3. 交互式选择参与转换的文件（非交互/CI 环境自动全选）
4. libclang 解析 AST，提取类 / 函数 / 枚举 / 模板实例化
5. 生成 `.cpp2rust/<feature>/rust/` 下的 hicc Rust 脚手架
6. 生成冒烟测试 `.cpp2rust/<feature>/rust/tests/smoke.rs`（验证生成的 FFI 类型可被编译链接；可用 `CPP2RUST_GEN_SMOKE=0` 关闭）

> **验证闭环**：进入 `.cpp2rust/<feature>/rust/` 执行 `cargo test`，即可对生成的 Rust FFI 做"编译 + 冒烟"验证。`smoke.rs` 已存在时不会被覆盖，可在其上补充"构造→调用→断言"逻辑。

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

# merge 完成后同时导出到任意目录（单/多 feature 均支持）
cpp2rust-demo merge --output-dir /tmp/mylib-out
cpp2rust-demo merge --feature linux_x86 --output-dir /tmp/linux-out
cpp2rust-demo merge --feature linux_x86 --feature arm_embedded --output-dir /tmp/multi-out
```

`merge` 将 `init` 的扁平输出整理为按 C++ 目录结构组织的 Rust 项目，并提供备份机制：

```
.cpp2rust/<feature>/rust/
    ├── src.1/   ← init 输出原始备份（首次运行时 rename from src）
    └── src/     ← merge 输出，真实目录（维持 C++ 目录结构）
```

同时在 `.cpp2rust/<feature>/meta/` 下生成 `api-manifest.md`（C++ → Rust API 对账清单）。

多 feature 合并时，输出到 `.cpp2rust/<feat1>_<feat2>/rust/`，生成含 `[features]` 段的 `Cargo.toml` 和按 feature 条件编译的 `src/lib.rs`、`build.rs`：

```bash
cd .cpp2rust/linux_x86_arm_embedded/rust
cargo build --features linux_x86
cargo build --features arm_embedded
```

指定 `--output-dir` 时，merge 完成后自动将产物导出到该目录：

```
/tmp/mylib-out/
    ├── meta/        （.cpp2rust/ 的完整副本，包含 api-manifest.md 等）
    ├── src/         （合并后的 Rust 源码）
    ├── build.rs
    └── Cargo.toml
```

多 feature 配合 `--output-dir` 时，导出目录结构：

```
/tmp/multi-out/
    ├── meta/            （包含各 feature 的 api-manifest.md 等）
    ├── src/
    │   ├── lib.rs       （顶层 #[cfg(feature = "linux_x86")] / #[cfg(feature = "arm_embedded")] 路由）
    │   ├── linux_x86/   （linux_x86 feature 的绑定文件）
    │   └── arm_embedded/（arm_embedded feature 的绑定文件）
    ├── build.rs         （含 #[cfg(feature = ...)] 条件编译段）
    └── Cargo.toml       （含 [features] 段）
```

**典型使用场景：**

```bash
# 场景一：CI/CD 流水线中将 FFI 脚手架发布为独立 crate
#   在 CMake/Make 构建完成后自动生成并导出到 Rust workspace
cpp2rust-demo init -- cmake --build build -j8
cpp2rust-demo merge --output-dir ../rust-ffi/mylib

# 场景二：交叉编译多平台同时导出
cpp2rust-demo init -- make ARCH=x86_64 # 生成 linux_x86 feature
cpp2rust-demo init -- make ARCH=aarch64 --feature arm_embedded
cpp2rust-demo merge --feature linux_x86 --feature arm_embedded --output-dir dist/

# 场景三：GitHub Actions 工件上传
- run: cpp2rust-demo merge --output-dir ${{ runner.temp }}/ffi-out
- uses: actions/upload-artifact@v4
  with:
    name: rust-ffi-scaffold
    path: ${{ runner.temp }}/ffi-out
```

**参数说明：**

| 参数 | 必填 | 说明 |
|------|------|------|
| `--feature <name>` | ❌ | 要操作的构建目标（默认 `default`）；**可重复指定**，≥2 个时进入多 feature 合并模式 |
| `--output-dir <DIR>` | ❌ | 导出目标目录（不存在时自动创建）；指定时在 merge 完成后追加导出步骤，不指定则走普通 merge 流程 |

### 环境变量

| 变量 | 说明 |
|------|------|
| `CPP2RUST_CXX` | 覆盖默认 C++ 编译器（默认自动检测 g++/clang++/c++，支持带版本后缀如 g++-13） |
| `CPP2RUST_DEBUG` | 非空时输出 hook 调试日志到 stderr |
| `CPP2RUST_GEN_SMOKE` | 控制是否生成冒烟测试 `tests/smoke.rs`（默认开启；设为 `0`/`false`/`no`/`off` 关闭） |
| `CPP2RUST_GEN_TEMPLATES` | 控制是否生成模板类 / 模板函数的泛型 hicc 骨架、**模板实例化别名及构造工厂骨架**（默认**关闭**；设为 `1`/`true`/`yes`/`on` 开启）。关闭时默认产物逐字节不变 |
| `CPP2RUST_GEN_PROXY` | 控制是否生成 `@make_proxy` 代理工厂骨架（让 Rust 侧实现 C++ 抽象接口；默认**关闭**；设为 `1`/`true`/`yes`/`on` 开启）。关闭时默认产物逐字节不变 |
| `CPP2RUST_GEN_DYNAMIC_CAST` | 控制是否生成 `@dynamic_cast` 下行转换骨架（RTTI 场景把多态基类指针向下转换为派生类指针；默认**关闭**；设为 `1`/`true`/`yes`/`on` 开启）。关闭时默认产物逐字节不变 |

---

## 快速开始

### 安装依赖

#### Linux（Ubuntu / Debian）

```bash
# 系统依赖
sudo apt-get install clang libclang-dev g++ libstdc++-14-dev

# 从 GitHub 安装（无需克隆仓库）
cargo install --git https://github.com/LuuuXXX/cpp2rust-demo

# 或从本地源码安装（开发者）
cargo install --path .
```

#### macOS

```bash
# 安装 Homebrew LLVM（提供 libclang 和 clang++）
brew install llvm

# 设置 LIBCLANG_PATH（使工具能找到 libclang.dylib）
# 建议写入 ~/.zprofile 或 ~/.bash_profile 永久生效
export LIBCLANG_PATH=$(brew --prefix llvm)/lib

# 确认 Xcode Command Line Tools 已安装（提供 make、ar 等基础工具）
xcode-select --install  # 若已安装会提示跳过

# 从 GitHub 安装（无需克隆仓库）
cargo install --git https://github.com/LuuuXXX/cpp2rust-demo

# 或从本地源码安装（开发者）
cargo install --path .
```

> **macOS SIP（系统完整性保护）注意事项**：`DYLD_INSERT_LIBRARIES`（macOS 上的编译拦截机制）
> 对受系统保护的二进制（如 `/usr/bin/g++`、`/usr/bin/clang++`）无效，SIP 会静默忽略注入。
>
> **解决方案**：使用 Homebrew 安装的编译器，它们位于 `/opt/homebrew/bin/` 或 `/usr/local/bin/`，
> 不受 SIP 保护：
>
> ```bash
> brew install gcc    # 提供 g++-14 等版本化编译器
> # 或
> brew install llvm   # 提供 $(brew --prefix llvm)/bin/clang++
> ```
>
> 调用 `cpp2rust-demo init` 时，将 Homebrew 编译器路径置于 PATH 最前面，或通过 `CPP2RUST_CXX`
> 环境变量显式指定：
>
> ```bash
> # 方式一：调整 PATH（推荐）
> export PATH="$(brew --prefix llvm)/bin:$PATH"
> cpp2rust-demo init -- make -j4
>
> # 方式二：通过环境变量显式指定
> CPP2RUST_CXX=$(brew --prefix llvm)/bin/clang++ cpp2rust-demo init -- make -j4
> ```

> **注意**：`hook/hook.cpp` 已内嵌进 binary，无需额外文件。首次执行 `init` 时工具
> 自动将 hook 源码解压到 `~/.local/share/cpp2rust-demo/hook/`（Linux）或
> `~/Library/Application Support/cpp2rust-demo/hook/`（macOS）并编译；后续调用在 hook 库为最新版时自动跳过重编译。

#### Windows

Windows 平台通过将 `hook_shim.exe` 注入到 PATH 最前面来拦截 `g++`/`clang++`/`cl` 调用，支持 MinGW-w64 和 MSVC 两种工具链。

##### 前提：Rust 工具链

```powershell
# 从 https://rustup.rs 下载安装 rustup
# 安装时选择对应工具链目标：
#   MinGW-w64 路径：stable-x86_64-pc-windows-gnu
#   MSVC 路径：    stable-x86_64-pc-windows-msvc（推荐）
rustup toolchain install stable-x86_64-pc-windows-msvc
```

##### 方式 A：MinGW-w64（`g++`）

1. 安装 MSYS2（https://www.msys2.org）并打开 MSYS2 MinGW 64-bit 终端：
   ```bash
   pacman -S mingw-w64-x86_64-toolchain mingw-w64-x86_64-clang
   ```
2. 确认 `g++` 和 `clang` 在 PATH 中：
   ```bash
   g++ --version    # 期望输出 MinGW-w64 g++ ...
   clang++ --version
   ```
3. 安装工具（在 MSYS2 MinGW 终端或 PowerShell 中）：
   ```powershell
   cargo install --git https://github.com/LuuuXXX/cpp2rust-demo
   ```
4. 在目标 C++ 项目目录运行 `init`（PowerShell 或 MSYS2 终端）：
   ```powershell
   cpp2rust-demo init -- make -j4
   # 或使用 CMake：
   cpp2rust-demo init -- cmake --build build
   ```

##### 方式 B：MSVC（`cl.exe`）

1. 安装 Visual Studio 2019/2022（包含 **"使用 C++ 的桌面开发"** 工作负载），或安装 "Build Tools for Visual Studio"
2. 打开 **"x64 Native Tools Command Prompt for VS"**（确保 `cl.exe` 在 PATH 中）：
   ```cmd
   cl.exe /?    :: 期望输出 Microsoft (R) C/C++ 编译器版本 ...
   ```
3. 安装 LLVM for Windows（提供 `libclang.dll`，构建时必须）：
   - 从 https://github.com/llvm/llvm-project/releases 下载 LLVM-x.y.z-win64.exe
   - 安装后设置环境变量（在 x64 命令提示符中）：
     ```cmd
     set LIBCLANG_PATH=C:\Program Files\LLVM\bin
     ```
     建议将其写入系统环境变量以永久生效。
4. 安装工具：
   ```cmd
   cargo install --git https://github.com/LuuuXXX/cpp2rust-demo
   ```
5. 运行 `init`：
   ```cmd
   cpp2rust-demo init -- msbuild MyProject.sln /p:Configuration=Release
   :: 或 CMake：
   cpp2rust-demo init -- cmake --build build --config Release
   ```

> **Windows PATH 注入机制**：`hook_shim.exe` 会在运行时自动注入到 PATH 最前面，拦截编译器调用后记录预处理结果至 `.cpp2rust` 文件。首次执行 `init` 时工具将 `hook_shim.exe` 解压至 `%APPDATA%\cpp2rust-demo\hook\` 并通过 PATH 前置使其生效；后续调用若二进制未变则自动跳过重部署。

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
1. 首次运行时将内嵌的 `hook.cpp` 解压到用户数据目录并编译为 `libhook.so`（Linux）/ `libhook.dylib`（macOS）（后续调用自动跳过）
2. 通过 `LD_PRELOAD`（Linux）/ `DYLD_INSERT_LIBRARIES`（macOS）注入构建过程，捕获 `.cpp2rust` 预处理文件
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
    ├── src.1/   ← init 输出的原始备份（首次运行时 rename from src）
    └── src/     ← merge 输出，真实目录（维持 C++ 目录结构）
```

同时在 `.cpp2rust/<feature>/meta/` 下生成：
- `merge-report.md`：merge 阶段汇总报告（.rs 文件数、FFI 绑定统计、降级标记）
- `api-manifest.md`：C++ → Rust API 对账清单（Markdown 格式，记录每个类方法和独立函数的 C++ 签名与 Rust 签名，含降级标记）

- 首次运行：`src/` 重命名为 `src.1/`，merge 输出写入新的 `src/`
- 重复运行：`src.1/` 保持不变（原始 init 输出），`src/` 重新写入最新 merge 输出

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
| `[FP]` | 函数指针参数（`void (*)(...)` 类型，自动映射为 `unsafe extern "C" fn(...)`，生成 `cpp2rust-todo[FP]` 注释提示安全性要求） | 确认回调符合 `extern "C"` 调用约定；若需 Rust 闭包，手动编写 trampoline |
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

> **生成 Cargo.toml 条件引入 `hicc-std` 依赖**：工具生成的 Rust FFI 代码通过 C++ 侧自定义包装类
> 将 STL 类型暴露为普通 `extern "C"` 接口，所有平台均可编译运行，无需直接使用 `hicc_std::` 类型。
> 为方便在 Linux / Windows 上直接使用 `hicc_std::` 类型别名（如 `hicc_std::string`、`hicc_std::vector`
> 等），工具自动在生成的 `Cargo.toml` 中通过
> `[target.'cfg(not(target_os = "macos"))'.dependencies]` 引入 `hicc-std`，macOS 不引入。
>
> **macOS 不引入的原因**：`hicc-std 0.2` 在 macOS Apple Clang 下存在编译问题——其 `build.rs`
> 在非 MSVC 平台统一链接 `stdc++`（GNU libstdc++），而 Apple Clang 默认使用 `libc++`
> （需链接 `-lc++`），导致 `cargo build` 在 macOS 上失败。macOS 用户若需直接使用
> `hicc_std::` 类型，可在项目 `build.rs` 中手动处理标准库链接后再添加 `hicc-std` 依赖；
> 通常使用工具生成的 wrapper 类方式即可满足需求，无需额外步骤。

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
> 平台列：Linux = Linux（GCC/Clang）；macOS = macOS（Apple Clang）；Win = Windows（MinGW/MSVC）
> `¹` 标注的特性：生成项目在 Linux/Win 上自动引入 `hicc-std` 依赖，可直接使用 `hicc_std::` 类型别名；macOS 不引入，仅支持 wrapper 类方式（功能等价，但不支持 `hicc_std::` 直接类型）

| 示例 | 类别 | C++ 特性 | 状态 | 平台 | FFI 策略 |
|------|------|---------|------|------|---------|
| [001_hello_world](examples/001_hello_world) | 基础函数 | extern "C" 函数 | ✅ | 全平台 | AST 直接提取 → `import_lib!` |
| [002_function_overload](examples/002_function_overload) | 基础函数 | 函数重载 | ✅ | 全平台 | 各重载名称加类型后缀区分（`_i32`/`_f64`）→ `import_lib!` |
| [003_default_args](examples/003_default_args) | 基础函数 | 默认参数 | ✅ | 全平台 | C++ 侧展开为多个固定参数重载，写入 `hicc::cpp!` |
| [004_inline_functions](examples/004_inline_functions) | 基础函数 | inline 函数 | ✅ | 全平台 | 函数体从 `.cpp2rust` 提取，内联写入 `hicc::cpp!` |
| [005_variadic_functions](examples/005_variadic_functions) | 基础函数 | C 可变参数（`...`） | ⚠️ `[CV]` | 全平台 | `...` 参数函数整体跳过；头文件中的固定参数 wrapper 直接 `extern "C"` 绑定 |
| [006_class_basic](examples/006_class_basic) | 类与对象 | 基础类 | ✅ | 全平台 | opaque pointer + `import_class!` + ctor/dtor shim |
| [007_class_constructor](examples/007_class_constructor) | 类与对象 | 构造/析构 | ✅ | 全平台 | `*_new()` / `*_delete()` 必要 shim |
| [008_class_copy](examples/008_class_copy) | 类与对象 | 拷贝构造 | ✅ | 全平台 | `*_copy(const Foo*)` 必要 shim |
| [009_class_move](examples/009_class_move) | 类与对象 | 移动构造 | ✅ | 全平台 | `*_move(Foo*)` shim（内部 `std::move()`） |
| [010_class_static](examples/010_class_static) | 类与对象 | 静态成员 | ✅ | 全平台 | `{class}_get_{field}` / `{class}_set_{field}` 必要 shim |
| [011_class_const](examples/011_class_const) | 类与对象 | const 成员函数 | ✅ | 全平台 | 直接 `import_class!`，映射为 `fn method(&self)` |
| [012_class_volatile](examples/012_class_volatile) | 类与对象 | volatile 成员函数 | ⚠️ `[VM]` | 全平台 | `volatile this` 方法从 `import_class!` 中整体移除；`extern "C"` shim（接收 `volatile T*`）仍进入 `import_lib!` |
| [013_inheritance_single](examples/013_inheritance_single) | 面向对象 | 单继承 | ✅ | 全平台 | 基类方法在子类 `import_class!` 中一并提升，无 shim |
| [014_inheritance_multiple](examples/014_inheritance_multiple) | 面向对象 | 多继承 | ✅ | 全平台 | 多条继承链展开，所有方法通过 `import_class!` 直接绑定 |
| [015_virtual_basic](examples/015_virtual_basic) | 面向对象 | 虚函数 | ✅ | 全平台 | opaque pointer 调用，hicc 宏负责虚表 dispatch |
| [016_virtual_pure](examples/016_virtual_pure) | 面向对象 | 纯虚/抽象类 | ✅ | 全平台 | 抽象类只生成前向声明；子类通过 `import_class!` 绑定 |
| [017_virtual_override](examples/017_virtual_override) | 面向对象 | override | ✅ | 全平台 | override 语义透传，与普通虚函数相同 |
| [018_virtual_diamond](examples/018_virtual_diamond) | 面向对象 | 菱形继承（virtual 继承） | ✅ | 全平台 | 为每条继承方法生成命名 shim（`d_getAValue(D*)`），避免指针调整 |
| [019_operator_overload](examples/019_operator_overload) | 运算符/类型 | 运算符重载 | ⚠️ `[OP]` | 全平台 | 自动生成命名 shim（`{class}_add` 等）写入 `hicc::cpp!` + `import_lib!`；Rust `ops::*` trait 需手动实现 |
| [020_friend_function](examples/020_friend_function) | 运算符/类型 | 友元函数 | ✅ | 全平台 | 友元函数提取为普通函数写入 `import_lib!` |
| [021_explicit_ctor](examples/021_explicit_ctor) | 运算符/类型 | explicit 构造函数 | ✅ | 全平台 | `explicit` 对 FFI 透明，与普通构造相同 |
| [022_mutable_member](examples/022_mutable_member) | 运算符/类型 | mutable 成员 | ✅ | 全平台 | `mutable` 对 FFI 透明，直接 `import_class!` |
| [023_typeid_rtti](examples/023_typeid_rtti) | 运算符/类型 | typeid/RTTI/dynamic_cast | ✅ | 全平台 | 注入整数枚举 + 虚函数 `getType()`，完全绕过 `typeid` |
| [024_template_function](examples/024_template_function) | 模板实例化 | 函数模板 | ✅ | 全平台 | 忽略模板声明，为每个实例化版本生成命名 C 包装函数 |
| [025_template_class](examples/025_template_class) | 模板实例化 | 类模板 | ✅ | 全平台 | 只处理实际实例化的具体类型，按普通类生成 |
| [026_template_specialization](examples/026_template_specialization) | 模板实例化 | 模板偏特化 | ✅ | 全平台 | 偏特化视为实例化路径之一，收集通过该路径实例化的类型 |
| [027_template_instantiation](examples/027_template_instantiation) | 模板实例化 | 显式模板实例化 | ✅ | 全平台 | 显式实例化在 AST 中直接可见，按普通类处理 |
| [028_variadic_template](examples/028_variadic_template) | 模板实例化 | 可变参数模板 | ⚠️ `[VA]` | 全平台 | 生成 wrapper 类 + 按参数数量展开的静态方法；超出范围的组合需手动添加 |
| [029_unique_ptr](examples/029_unique_ptr) | 智能指针/内存 | std::unique_ptr | ✅ | 全平台 | opaque pointer；`*_new()` 返回裸指针，调用方 `*_delete()` 释放 |
| [030_shared_ptr](examples/030_shared_ptr) | 智能指针/内存 | std::shared_ptr | ✅ | 全平台 | `*_clone()` shim 增加引用计数，`*_delete()` 减少；其余方法直接绑定 |
| [031_custom_deleter](examples/031_custom_deleter) | 智能指针/内存 | 自定义删除器 | ✅ | 全平台 | 删除器函数注入 `hicc::cpp!`，`*_delete()` shim 内部调用自定义删除器 |
| [032_placement_new](examples/032_placement_new) | 智能指针/内存 | placement new | ✅ | 全平台 | 生成 `*_placement_new(ptr, ...)` 必要 shim |
| [033_raii_pattern](examples/033_raii_pattern) | 智能指针/内存 | RAII 模式 | ✅ | 全平台 | 析构函数生成 `*_delete()` shim，Rust 侧可实现 `Drop` trait |
| [034_vector_basic](examples/034_vector_basic) | STL 容器 | std::vector\<T\> | ✅ | 全平台¹ | 薄 wrapper 类 `IntVector`（`VectorImpl<int>` 封装）→ `import_class!` |
| [035_map_basic](examples/035_map_basic) | STL 容器 | std::map\<K,V\> | ✅ | 全平台¹ | 薄 wrapper 类 `StringIntMap`（`MapImpl<string,int>` 封装）→ `import_class!` |
| [036_string_basic](examples/036_string_basic) | STL 容器 | std::string | ✅ | 全平台¹ | string wrapper，`c_str()`/`length()` 等通过 `import_class!` 绑定 |
| [037_array_basic](examples/037_array_basic) | STL 容器 | std::array\<T,N\> | ✅ | 全平台¹ | 数组 wrapper（N 在实例化时已知）→ `import_class!` |
| [038_tuple_basic](examples/038_tuple_basic) | STL 容器 | std::tuple\<T...\> | ✅ | 全平台¹ | tuple wrapper，按位置 `get<N>()` 通过 `import_class!` 绑定 |
| [039_lambda_basic](examples/039_lambda_basic) | 函数对象 | Lambda 表达式 | ⚠️ `[LM]` | 全平台 | 无状态 lambda → 函数指针；有状态 lambda → class wrapper + `call()` 方法；C 函数指针参数自动映射为 `unsafe extern "C" fn(...)`（加 `[FP]` 注释） |
| [040_std_function](examples/040_std_function) | 函数对象 | std::function\<Sig\> | ⚠️ `[LM]` | 全平台 | 类型擦除容器，统一用 class wrapper + opaque pointer；C 函数指针参数自动映射为 `unsafe extern "C" fn(...)` |
| [041_functional_bind](examples/041_functional_bind) | 函数对象 | std::bind | ✅ | 全平台 | 产物本质是函数对象，同有状态 lambda 策略，`import_class!` 完全覆盖 |
| [042_exception_basic](examples/042_exception_basic) | 函数对象 | C++ 异常处理 | ✅ | 全平台 | shim 层 `try/catch` 捕获异常，转为错误码 + 错误消息字符串返回 |
| [043_namespace_nested](examples/043_namespace_nested) | 高级特性 | 嵌套命名空间 | ✅ | 全平台 | `void*` opaque pointer + raw `extern "C"` 绑定，函数名前缀扁平化（`foo::bar::Baz` → `foo_bar_baz_*`） |
| [044_enum_class](examples/044_enum_class) | 高级特性 | 强类型枚举（enum class） | ✅ | 全平台 | 枚举值导出为 Rust `const`，建议手动实现 `enum` + `TryFrom<i32>` |
| [045_union_basic](examples/045_union_basic) | 高级特性 | union | ✅ | 全平台 | opaque pointer + 按字段名 getter/setter shim；Rust 侧用 `#[repr(C)] union` |
| [046_constexpr_basic](examples/046_constexpr_basic) | 高级特性 | constexpr 常量/函数 | ✅ | 全平台 | 编译期常量读取 AST `IntegerLiteral` 值，生成 Rust `const`；constexpr 函数按普通函数处理 |
| [047_noexcept_basic](examples/047_noexcept_basic) | 高级特性 | noexcept | ✅ | 全平台 | `noexcept` 语义对 FFI 透明，直接处理 |
| [048_summary](examples/048_summary) | 高级特性 | 综合 FFI 模式 | ✅ | 全平台 | 以上所有策略的组合应用 |

> **STL 容器核心策略**：先在 `hicc::cpp!` 中生成薄 wrapper 类（如 `IntVector` 封装 `std::vector<int>`），再对 wrapper 类做 `import_class!` 绑定，规避模板方法签名复杂度。所有平台均支持此方式；Linux / Windows 上生成项目还额外携带 `hicc-std` 依赖（见上方说明），可直接使用 `hicc_std::vector`、`hicc_std::map`、`hicc_std::string` 等类型别名（macOS 不支持）。

---

## 降级特性详解（7 项）

| TAG | 示例 | C++ 特性 | 无法完全自动的根本原因 | 自动降级策略 | 用户剩余工作 |
|-----|------|---------|---------------------|------------|------------|
| `[OP]` | 019 | 运算符重载 | C ABI 无运算符符号；FFI 边界只能传命名函数 | 为每个运算符生成命名 shim（`{class}_add/sub/mul/div/mod/shl/shr/bitand/bitor/bitxor`、比较：`{class}_eq/ne/lt/gt/le/ge`、一元：`{class}_negate/not/bitnot/pre_inc/pre_dec`），写入 `hicc::cpp!` + `import_lib!` | 可选：手动实现 `impl std::ops::Add<T> for T` 等 |
| `[VA]` | 028 | 可变参数模板 | `...Args` 是编译期展开，FFI 无法表达"任意数量参数" | 生成 wrapper 类，按参数数量和类型组合分别封装为静态方法（`sum_1`/`sum_2` 等） | 若需要新的参数数量/类型组合，在 `hicc::cpp!` 中手动添加对应方法和包装函数 |
| `[LM]` | 039 | 有状态 Lambda | 匿名闭包类型，FFI 无法表达捕获列表 | 无状态 lambda → 函数指针；有状态 lambda → class wrapper + opaque pointer | 若需 Rust 闭包 → C++ 回调，手动编写 trampoline |
| `[LM]` | 040 | std::function | 类型擦除容器，捕获状态不透明 | 统一使用 class wrapper + opaque pointer | 可选：手动实现 Rust 闭包 → `std::function` 适配层 |
| `[CV]` | 005 | C 可变参数函数 | C 的 `...` 参数在运行时按 `va_list` 访问，Rust FFI 要求精确的静态类型列表，无法表达可变元 | 含 `...` 的函数（`is_variadic = true`）整体跳过，不生成任何绑定 | 在头文件中为每种实际调用的参数组合提供固定参数 wrapper，工具自动绑定这些 wrapper |
| `[FP]` | 039, 040 | 函数指针参数 | C++ 成员函数指针（`int (Cls::*)()` 等）无法映射为 Rust FFI 类型 | C 函数指针（`void (*)(int)` 等）自动映射为 `unsafe extern "C" fn(i32)`，标记 `is_unsafe = true` 并在绑定前加 `// cpp2rust-todo[FP]:` 注释；C++ 成员函数指针仍整体跳过 | 确认回调符合 `extern "C"` 调用约定；若需 Rust 闭包，手动编写 trampoline |
| `[VM]` | 012 | volatile 成员函数 | hicc 通过方法指针类型进行检查，`volatile this` 修饰的方法指针（`R (T::*)() volatile`）在 Rust 无对应语义，导致类型不匹配 | `is_volatile = true` 的方法从 `import_class!` 中整体移除；`extern "C"` shim 若以 `volatile T*` 为第一参数，则仍进入 `import_lib!` | 检查 `import_lib!` 中是否已有对应 `volatile T*` C shim；若无，在头文件中手动添加 `void foo_read(volatile Foo* self)` 并重新运行 `init` |
| `[LONG_DOUBLE]` | — | `long double` 类型 | x86-64 Linux 的 `long double` 是 80 位扩展浮点，Rust 无原生对应类型 | 自动降级映射为 `f64`（64 位双精度），有精度损失，并在对应绑定处加 `// cpp2rust-todo[LONG_DOUBLE]` 注释 | 若需精确 80 位精度，考虑引入第三方 `f128`/`rug` crate，或改用 C 桥接函数转换为 `double` 后再绑定 |

各降级特性的完整代码示例（C++ 源码 + 生成的 Rust FFI 代码）见 [docs/INTRODUCTION.md — 降级特性详解](docs/INTRODUCTION.md#part-3降级特性详解)。

### 类型映射注意事项

工具在将 C++ 类型映射为 Rust FFI 类型时，以下情况需要特别注意：

| C++ 类型 | Rust 映射 | 注意事项 |
|---------|----------|---------|
| `long double` | `f64` | **精度损失**：x86-64 Linux 上 `long double` 为 80 位扩展浮点，映射为 64 位 `f64` 会丢失精度。自动标注 `cpp2rust-todo[LONG_DOUBLE]`。 |
| `T&`（左值引用） | `&mut T` | **生命周期安全**：Rust 引用携带生命周期保证，而 FFI 中 C++ 引用的生命周期由调用方管理，Rust 编译器无法验证。使用时需确保被引用对象的生命周期长于 Rust 引用。 |
| `const T&` | `&T` | 同上，const 引用映射为不可变引用，生命周期同样由调用方负责。 |
| `void*` | `*mut u8` | opaque 指针，类型信息丢失，建议通过 hicc `import_class!` 的 opaque 类型封装。 |
| `T[N]` | `*mut T` | C 数组参数退化为指针，元素数量信息丢失。 |

---

## 测试体系

测试分五层，位于 `tests/` 目录：

| 层 | 文件 | 验证内容 | 当前状态 |
|----|------|---------|---------|
| **L1** 黄金文件测试 | `l1_golden_tests.rs` | 工具生成的 hicc 脚手架与 `rust_hicc/src/lib.rs`（或 `main.rs`）中对应块一致 | ✅ **49/49 通过** |
| **L2** 编译测试 | `l2_compile_tests.rs` | 仓库中现有的 `rust_hicc/` 能通过 `cargo build` | ✅ **48/48 通过** |
| **L3** 运行测试 | `l3_run_tests.rs` | `cargo run` 输出与各示例 README 中"运行结果"一致 | ✅ **48/48 通过** |
| **L_smoke** 冒烟测试 | 各示例 `tests/smoke.rs` | `cargo test` 验证 FFI 绑定行为（015–018、023–027 已覆盖） | ✅ 通过（已改造示例） |
| **L4** E2E 测试 | `rapidjson_e2e_test.rs` 等 | 对真实开源项目执行完整 init + merge 转换：①rapidjson（10 子系统 shim）验证 `import_lib!` FFI 绑定；②五大库（tinyxml2 / pugixml / sqlite3 / nlohmann-json / fmtlib）验证工具在不同类型项目上的覆盖率与鲁棒性 | ✅ 通过 |
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

# 运行 L4 五大库 E2E 测试（须先初始化对应子模块）
# git submodule update --init references/tinyxml2 references/pugixml references/nlohmann-json references/fmtlib
cargo test --test tinyxml2_e2e_test -- --test-threads=1
cargo test --test pugixml_e2e_test -- --test-threads=1
cargo test --test sqlite3_e2e_test -- --test-threads=1   # Linux 需安装 libsqlite3-dev
cargo test --test nlohmann_json_e2e_test -- --test-threads=1
cargo test --test fmtlib_e2e_test -- --test-threads=1

# 运行 L5 nm 符号验证测试
cargo test --test l5_nm_symbol_tests -- --include-ignored

# 运行某个示例的全部测试（含 smoke 冒烟）
cargo test 006_class_basic

# 运行改造示例的 L_smoke 冒烟测试（需在示例目录执行 cargo test）
# 例如：
cd examples/024_template_function/rust_hicc && cargo test
cd examples/025_template_class/rust_hicc && cargo test
cd examples/026_template_specialization/rust_hicc && cargo test
cd examples/027_template_instantiation/rust_hicc && cargo test
cd examples/015_virtual_basic/rust_hicc && cargo test
cd examples/016_virtual_pure/rust_hicc && cargo test
cd examples/017_virtual_override/rust_hicc && cargo test
cd examples/018_virtual_diamond/rust_hicc && cargo test
cd examples/023_typeid_rtti/rust_hicc && cargo test

# 更新黄金文件（工具输出有意变更时使用）
cargo test --test l1_golden_tests update_all_goldens -- --include-ignored
```

---

## 局限性

| 场景 | 说明 |
|------|------|
| **命名空间类** | extern "C" 函数签名中含 `::` 类型或 `void*` opaque 指针时，会压制 `import_class!`/`import_lib!` 块（仅生成空 `cpp!`），需手动绑定 |
| **运算符重载** | 生成命名 shim + `[OP]` TODO，Rust 运算符 trait 需手动实现 |
| **有状态 Lambda / std::function** | 生成 class wrapper，若需 Rust 闭包回调需手动编写 trampoline |
| **可变参数模板** | 按调用点展开有限版本，超出范围的参数组合需手动添加 |
| **业务逻辑** | 工具只生成 FFI 绑定层（`lib.rs`），`fn main()` 和业务代码需手动编写 |
| **跨翻译单元模板** | 每个 `.cpp2rust` 独立解析，跨文件模板实例化可能遗漏（`merge` 阶段部分缓解） |

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

# 编译 C++ 共享库（Linux）
cd cpp && g++ -shared -fPIC hello_world.cpp -o libhello_world.so && cd ..

# 编译 C++ 共享库（macOS）
cd cpp && clang++ -dynamiclib hello_world.cpp -o libhello_world.dylib && cd ..

# 编译并运行 Rust FFI
cd rust_hicc && cargo run
```

---

## 依赖

- **操作系统**：
  - Linux / macOS（`LD_PRELOAD` / `DYLD_INSERT_LIBRARIES` 编译拦截）
  - Windows（`hook_shim.exe` PATH 注入，同时支持 GNU/MinGW 和 MSVC 编译器）
- C++ 编译器：g++ 或 clang++（C++11 或更高）
- Rust 工具链：rustc / cargo（1.82+）
- libclang（用于 AST 解析）：`libclang-dev`
- [`hicc`](https://crates.io/crates/hicc) `0.2` 和 [`hicc-build`](https://crates.io/crates/hicc-build) `0.2`

---

## 故障排查

### `libclang not found` / `Unable to find libclang`

libclang 未找到时工具会在 init 阶段报错。解决方法：

```sh
# Ubuntu / Debian
sudo apt-get install libclang-dev
export LIBCLANG_PATH=/usr/lib/llvm-14/lib   # 替换为实际版本路径

# macOS（Homebrew）
brew install llvm
export LIBCLANG_PATH=$(brew --prefix llvm)/lib

# 永久生效：将 export 行写入 ~/.bashrc 或 ~/.zshrc
```

### `make failed in hook/`

表明 hook 构建失败，通常是缺少 C++ 编译器：

```sh
# Ubuntu / Debian
sudo apt-get install g++

# macOS
xcode-select --install    # 或安装 Xcode
```

### macOS SIP 导致捕获失败

`DYLD_INSERT_LIBRARIES` 对受 SIP（系统完整性保护）保护的系统二进制（如 `/usr/bin/g++`）无效，注入会被静默忽略。

解决方法：使用不受 SIP 保护的编译器（如 Homebrew 安装的 `g++` 或 `clang++`）执行构建命令。详见 [前置条件说明](#前置条件)。

### 生成的 Rust 项目 `cargo check` 失败

常见原因：

- **缺少 `hicc`/`hicc-build` 依赖**：确保 `Cargo.toml` 中包含 `hicc = "0.2"` 和 `[build-dependencies] hicc-build = "0.2"`
- **libclang 版本与 clang-sys 不兼容**：升级 libclang 或添加 `LIBCLANG_PATH` 指向正确路径
- **降级类型（`cpp2rust-todo[TAG]`）**：生成代码中带 `cpp2rust-todo` 注释的函数需要手动补全类型映射

---

## 仓库结构

```
cpp2rust-demo/
├── hook/              # LD_PRELOAD 拦截器（hook.cpp + Makefile）
├── src/               # 工具源码（Rust）
├── examples/          # 48 个示例，每个含 cpp/ 和 rust_hicc/ 子目录
├── tests/             # 五层测试体系（L1–L5）
├── docs/
│   ├── plans/v5/      # 完整方案文档（automated-cpp2rust-ffi-v5.md）
│   └── references/    # hicc、c2rust-demo 等参考文档
└── references/
    └── c2rust-demo/   # C 语言版参考实现（同架构）
```

---

## 许可

MIT

