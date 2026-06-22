# usage — 本地验证脚本与 SKILL 使用文档

本目录包含针对多个主流 C++ 库的 cpp2rust-demo FFI 转换验证脚本，覆盖 Linux（Bash `.sh`）和 Windows（PowerShell `.ps1`）两个平台。

---

## 一、验证脚本汇总

| 库 | 工作流类型 | Linux / macOS | Windows |
|----|-----------|---------------|---------|
| [rapidjson](https://github.com/Tencent/rapidjson) | shim 层（extern-C 包装） | [`verify-rapidjson-ffi.sh`](verify-rapidjson-ffi.sh) | [`verify-rapidjson-ffi.ps1`](verify-rapidjson-ffi.ps1) |
| [tinyxml2](https://github.com/leethomason/tinyxml2) | 直接 C++（hicc 直出） | [`verify-tinyxml2-ffi.sh`](verify-tinyxml2-ffi.sh) | [`verify-tinyxml2-ffi.ps1`](verify-tinyxml2-ffi.ps1) |
| [pugixml](https://github.com/zeux/pugixml) | 直接 C++（hicc 直出） | [`verify-pugixml-ffi.sh`](verify-pugixml-ffi.sh) | [`verify-pugixml-ffi.ps1`](verify-pugixml-ffi.ps1) |
| [fmtlib / {fmt}](https://github.com/fmtlib/fmt) | 直接 C++（多 .cc 源文件） | [`verify-fmtlib-ffi.sh`](verify-fmtlib-ffi.sh) | [`verify-fmtlib-ffi.ps1`](verify-fmtlib-ffi.ps1) |
| [nlohmann/json](https://github.com/nlohmann/json) | header-only（驱动文件） | [`verify-nlohmann-json-ffi.sh`](verify-nlohmann-json-ffi.sh) | [`verify-nlohmann-json-ffi.ps1`](verify-nlohmann-json-ffi.ps1) |
| [magic_enum](https://github.com/Neargye/magic_enum) | header-only（驱动文件） | [`verify-magic-enum-ffi.sh`](verify-magic-enum-ffi.sh) | [`verify-magic-enum-ffi.ps1`](verify-magic-enum-ffi.ps1) |
| [toml++](https://github.com/marzer/tomlplusplus) | header-only（驱动文件） | [`verify-tomlplusplus-ffi.sh`](verify-tomlplusplus-ffi.sh) | [`verify-tomlplusplus-ffi.ps1`](verify-tomlplusplus-ffi.ps1) |
| [sqlite3](https://www.sqlite.org/) | 系统库（extern-C 接口） | [`verify-sqlite3-ffi.sh`](verify-sqlite3-ffi.sh) | [`verify-sqlite3-ffi.ps1`](verify-sqlite3-ffi.ps1) |

**工作流类型说明：**

| 类型 | 说明 | 典型库 |
|------|------|--------|
| **shim 层** | 需手写 `extern "C"` 包装层（shim），工具提取 shim 函数生成 `import_lib!` | rapidjson |
| **直接 C++（hicc 直出）** | 直接对库源码运行，工具识别命名空间类生成 `import_class!` | tinyxml2、pugixml、fmtlib |
| **header-only（驱动文件）** | 无 .cpp 源文件，需编写含库头文件的临时驱动 .cpp | nlohmann/json、magic_enum、toml++ |
| **系统库** | 通过系统包管理器安装，无需子模块，通过 `rustc-link-lib` 链接 | sqlite3 |

---

## 二、快速开始

### Linux / macOS

```bash
# ① 安装系统依赖（Ubuntu/Debian）
sudo apt-get install -y clang libclang-dev g++ libstdc++-14-dev binutils git curl

# ② 安装 Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# ③ 初始化所需子模块（以 tinyxml2 为例）
git submodule update --init references/tinyxml2

# ④ 运行验证脚本
bash usage/verify-tinyxml2-ffi.sh

# ⑤ 已安装 cpp2rust-demo 时跳过重新编译
SKIP_INSTALL=1 bash usage/verify-tinyxml2-ffi.sh
```

### Windows（PowerShell）

```powershell
# ① 安装工具（以 winget 为例）
winget install Git.Git LLVM.LLVM Rustlang.Rustup
# 或通过 MSYS2：
# pacman -S mingw-w64-x86_64-gcc mingw-w64-x86_64-llvm

# ② 初始化所需子模块
git submodule update --init references/tinyxml2

# ③ 运行验证脚本（在 PowerShell 7+ 或 Windows PowerShell 5.1 中执行）
.\usage\verify-tinyxml2-ffi.ps1

# ④ 跳过重新安装
$env:SKIP_INSTALL = "1"; .\usage\verify-tinyxml2-ffi.ps1
```

> **Windows 注意**：
> - 脚本会自动检测 `cl.exe`（MSVC）、`g++`（MinGW/MSYS2）、`clang++`（LLVM），优先使用 cl.exe
> - 符号验证优先使用 `llvm-nm`，其次 `nm`；MSVC 环境可使用 `dumpbin`
> - Windows 上 `.opts` 文件不写出，`build.rs` 注入始终走方案 B（脚本自动处理）

---

## 三、各库子模块初始化

大多数库通过 git 子模块引入，运行脚本前需先初始化对应子模块：

```bash
# 初始化单个子模块
git submodule update --init references/tinyxml2
git submodule update --init references/pugixml
git submodule update --init references/fmtlib
git submodule update --init references/nlohmann-json
git submodule update --init references/magic_enum
git submodule update --init references/tomlplusplus
# sqlite3: 无需子模块，安装系统包即可
sudo apt-get install -y libsqlite3-dev   # Ubuntu/Debian
# pacman -S mingw-w64-x86_64-sqlite3     # MSYS2 Windows

# 一次性初始化所有子模块
git submodule update --init
```

rapidjson 的 shim 参考实现位于 `references/rapidjson-refactoring/`（vendored 目录，无需子模块初始化）。

---

## 四、所有脚本共同的 §0–§7 结构

每个脚本均按以下阶段执行，保证风格和验证深度一致：

```
§ 0. 环境检查          检测编译器 / cargo / nm / libclang
§ 1. 安装工具          cargo install --git ... --bin cpp2rust-demo（SKIP_INSTALL=1 跳过）
§ 2. 定位库源码        验证子模块或系统头文件存在
§ 3. 编译目标文件      g++ / cl 预先编译，为 §6 nm 符号验证准备 .o/.obj 文件
§ 4. cpp2rust-demo init  拦截编译命令，生成 hicc 三段式 FFI 脚手架
§ 5. cpp2rust-demo merge  整理输出目录结构
§ 5a. 校验 build.rs    方案A：工具自动注入（Linux）；方案B：脚本兜底补全（Windows/fallback）
§ 5b. cargo check      验证生成的 Rust 项目语法与类型正确
§ 5c. cargo test       验证生成的冒烟测试可编译 / 链接 / 运行（有 smoke.rs 时）
§ 6. FFI 验证          nm 符号、import_class!/import_lib! 计数、link_name 一致性、预处理文件统计
§ 7. 生成结果汇报      汇总捕获文件数 / FFI 绑定数 / 降级标记统计
```

---

## 五、可配置环境变量

所有脚本均支持以下环境变量（Linux）或 `$env:` 前缀变量（Windows PowerShell）：

| 变量 | 默认值 | 说明 |
|------|--------|------|
| `FEATURE` | 库名_ffi（如 `tinyxml2_ffi`） | feature 名称，影响输出目录 |
| `SKIP_INSTALL` | `0` | 置 `1` 跳过 `cargo install`（已安装时使用） |

### 各库默认 FEATURE 名

| 库 | 默认 FEATURE |
|----|-------------|
| rapidjson | `rapidjson_shim` |
| tinyxml2 | `tinyxml2_ffi` |
| pugixml | `pugixml_ffi` |
| fmtlib | `fmtlib_ffi` |
| nlohmann/json | `nlohmann_json_ffi` |
| magic_enum | `magic_enum_ffi` |
| toml++ | `tomlplusplus_ffi` |
| sqlite3 | `sqlite3_ffi` |

---

## 六、rapidjson 详细说明（shim 工作流）

rapidjson 是纯 C++ 库（无 `extern "C"` 导出），需要先编写 shim 层再运行工具。

### 脚本执行阶段（仅 rapidjson 特有差异）

```
§ 2. 定位本地 shim 文件
     references/rapidjson-refactoring/rapidjson_sys/shim/（已包含 10 个子系统 shim）
§ 3. 编译 shim 目标文件（供 §6c 符号交叉比对）
§ 6c. shim 函数名交叉比对
     从生成代码提取 #[cpp(func = "...")] 标注的 shim 函数名与 nm 符号表比对
```

### 查看 rapidjson 生成结果

```bash
# 生成产物目录
ls .cpp2rust/rapidjson_shim/rust/src/

# 查找 import_lib! 绑定
grep -rn "hicc::import_lib!" .cpp2rust/rapidjson_shim/rust/src/

# 查找降级标记
grep -rn "cpp2rust-todo" .cpp2rust/rapidjson_shim/rust/src/
```

---

## 七、SKILL 工作流完整说明

### 什么是 cpp2rust-convert Skill？

`.github/skills/cpp2rust-convert.md` 是一个 GitHub Copilot Agent Skill 文件，
当你在任意 C++ 项目根目录请求 FFI 转换时，Copilot 会自动读取此 Skill 并引导你完成转换。

### 前提条件

```bash
# 1. 安装 cpp2rust-demo
cargo install --git https://github.com/LuuuXXX/cpp2rust-demo --locked --bin cpp2rust-demo

# 2. 安装系统依赖（Ubuntu/Debian）
sudo apt-get install -y clang libclang-dev g++ libstdc++-14-dev
```

### SKILL 交互式流程（以 tinyxml2 为例）

```
# 在 references/tinyxml2 目录打开 GitHub Copilot 对话

Agent: 请输入 feature 名称：
User:  tinyxml2_ffi

Agent: 请输入构建命令：
User:  g++ -c -std=c++11 tinyxml2.cpp

# Agent 自动执行：
cpp2rust-demo init --feature tinyxml2_ffi -- g++ -c -std=c++11 tinyxml2.cpp
cpp2rust-demo merge --feature tinyxml2_ffi
```

### SKILL vs. CLI 脚本对比

| 维度 | CLI 脚本（.sh / .ps1） | SKILL（GitHub Copilot） |
|------|------------------------|--------------------------|
| 交互方式 | 全自动批处理，无需人工干预 | 对话式，逐步引导 |
| 符号验证 | 内置 nm/dumpbin 四步验证 | 不含符号验证（仅转换） |
| 结果解读 | 原始文件/符号输出 | Agent 自然语言解释 todo 标记 |
| 适用场景 | CI/CD、批量验证、符号审计 | 首次使用、探索性转换 |

---

## 八、常见问题

**Q: 脚本提示"未找到命令：cpp2rust-demo"**

```bash
cargo install --git https://github.com/LuuuXXX/cpp2rust-demo --locked --bin cpp2rust-demo
export PATH="$HOME/.cargo/bin:$PATH"
```

**Q: 生成的 .rs 文件为空 / 只有 lib.rs**

可能原因：
1. 编译拦截未捕获到 `.cpp` 文件（检查 `LD_PRELOAD` 是否生效）
2. 所有编译单元已是最新，Make/CMake 跳过了重编译（删除构建产物重试）
3. header-only 库：驱动文件编译超时（可调大超时或检查 libclang 是否安装）

**Q: Windows 上 cargo check 报 "Cannot open include file"**

所有注入 `build.rs` 的 include 路径须为 Windows 原生路径（`D:/a/...`）。
脚本的 `ConvertTo-BuildPath` 函数已通过 `cygpath -m` 自动处理此问题。

**Q: 降级标记 `[OP]`/`[VA]`/`[LM]` 如何处理**

| 标记 | 原因 | 手动操作 |
|------|------|---------|
| `[OP]` | 运算符重载 | 为生成的 `{class}_add` 等命名 shim 实现 `std::ops::*` trait |
| `[VA]` | 可变参数模板 | 在 `hicc::cpp!` 中添加新的参数组合 |
| `[LM]` | 有状态 Lambda / std::function | 手动编写 trampoline |

```bash
# 查找所有降级标记
grep -rn "cpp2rust-todo" .cpp2rust/<FEATURE>/rust/src/
```

### 可配置环境变量

| 变量 | 默认值 | 说明 |
|------|--------|------|
| `RAPIDJSON_DIR` | `/tmp/rapidjson-ffi-demo` | rapidjson 克隆目录 |
| `FEATURE` | `rapidjson_tests` | cpp2rust-demo feature 名称 |
| `SKIP_INSTALL` | `0` | 置 `1` 跳过 `cargo install`（已安装时加速） |

示例：

```bash
# 指定克隆目录和 feature 名称
RAPIDJSON_DIR=/opt/rapidjson FEATURE=rj_tests bash usage/verify-rapidjson-ffi.sh

# 已安装 cpp2rust-demo 时跳过 cargo install
SKIP_INSTALL=1 bash usage/verify-rapidjson-ffi.sh
```

### 脚本执行阶段

```
§ 0. 环境检查 & 依赖安装
     检测 git / cmake / g++ / cargo / nm / objdump / libclang
     自动搜索或安装 GTest 源码（FindGTestSrc.cmake 搜索路径）

§ 1. 安装 cpp2rust-demo
     cargo install --git https://github.com/LuuuXXX/cpp2rust-demo --locked

§ 2. git clone rapidjson
     git clone --depth 1 https://github.com/Tencent/rapidjson.git <RAPIDJSON_DIR>
     （目录已存在则 git pull）

§ 3. 配置构建环境（CMake）
     cmake -B build -DCMAKE_BUILD_TYPE=Debug
           -DRAPIDJSON_BUILD_TESTS=ON       ← GTest 存在时
           -DGTEST_SOURCE_DIR=<gtest>

§ 4. cpp2rust-demo init（编译拦截 + FFI 脚手架生成）
     LD_PRELOAD 注入 cmake --build，捕获所有 .cpp2rust 预处理文件
     输出：<RAPIDJSON_DIR>/.cpp2rust/<FEATURE>/rust/src/

§ 5. cpp2rust-demo merge（整理输出目录）
     src/ → src.1/（备份）+ src.2/（模块化）+ src → src.2（symlink）

§ 5a. 校验 build.rs（方案 A：工具自动注入头路径 / C++ 标准 / 实现 .cpp）
     init 已从 .opts 落盘编译元数据（meta/build-meta.json），生成的 build.rs
     自动注入 cc_build.include/std/file；脚本仅校验，未自包含时才退回就地补全

§ 6. 符号验证（四子步）
     6a. nm --demangle 查看编译产物 C++ mangled 符号
     6b. 生成 Rust 代码中的 FFI 声明（hicc::cpp! / import_class! / import_lib!）
     6c. shim 函数名 vs. nm 符号交叉比对（nm 结果一次性缓存）
     6d. .cpp2rust 预处理文件完整性检查（行数统计）

§ 7. 生成结果汇报
     捕获文件数 / 生成 Rust 文件数 / 降级标记（[OP][VA][LM]）统计
```

### 输出目录结构

```
<RAPIDJSON_DIR>/
└── .cpp2rust/<FEATURE>/
    ├── c/                          # 预处理文件（.cpp2rust 后缀）
    │   └── test/unittest/
    │       ├── bigintegertest.cpp.cpp2rust
    │       ├── documenttest.cpp.cpp2rust
    │       └── ...
    ├── meta/
    │   ├── build_cmd.txt           # 原始构建命令
    │   └── init-interface-report.md
    └── rust/
        ├── src.1/                  # init 输出原始备份（merge 后生成）
        ├── src.2/                  # merge 整理后的模块化结构
        └── src -> src.2            # symlink（始终指向最新输出）
            ├── lib.rs
            ├── bigintegertest.rs
            ├── documenttest.rs
            └── ...
```

### 查看生成结果

```bash
# 查看 lib.rs（所有模块入口）
cat <RAPIDJSON_DIR>/.cpp2rust/<FEATURE>/rust/src/lib.rs

# 查看某个文件的 hicc 三段式代码
cat <RAPIDJSON_DIR>/.cpp2rust/<FEATURE>/rust/src/bigintegertest.rs

# 查找所有包含 import_class! / import_lib! 的文件
find <RAPIDJSON_DIR>/.cpp2rust/<FEATURE>/rust/src -name '*.rs' \
    | xargs grep -l 'import_class\|import_lib'

# 查看所有降级标记
grep -rn "cpp2rust-todo" <RAPIDJSON_DIR>/.cpp2rust/<FEATURE>/rust/src/
```

---

