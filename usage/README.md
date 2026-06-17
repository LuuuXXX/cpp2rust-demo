# usage — 本地验证脚本与 SKILL 使用文档

本目录包含对 rapidjson 项目进行 cpp2rust-demo FFI 转换的**所有可用方式**：

| 文件 | 说明 |
|------|------|
| [`verify-rapidjson-ffi.sh`](verify-rapidjson-ffi.sh) | 全自动 Shell 脚本（CLI 方式，适合批量/CI 场景） |
| 本文档（README.md） | 脚本用法 + SKILL 交互式工作流完整说明 |

SKILL 文件本身位于 [`.github/skills/cpp2rust-convert.md`](../.github/skills/cpp2rust-convert.md)，由 GitHub Copilot Agent 自动读取，无需手动调用。

---

## 一、快速开始（选择其中一种方式）

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

## 三、其他项目验证脚本（8 个真实项目全覆盖）

本目录包含 8 个真实 C++ 项目的完整 FFI 验证脚本，覆盖从简单到复杂的各种场景：

| 脚本 | 项目 | 特点 | 验证重点 |
|------|------|------|---------|
| `verify-rapidjson-ffi.sh` | rapidjson | 纯 C++ + shim 层 | extern-C shim 工作流 |
| `verify-tinyxml2-ffi.sh` | tinyxml2 | 单头 + 单 .cpp | 最简 OOP 类继承 |
| `verify-pugixml-ffi.sh` | pugixml | header-only XML | header-only 模式 |
| `verify-sqlite3-ffi.sh` | sqlite3 | 超大单文件 (~23 万行) | 超大文件解析 |
| `verify-nlohmann-json-ffi.sh` | nlohmann/json | header-only 重度模板 | 模板类提取 + 合并 |
| `verify-fmtlib-ffi.sh` | fmtlib | 现代 C++ 多文件 + CMake | CMake 构建集成 |
| `verify-magic-enum-ffi.sh` | magic_enum | constexpr + 模板元编程 | enum class + constexpr |
| `verify-tomlplusplus-ffi.sh` | toml++ | 大型单头模板库 | 大型 header-only 模板 |

### 快速开始

**运行单个项目验证**：
```bash
# 示例 1：验证 tinyxml2
bash usage/verify-tinyxml2-ffi.sh

# 示例 2：验证 nlohmann/json
bash usage/verify-nlohmann-json-ffi.sh

# 示例 3：跳过 cargo install（已安装时加速）
SKIP_INSTALL=1 bash usage/verify-pugixml-ffi.sh
```

**一键运行全部验证**：
```bash
bash usage/verify-all.sh
```

### 环境依赖

所有脚本共享以下系统依赖（Ubuntu/Debian）：

```bash
sudo apt-get install -y clang libclang-dev g++ libstdc++-14-dev \
                        binutils git curl
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# fmtlib 脚本额外需要 CMake
sudo apt-get install -y cmake
```

### 验证脚本结构说明

每个脚本遵循统一的 7 段式结构（参考 rapidjson 脚本）：

1. **§ 0. 环境检查**：检测 git / g++ / cargo / nm / libclang
2. **§ 1. 安装工具**：`cargo install cpp2rust-demo`（可通过 `SKIP_INSTALL=1` 跳过）
3. **§ 2. 定位源文件**：定位或克隆目标项目（使用 `references/` 子模块）
4. **§ 3. 编译目标文件**：编译 .o 供后续 nm 符号验证
5. **§ 4-5. init + merge**：捕获构建 → 生成 FFI → 整理输出
6. **§ 5a-5c. 验证生成项目**：校验 build.rs → cargo check → cargo test
7. **§ 6. FFI 验证**：nm 符号 / import_class! / import_lib! / link_name / 预处理文件完整性
8. **§ 7. 结果汇报**：捕获文件数 / 生成文件数 / 降级标记统计

### 常见问题

**Q: 提示"子模块未初始化"**
```bash
# 初始化全部子模块
git submodule update --init --recursive

# 或只初始化特定项目
git submodule update --init references/tinyxml2
```

**Q: 如何自定义 feature 名称**
```bash
FEATURE=my_custom_name bash usage/verify-tinyxml2-ffi.sh
```

**Q: 脚本适用场景**
- ✅ **本地开发验证**：在提交 PR 前验证工具对真实项目的处理能力
- ✅ **回归测试**：在修改工具核心逻辑后快速验证是否引入 bug
- ✅ **CI/CD 集成**：作为 GitHub Actions 的验证步骤（见各项目独立 workflow）
- ✅ **学习样板**：为自己的 C++ 项目编写类似验证脚本的参考

---

## 四、符号验证说明

脚本的 § 6 阶段会执行四步符号验证，帮助确认 FFI 导出是否符合预期：

### 四.1 — 编译产物符号

使用 `nm --demangle` 查看 rapidjson 编译产物（`.o`/`.so`/`.a`）中的 C++ mangled 符号，
确认测试相关类型（如 `BigInteger`、`Document`、`Reader`）已被编译。

### 四.2 — 生成 Rust 代码 FFI 声明

检查 cpp2rust-demo 生成的 `.rs` 文件中的三段式声明：

| 声明块 | 内容 |
|--------|------|
| `hicc::cpp! { ... }` | C++ shim 实现（ctor/dtor/operator 等必要 shim） |
| `hicc::import_class! { ... }` | 类方法绑定（hicc 处理虚表 dispatch） |
| `hicc::import_lib! { ... }` | 全局/关联函数绑定 |

### 四.3 — shim 函数名交叉比对

从生成代码中提取 `#[cpp(func = "...")]` 标注的 shim 函数名，与 `nm` 符号表交叉比对：

- ✓ **在目标文件中找到**：该 C shim 函数已在编译产物中存在
- ? **未在目标文件中直接找到**：该 shim 函数由 `hicc::cpp!` 宏在 Rust 构建时展开，正常现象

### 四.4 — 预处理文件完整性

统计各 `.cpp2rust` 文件行数，确认预处理捕获内容完整（行数越多说明捕获越充分）。

---

## 五、SKILL 工作流完整说明

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

## 六、常见问题

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
