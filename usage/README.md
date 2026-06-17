# usage — 本地验证脚本与 SKILL 使用文档

本目录包含对真实 C++ 项目进行 cpp2rust-demo FFI 转换的**所有可用方式**。除针对
rapidjson 的端到端脚本外，还为 E2E 已覆盖的 7 个真实库各提供了一份**可本地直接执行**的
验证脚本，并由共享库 `lib/verify-common.sh` 复用七阶段骨架、`verify-all.sh` 统一入口编排。

| 文件 | 类型 | 说明 |
|------|------|------|
| [`verify-rapidjson-ffi.sh`](verify-rapidjson-ffi.sh) | 独立脚本 | rapidjson 全自动 Shell 脚本（CLI 方式，CMake + GTest 拦截） |
| [`verify-tinyxml2-ffi.sh`](verify-tinyxml2-ffi.sh) | 实现 .cpp | tinyxml2，源 `tinyxml2.cpp`（子模块 `references/tinyxml2`） |
| [`verify-pugixml-ffi.sh`](verify-pugixml-ffi.sh) | 实现 .cpp | pugixml，源 `src/pugixml.cpp`（子模块 `references/pugixml`） |
| [`verify-sqlite3-ffi.sh`](verify-sqlite3-ffi.sh) | 系统头 | sqlite3，C++ 驱动包装系统 `sqlite3.h`（纯 extern "C" 接口） |
| [`verify-nlohmann-json-ffi.sh`](verify-nlohmann-json-ffi.sh) | header-only | nlohmann/json，最小驱动 include 库头触发解析 |
| [`verify-fmtlib-ffi.sh`](verify-fmtlib-ffi.sh) | 实现 .cpp | fmt，源 `src/format.cc`、`src/os.cc`（子模块 `references/fmtlib`） |
| [`verify-magic-enum-ffi.sh`](verify-magic-enum-ffi.sh) | header-only | magic_enum，最小驱动 include 库头 |
| [`verify-tomlplusplus-ffi.sh`](verify-tomlplusplus-ffi.sh) | header-only | toml++，最小驱动 include 库头 |
| [`lib/verify-common.sh`](lib/verify-common.sh) | 共享库 | 七阶段骨架与 `vc_*` 通用函数，由各 per-library 脚本 `source` |
| [`verify-all.sh`](verify-all.sh) | 统一入口 | 顺序（或按 `LIBS=` 过滤）调用全部 `verify-*-ffi.sh`，汇总通过/跳过/失败矩阵 |
| 本文档（README.md） | 文档 | 脚本用法 + SKILL 交互式工作流完整说明 |

> **header-only vs. 实现 .cpp 两类**：带真实实现 .cpp 的库（tinyxml2/pugixml/fmtlib）
> 直接拦截库自身编译单元；header-only 库（nlohmann-json/magic-enum/tomlplusplus）生成
> 最小驱动 .cpp，`#include` 库头触发解析压测，驱动类方法仅声明、签名只用标量/std。
> sqlite3 为纯 `extern "C"` 接口，经一层 C++ 驱动包装系统头，工具可能不生成绑定（预期内）。

SKILL 文件本身位于 [`.github/skills/cpp2rust-convert.md`](../.github/skills/cpp2rust-convert.md)，由 GitHub Copilot Agent 自动读取，无需手动调用。

---

## 〇、真实项目本地验证脚本（verify-*-ffi.sh + verify-all.sh）

这 7 份脚本与既有 8 个 `.github/workflows/e2e-*.yml` 形成「CI 集成测试 ↔ 本地验证脚本」
的对照关系，可在本地 Ubuntu 直接执行，产出 init/merge/cargo check/test 结果与符号汇报。

### 快速开始

```bash
# 系统依赖（Ubuntu/Debian，首次执行前安装一次）
sudo apt-get install -y clang libclang-dev g++ libstdc++-14-dev cmake \
                        libsqlite3-dev binutils git curl
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 跑单个库（子模块缺失时自动 git submodule update --init，失败则 warn 跳过）
bash usage/verify-tinyxml2-ffi.sh

# 已安装 cpp2rust-demo 时跳过 cargo install 加速
SKIP_INSTALL=1 bash usage/verify-nlohmann-json-ffi.sh

# 一次性跑全部库，末尾输出 PASS/SKIP/FAIL 矩阵（任一库 FAIL → 非零退出供 CI 捕获）
bash usage/verify-all.sh

# 仅跑指定库（空格分隔）
LIBS="tinyxml2 sqlite3" bash usage/verify-all.sh
```

### 通用环境变量（所有 verify-*-ffi.sh 共享，定义于 lib/verify-common.sh）

