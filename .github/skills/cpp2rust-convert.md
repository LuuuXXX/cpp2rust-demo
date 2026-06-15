# Skill: cpp2rust-convert

**触发条件**：用户在 C++ 项目根目录请求"转换为 Rust FFI"，或要求"用 cpp2rust-demo 分析/转换这个 C++ 项目"。

**前提**：已安装 `cpp2rust-demo`（`cargo install --git https://github.com/LuuuXXX/cpp2rust-demo`，该工具目前从 GitHub 源码安装）及系统依赖（`clang`、`libclang-dev`、`g++`）。

---

## 执行步骤

### 步骤 0：确认工作目录

确保当前目录是 C++ 项目根目录（存在 `.cpp` 文件、`Makefile`、`CMakeLists.txt` 等构建标志）。
若不确定，先询问用户或使用 `ls` / `find` 探查。

### 步骤 1：询问 feature 名称

向用户提问：

> 请输入 feature 名称（直接回车跳过则使用默认值 `default`）：

- 若用户提供了名称，使用该名称；
- 若用户跳过（直接回车或未填写），使用 `default`。

```bash
FEATURE=<用户输入，或 default>
```

### 步骤 2：询问构建命令

向用户提问：

> 请输入构建命令（例如 `make -j$(nproc)`、`cmake --build build -- -j$(nproc)`、`bash build.sh` 等）：

- 用户**必须**提供构建命令，不可为空。
- 若用户提供的是 CMake 项目，提示其先在外部执行配置步骤，再将构建命令（如 `cmake --build build -- -j$(nproc)`）填入此处：
  ```bash
  cmake -B build -DCMAKE_BUILD_TYPE=Debug
  # 然后将下方命令作为构建命令输入：
  cmake --build build -- -j$(nproc)
  ```

```bash
BUILD_CMD=<用户输入的构建命令>
```

### 步骤 3：执行捕获与代码生成

```bash
cpp2rust-demo init --feature "$FEATURE" -- $BUILD_CMD
```

`init` 自动完成：
1. 将 binary 内嵌的 `hook.cpp` 解压到用户数据目录并编译为 `libhook.so`（首次运行；后续 hook 库为最新版时自动跳过）
2. 通过 `LD_PRELOAD` 拦截构建过程，捕获 `.cpp2rust` 预处理文件
3. 交互式（或自动全选）选择参与转换的文件
4. libclang 解析 AST，生成 `.cpp2rust/$FEATURE/rust/` 下的 hicc Rust 脚手架

### 步骤 4：执行合并整理（可选但推荐）

```bash
cpp2rust-demo merge --feature "$FEATURE"
```

生成后的目录结构将与 C++ 项目目录结构一一对应。

### 步骤 5：汇报结果

执行完成后，向用户汇报：

1. 生成文件数量和路径（`.cpp2rust/$FEATURE/rust/src/`）
2. 若有 `cpp2rust-todo` 标记，列出各 TAG 及出现次数，说明需要手动处理的位置：
   - `[OP]`：运算符重载 — 需手动实现 Rust `std::ops::*` trait
   - `[VA]`：可变参数模板 — 需按需扩充 wrapper 方法
   - `[LM]`：有状态 Lambda — 需手动编写 trampoline

---

## 完整示例（以 rapidjson 为例）

以下为 Agent 与用户交互的完整流程示意：

```
Agent: 请输入 feature 名称（直接回车跳过则使用默认值 `default`）：
User:  （回车跳过）
→ FEATURE=default

Agent: 请输入构建命令（例如 `make -j$(nproc)`、`cmake --build build -- -j$(nproc)` 等）：
User:  cmake --build build -- -j$(nproc)
→ BUILD_CMD=cmake --build build -- -j$(nproc)
```

```bash
# 克隆 rapidjson
git clone https://github.com/Tencent/rapidjson /tmp/rapidjson
cd /tmp/rapidjson

# CMake 项目：先执行配置步骤（在 Agent 询问构建命令前手动完成）
cmake -B build -DCMAKE_BUILD_TYPE=Debug

# Agent 收集用户输入后执行捕获构建
cpp2rust-demo init --feature "$FEATURE" -- $BUILD_CMD

# 整理目录结构
cpp2rust-demo merge --feature "$FEATURE"

# 查看生成结果
find .cpp2rust/default/rust/src -name "*.rs" | head -20
grep -r "cpp2rust-todo" .cpp2rust/default/rust/src/ | grep -o '\[.*\]' | sort | uniq -c
```

---

## 环境变量参考

| 变量 | 说明 |
|------|------|
| `CPP2RUST_CC` | 覆盖默认 C++ 编译器（默认自动检测 g++/clang++/c++） |
| `CPP2RUST_LD` | 覆盖默认链接器（默认自动检测 ld/lld 等） |
| `CPP2RUST_DEBUG` | 非空时输出 hook 调试日志到 stderr |

---

## 生成代码结构说明

生成的 Rust 代码采用 **hicc 三段式**格式：

```rust
// 段 1：C++ 实现内联（含必要 shim）
hicc::cpp! { /* ... */ }

// 段 2：类方法绑定
hicc::import_class! { /* ... */ }

// 段 3：全局/关联函数绑定
hicc::import_lib! { /* ... */ }
```

详细说明参见仓库 README 与 `docs/INTRODUCTION.md`。
