# usage — 本地验证脚本与 SKILL 使用文档

本目录提供 cpp2rust-demo 的**可本地直接执行**端到端验证脚本，覆盖两类工作流：

- **直出工作流（默认）**：cpp2rust-demo 默认对命名空间类与自由函数「直出」safe FFI
  （`import_class!` / `import_lib!`），**无需手写 extern-C shim**。绝大多数 C++ 库
  （tinyxml2 / pugixml / nlohmann-json / fmtlib / magic_enum / tomlplusplus）以及纯
  C 接口库（sqlite3）走这条路径。
- **shim 工作流**：对「纯 C++、无可导出类」或需要精确控制 ABI 的场景，可先手写一层
  extern-C 包装（shim），再用 cpp2rust-demo 提取。rapidjson 即以此为例。

> 何时直出、何时写 shim 的决策表见 [`../docs/WORKFLOW.md`](../docs/WORKFLOW.md)。

## 脚本清单

| 脚本 | 工作流 | 库 / 说明 | 关键环境变量 |
|------|--------|-----------|--------------|
| [`lib/common.sh`](lib/common.sh) | — | 公共函数库（日志 / 安装 / init+merge / cargo check / 统计 / 汇报），被各脚本 `source` | — |
| [`verify-tinyxml2.sh`](verify-tinyxml2.sh) | 直出 | tinyxml2（编译 `tinyxml2.cpp` 触发拦截） | `FEATURE` `SKIP_INSTALL` `TINYXML2_DIR` |
| [`verify-pugixml.sh`](verify-pugixml.sh) | 直出 | pugixml（编译 `src/pugixml.cpp`） | `FEATURE` `SKIP_INSTALL` `PUGIXML_DIR` |
| [`verify-nlohmann-json.sh`](verify-nlohmann-json.sh) | 直出 | nlohmann/json（header-only 最小驱动） | `FEATURE` `SKIP_INSTALL` `NLOHMANN_DIR` |
| [`verify-fmtlib.sh`](verify-fmtlib.sh) | 直出 | {fmt}（编译 `src/format.cc` `src/os.cc`） | `FEATURE` `SKIP_INSTALL` `FMTLIB_DIR` |
| [`verify-magic-enum.sh`](verify-magic-enum.sh) | 直出 | magic_enum（header-only 最小驱动） | `FEATURE` `SKIP_INSTALL` `MAGIC_ENUM_DIR` |
| [`verify-tomlplusplus.sh`](verify-tomlplusplus.sh) | 直出 | toml++（header-only 最小驱动） | `FEATURE` `SKIP_INSTALL` `TOMLPP_DIR` |
| [`verify-sqlite3.sh`](verify-sqlite3.sh) | C 接口 | sqlite3（系统头 `extern "C"` wrapper） | `FEATURE` `SKIP_INSTALL` `SQLITE3_HEADER` |
| [`verify-all.sh`](verify-all.sh) | 直出 | 顺序执行上述 7 个脚本并汇总总表 | `ONLY` `SKIP_INSTALL` |
| [`verify-rapidjson-ffi.sh`](verify-rapidjson-ffi.sh) | shim | rapidjson（基于 references 的 extern-C shim 层） | `FEATURE` `SKIP_INSTALL` |
| 本文档（README.md） | — | 脚本用法 + SKILL 交互式工作流完整说明 | — |

### 直出工作流脚本示例

```bash
# 单个库（已安装 cpp2rust-demo 时跳过 cargo install）
SKIP_INSTALL=1 bash usage/verify-tinyxml2.sh

# 全部 7 个直出/ C 接口库，汇总成总表
SKIP_INSTALL=1 bash usage/verify-all.sh

# 只跑子集
SKIP_INSTALL=1 ONLY=tinyxml2,fmtlib bash usage/verify-all.sh
```

每个直出脚本统一流程：检测/初始化对应子模块 → 安装工具（支持 `SKIP_INSTALL=1`）→
构造最小构建命令（`g++ -c` 编译该库实现单元或最小驱动 .cpp，触发 LD_PRELOAD 拦截）→
`init --feature <lib>` → `merge` → 校验生成 `build.rs` → `cargo check` →（若生成
smoke 测试）`cargo test` → 统计 `import_class!`/`import_lib!`/降级标记 → 结果汇报。
所有脚本使用 `set -euo pipefail` 并以全局 `SCRIPT_ERRORS` 计数，保证 CI/本地一致失败退出。

SKILL 文件本身位于 [`.github/skills/cpp2rust-convert.md`](../.github/skills/cpp2rust-convert.md)，由 GitHub Copilot Agent 自动读取，无需手动调用。

---

## 一、快速开始（选择其中一种方式）

### 方式 A：运行 Shell 脚本（全自动）

```bash
# 系统依赖（Ubuntu/Debian，首次执行前安装一次）
sudo apt-get install -y clang libclang-dev g++ libstdc++-14-dev cmake \
                        libgtest-dev libsqlite3-dev binutils git curl
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 直出工作流：验证 tinyxml2（或 verify-all.sh 批量）
bash usage/verify-tinyxml2.sh

# shim 工作流：验证 rapidjson
bash usage/verify-rapidjson-ffi.sh
```

### 方式 B：通过 GitHub Copilot Agent Skill（对话式）