| 变量 | 默认值 | 说明 |
|------|--------|------|
| `SKIP_INSTALL` | `0` | 置 `1` 跳过 `cargo install`（已安装时加速，CI 常用） |
| `FEATURE` | `<lib>_ffi` | cpp2rust-demo feature 名称 |
| `CXX_STD` | `c++17` | 驱动/源编译与生成 build.rs 注入的 C++ 标准 |
| `STRICT_CARGO` | `0` | 置 `1` 时 `cargo check`/`cargo test` 失败也计入错误（CI 严格模式） |

> 默认情况下，`cargo check`/`cargo test` 失败仅**软报告**（汇总矩阵标 ⚠），不中断脚本，
> 因真实库可能触发工具尚未实现的 codegen 路径；结构性失败（未生成任何 FFI 绑定）才计错。

### 每库专属环境变量

| 脚本 | 变量 | 默认值 | 说明 |
|------|------|--------|------|
| `verify-sqlite3-ffi.sh` | `REQUIRE_SYSTEM_HEADER` | `/usr/include/sqlite3.h` | 系统 sqlite3 头路径，缺失则跳过 |

### 设计要点

- 脚本只复用现有能力（cpp2rust-demo init/merge、cargo、nm、git 子模块），不引入新依赖、不改 `src/`。
- 各库默认从对应 `references/<lib>` 子模块取源；子模块未初始化时自动 `git submodule update --init`（与 Makefile `submodules` 目标一致），失败则 `warn` 跳过、不中断。
- init/merge 在 `/tmp` 下的扁平工作目录执行，保证生成单元名与 `link_name` 为纯文件名，且不污染仓库与子模块。

---

## 一、快速开始（rapidjson — 选择其中一种方式）

### 方式 A：运行 Shell 脚本（全自动）

```bash
# 系统依赖（Ubuntu/Debian，首次执行前安装一次）
sudo apt-get install -y clang libclang-dev g++ libstdc++-14-dev cmake \
                        libgtest-dev binutils git curl
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 运行验证脚本
bash usage/verify-rapidjson-ffi.sh
```

### 方式 B：通过 GitHub Copilot Agent Skill（对话式）

```
# 在 rapidjson 项目根目录打开 Copilot 对话，输入：
"帮我用 cpp2rust-demo 把这个 C++ 项目转换为 Rust FFI"
```