```
# 在 rapidjson 项目根目录打开 Copilot 对话，输入：
"帮我用 cpp2rust-demo 把这个 C++ 项目转换为 Rust FFI"
```

Agent 会自动引导你完成 feature 命名 → 构建命令 → init → merge → 结果汇报。详见下文 [§ 4. SKILL 工作流](#4-skill-工作流完整说明)。

---

## 二、脚本详细说明（verify-rapidjson-ffi.sh — shim 工作流）

> **工作流说明**：rapidjson 是纯 C++ 库（无可导出类）。本脚本以仓库内置的
> extern-C shim 参考实现（`references/rapidjson-refactoring/rapidjson_sys/shim/`）为
> 输入，演示 **shim 工作流**：编译 shim → init 拦截 → merge → cargo check/test →
> 符号交叉比对。它**不**克隆 rapidjson、也不调用 CMake——shim 与 rapidjson 头文件
> 均来自仓库子目录。

### 可配置环境变量

| 变量 | 默认值 | 说明 |
|------|--------|------|
| `FEATURE` | `rapidjson_shim` | cpp2rust-demo feature 名称 |
| `SKIP_INSTALL` | `0` | 置 `1` 跳过 `cargo install`（已安装时加速） |

示例：

```bash
# 自定义 feature 名称
FEATURE=rj_shim bash usage/verify-rapidjson-ffi.sh

# 已安装 cpp2rust-demo 时跳过 cargo install
SKIP_INSTALL=1 bash usage/verify-rapidjson-ffi.sh
```

### 脚本执行阶段

```
§ 0. 环境检查
     检测 git / g++ / cargo / nm / libclang

§ 1. 安装 cpp2rust-demo
     cargo install --git https://github.com/LuuuXXX/cpp2rust-demo --locked
     （SKIP_INSTALL=1 时跳过）

§ 2. 定位本地 shim 文件
     references/rapidjson-refactoring/rapidjson_sys/shim/（extern-C 包装层）
     rapidjson 头文件：references/rapidjson-refactoring/rapidjson_legacy/include

§ 3. 编译 shim 目标文件（供后续 nm 符号验证）
     g++ -c 每个 shim .cpp → .o

§ 4. cpp2rust-demo init（编译拦截 + FFI 脚手架生成）
     LD_PRELOAD 拦截 g++ 编译 shim，捕获 .cpp2rust 预处理文件
     输出：<REPO>/.cpp2rust/<FEATURE>/rust/src/

§ 5. cpp2rust-demo merge（整理输出目录）
     src/ → src.1/（备份）+ src.2/（模块化）+ src → src.2（symlink）

§ 5a. 校验 build.rs（方案 A：工具自动注入头路径 / C++ 标准 / 实现 .cpp）
      init 已从 .opts 落盘编译元数据（meta/build-meta.json），生成的 build.rs
      自动注入 cc_build.include/std/file；脚本仅校验，未自包含时才退回就地补全

§ 5b. cargo check（验证生成的 Rust 项目可编译）
§ 5c. cargo test（若生成 tests/smoke.rs）

§ 6. 符号验证（多子步）
     6a. nm 查看 shim 目标文件 extern-C 符号
     6b. 生成 Rust 代码中的 FFI 声明（hicc::cpp! / import_class! / import_lib!）
     6c. shim 函数名 vs. nm 符号交叉比对（nm 结果一次性缓存）
     6d. .cpp2rust 预处理文件大小统计
     6e. struct/class 前缀 & restrict 清理验证

§ 7. 生成结果汇报
     捕获文件数 / 生成 Rust 文件数 / 降级标记（[OP][VA][LM]）统计
```

### 输出目录结构

```
<REPO>/                             # cpp2rust-demo 仓库根目录
└── .cpp2rust/<FEATURE>/
    ├── c/                          # 预处理文件（.cpp2rust 后缀，来自 shim/*.cpp）
    │   └── ...
    │       ├── value_ffi.cpp.cpp2rust
    │       ├── document_ffi.cpp.cpp2rust
    │       └── ...
    ├── meta/
    │   ├── build_cmd.txt           # 原始构建命令
    │   ├── build-meta.json         # 编译元数据（include/std/实现 .cpp）
    │   └── init-interface-report.md
    └── rust/
        ├── src.1/                  # init 输出原始备份（merge 后生成）
        ├── src.2/                  # merge 整理后的模块化结构
        └── src -> src.2            # symlink（始终指向最新输出）
            ├── lib.rs
            ├── value_ffi.rs
            ├── document_ffi.rs
            └── ...
```

### 查看生成结果

```bash
# 查看 lib.rs（所有模块入口）
cat <REPO>/.cpp2rust/<FEATURE>/rust/src/lib.rs

# 查找所有包含 import_class! / import_lib! 的文件
find <REPO>/.cpp2rust/<FEATURE>/rust/src -name '*.rs' \
    | xargs grep -l 'import_class\|import_lib'

# 查看所有降级标记
grep -rn "cpp2rust-todo" <REPO>/.cpp2rust/<FEATURE>/rust/src/
```

---

## 三、符号验证说明

脚本的 § 6 阶段会执行多步符号验证，帮助确认 FFI 导出是否符合预期：

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