Agent 会自动引导你完成 feature 命名 → 构建命令 → init → merge → 结果汇报。详见下文 [§ 4. SKILL 工作流](#4-skill-工作流完整说明)。

---

## 二、脚本详细说明（verify-rapidjson-ffi.sh）

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

## 三、符号验证说明

脚本的 § 6 阶段会执行四步符号验证，帮助确认 FFI 导出是否符合预期：

### 6a — 编译产物符号

使用 `nm --demangle` 查看 rapidjson 编译产物（`.o`/`.so`/`.a`）中的 C++ mangled 符号，
确认测试相关类型（如 `BigInteger`、`Document`、`Reader`）已被编译。

### 6b — 生成 Rust 代码 FFI 声明

检查 cpp2rust-demo 生成的 `.rs` 文件中的三段式声明：

| 声明块 | 内容 |
|--------|------|
| `hicc::cpp! { ... }` | C++ shim 实现（ctor/dtor/operator 等必要 shim） |
| `hicc::import_class! { ... }` | 类方法绑定（hicc 处理虚表 dispatch） |
| `hicc::import_lib! { ... }` | 全局/关联函数绑定 |

### 6c — shim 函数名交叉比对

从生成代码中提取 `#[cpp(func = "...")]` 标注的 shim 函数名，与 `nm` 符号表交叉比对：

- ✓ **在目标文件中找到**：该 C shim 函数已在编译产物中存在
- ? **未在目标文件中直接找到**：该 shim 函数由 `hicc::cpp!` 宏在 Rust 构建时展开，正常现象

### 6d — 预处理文件完整性

统计各 `.cpp2rust` 文件行数，确认预处理捕获内容完整（行数越多说明捕获越充分）。

---

## 四、SKILL 工作流完整说明

### 什么是 cpp2rust-convert Skill？

`.github/skills/cpp2rust-convert.md` 是一个 GitHub Copilot Agent Skill 文件，
当你在任意 C++ 项目根目录请求 FFI 转换时，Copilot 会自动读取此 Skill 并引导你完成转换。

### 前提条件

```bash
# 1. 安装 cpp2rust-demo
cargo install --git https://github.com/LuuuXXX/cpp2rust-demo --locked

# 2. 安装系统依赖
sudo apt-get install -y clang libclang-dev g++ libstdc++-14-dev

# 3. 克隆 rapidjson 并完成 CMake 配置（CMake 项目需提前配置）
git clone --depth 1 https://github.com/Tencent/rapidjson.git /tmp/rapidjson
cd /tmp/rapidjson
cmake -B build -DCMAKE_BUILD_TYPE=Debug \
      -DRAPIDJSON_BUILD_TESTS=ON \
      -DGTEST_SOURCE_DIR=/usr/src/gtest
```

### SKILL 交互式流程

```
# 在 /tmp/rapidjson 目录打开 GitHub Copilot 对话

Agent: 请输入 feature 名称（直接回车跳过则使用默认值 `default`）：
User:  rapidjson_tests
→ FEATURE=rapidjson_tests

Agent: 请输入构建命令（例如 `make -j$(nproc)`、`cmake --build build -- -j$(nproc)` 等）：
User:  cmake --build build -- -j$(nproc)
→ BUILD_CMD=cmake --build build -- -j$(nproc)

# Agent 自动执行：
cpp2rust-demo init --feature rapidjson_tests -- cmake --build build -- -j$(nproc)
cpp2rust-demo merge --feature rapidjson_tests

# Agent 汇报结果：生成文件数、降级标记统计、下一步建议
```

### SKILL vs. CLI 脚本对比

| 维度 | CLI 脚本（verify-rapidjson-ffi.sh） | SKILL（GitHub Copilot） |
|------|--------------------------------------|--------------------------|
| 交互方式 | 全自动批处理，无需人工干预 | 对话式，逐步引导 |
| feature 设置 | 环境变量 `FEATURE=...` | Agent 询问后自动填充 |
| 构建命令 | 脚本内 cmake 参数 | Agent 询问后自动填充 |
| GTest 检测 | 自动搜索安装 | 需提前手动配置 |
| 符号验证 | 内置 nm/objdump 四步验证 | 不含符号验证（仅转换） |
| 结果解读 | 原始文件/符号输出 | Agent 自然语言解释 todo 标记 |
| 适用场景 | CI/CD、批量验证、符号审计 | 首次使用、探索性转换、项目不熟悉时 |

### 降级标记处理（SKILL 汇报后的后续工作）

SKILL 完成后，若生成代码含降级标记，按以下方式处理：

| 标记 | 原因 | 手动操作 |
|------|------|---------|
| `[OP]` | 运算符重载（C ABI 无运算符符号） | 为生成的 `{class}_add` 等命名 shim 实现 `std::ops::*` trait |
| `[VA]` | 可变参数模板（编译期展开） | 按需在 `hicc::cpp!` 中添加新的参数数量/类型组合 |
| `[LM]` | 有状态 Lambda / std::function | 若需 Rust 闭包 → C++ 回调，手动编写 trampoline |

```bash
# 查找所有降级标记
grep -rn "cpp2rust-todo" /tmp/rapidjson/.cpp2rust/rapidjson_tests/rust/src/
```

---

## 五、常见问题

**Q: 脚本提示"未找到命令：cpp2rust-demo"**

```bash
# 安装
cargo install --git https://github.com/LuuuXXX/cpp2rust-demo --locked
# 确认 ~/.cargo/bin 在 PATH 中
export PATH="$HOME/.cargo/bin:$PATH"
```

**Q: GTest 未找到，测试目标被跳过**

```bash
sudo apt-get install -y libgtest-dev
# 或手动指定：
GTEST_SOURCE_DIR=/path/to/googletest bash usage/verify-rapidjson-ffi.sh
```

**Q: 生成的 .rs 文件为空 / 只有 lib.rs**

可能原因：
1. 编译拦截未捕获到 `.cpp` 文件（检查 `LD_PRELOAD` 是否生效）
2. CMake 使用了 Ninja 等不触发 `execve` 的构建器（改用 `cmake --build -- -j4 VERBOSE=1`）
3. 所有编译单元已是最新，Make 跳过了重编译（执行 `cmake --build --clean-first`）

**Q: 符号验证 6c 全部显示"?"**

这是正常现象。`hicc::cpp!` 宏中的 shim 函数在 Rust 构建时才被编译为 C++ 代码，
不会出现在 rapidjson 原始编译产物的 `.o` 文件中。
只有 `bigintegertest_ffi.cpp`（rapidjson_sys 项目的 shim）中的函数才会提前出现在目标文件里。

**Q: 如何只转换部分文件（而非全部 rapidjson 测试文件）**

脚本在非终端（管道/CI）环境运行，`cpp2rust-demo` 检测到 stdin 不是 TTY 时会自动全选所有文件。
若需交互式选择，需在**交互式终端**中手动执行以下命令：

```bash
cd <RAPIDJSON_DIR>
cpp2rust-demo init --feature "${FEATURE}" -- cmake --build build -- -j$(nproc)
```

在终端中运行时，工具会弹出多选界面，按空格键切换选中状态，回车确认。
