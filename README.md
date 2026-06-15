# cpp2rust-demo

[![CI](https://github.com/LuuuXXX/cpp2rust-demo/actions/workflows/ci.yml/badge.svg)](https://github.com/LuuuXXX/cpp2rust-demo/actions/workflows/ci.yml)

**C++ → Rust Safe FFI 自动化脚手架生成工具**。给定任意 C++ 项目，执行两条命令即可生成基于 [hicc](https://crates.io/crates/hicc) 的 Rust FFI 绑定层。

```bash
cpp2rust-demo init -- make -j4   # 捕获构建 + 生成 FFI 脚手架
cpp2rust-demo merge              # 备份并整理编译单元输出（可选）
```

> 工具只生成 FFI 脚手架（绑定声明 + 必要 C shim），不处理业务逻辑。深度技术方案见 [docs/INTRODUCTION.md](docs/INTRODUCTION.md)。

**主要特性**：跨平台编译拦截（LD_PRELOAD / DYLD / PATH 注入）、libclang AST 解析、hicc 三段式代码生成（`cpp!` / `import_class!` / `import_lib!`）、多 feature 支持、降级特性内联提示。

---

## 工作原理

LD_PRELOAD 拦截构建 → libclang 解析 `.cpp2rust` → FfiSpec IR → hicc 三段式 Rust 代码生成。

---

## 命令参考

| 子命令 | 作用 | 用法 |
|--------|------|------|
| `init` | 编译拦截 + AST 解析 + FFI 脚手架生成 | `cpp2rust-demo init -- make -j4` |
| `merge` | 整理输出 / 多 feature 合并 / 导出 | `cpp2rust-demo merge --feature default` |

### `init`

```bash
cpp2rust-demo init -- g++ -shared -fPIC mylib.cpp -o libmylib.so
cpp2rust-demo init -- make -j4
cpp2rust-demo init --feature linux_x86 -- make -j4
```

自动完成：hook 编译 → LD_PRELOAD 注入 → 文件选择 → AST 解析 → 代码生成 → 冒烟测试 `tests/smoke.rs`。

### `merge`

```bash
cpp2rust-demo merge                      # 单 feature 整理
cpp2rust-demo merge --feature a --feature b  # 多 feature 合并为统一 crate
cpp2rust-demo merge --output-dir /tmp/out    # 导出到指定目录
```

多 feature 合并后生成含 `[features]` 的 `Cargo.toml`，支持 `cargo build --features <name>` 按需编译。

### 环境变量

| 变量 | 说明 |
|------|------|
| `CPP2RUST_CXX` | 覆盖默认 C++ 编译器（支持带版本后缀如 `g++-13`） |
| `CPP2RUST_DEBUG` | 非空时输出 hook 调试日志到 stderr |
| `LIBCLANG_PATH` | 指定 libclang 搜索路径（macOS/Windows 必设） |

> v7 起无 `CPP2RUST_GEN_*` 开关：模板/proxy/dynamic_cast/smoke 全部默认生成。模板骨架以注释输出，proxy/dynamic_cast 为可编译活动绑定。

---

## 快速开始

### Linux（Ubuntu / Debian）

```bash
sudo apt-get install clang libclang-dev g++ libstdc++-14-dev
cargo install --git https://github.com/LuuuXXX/cpp2rust-demo
cd /path/to/my-cpp-project
cpp2rust-demo init -- make -j4
cpp2rust-demo merge
```

### macOS

```bash
brew install llvm
export LIBCLANG_PATH=$(brew --prefix llvm)/lib
export PATH="$(brew --prefix llvm)/bin:$PATH"
cargo install --git https://github.com/LuuuXXX/cpp2rust-demo
cpp2rust-demo init -- make -j4
```

> macOS SIP 对 `/usr/bin/g++` 等系统编译器无效。始终使用 Homebrew 编译器或将 `CPP2RUST_CXX` 设为 `$(brew --prefix llvm)/bin/clang++`。

### Windows

```powershell
# MinGW-w64（MSYS2）
pacman -S mingw-w64-x86_64-toolchain mingw-w64-x86_64-clang
cargo install --git https://github.com/LuuuXXX/cpp2rust-demo
cpp2rust-demo init -- make -j4

# MSVC：安装 VS Build Tools + LLVM，设置 LIBCLANG_PATH
set LIBCLANG_PATH=C:\Program Files\LLVM\bin
cargo install --git https://github.com/LuuuXXX/cpp2rust-demo
cpp2rust-demo init -- cmake --build build
```

### 最小示例

见 [examples/006_class_basic](examples/006_class_basic)：

```bash
cd examples/006_class_basic/cpp
g++ -shared -fPIC class_basic.cpp -o libclass_basic.so
cd ../rust_hicc && cargo run
```

---

## 生成代码格式

- **Direct 模式**：`import_class!`（`#[cpp(method)]` 直接绑方法）→ `import_lib!`（`make_unique<T>` 工厂 + 自由函数），无需 C shim。
- **Shim 模式**：`hicc::cpp!`（C shim）→ `import_class!`（类绑定）→ `import_lib!`（函数/全局绑定）。最小 shim 策略：只为 ctor/dtor/operator/static/placement new 生成 C 适配层。

---

## C++ 特性支持矩阵

> ✅ 完全自动　⚠️ 降级 + 内联 TODO　完整矩阵见 [docs/feature-matrix.md](docs/feature-matrix.md)

| 类别 | 编号 | 状态 |
|------|------|------|
| 基础函数 | 001–005 | ✅ / ⚠️ |
| 类与对象 | 006–012 | ✅ / ⚠️ |
| 面向对象 | 013–018 | ✅ |
| 运算符/类型 | 019–023 | ⚠️→✅ |
| 模板实例化 | 024–028 | ✅ / ⚠️ |
| 智能指针/内存 | 029–033 | ✅ |
| STL 容器 | 034–038 | ✅ |
| 函数对象 | 039–042 | ⚠️→✅ |
| 高级特性 | 043–048 | ✅ |
| Direct Binding | 049 | ✅ |

---

## 测试体系

| 层 | 验证内容 |
|----|---------|
| L1 | 黄金文件比对 |
| L2 | 编译测试 |
| L_smoke | 冒烟测试 |
| L4 | E2E 端到端 |

```bash
cargo test --lib
cargo test --test l1_golden_tests --features full-test -- --test-threads=1
```

---

## 绑定模式

cpp2rust-demo 自动判定绑定模式：**Direct Binding**（纯 C++ 类 → `make_unique<T>` + `#[cpp(method)]`，零前置工作）和 **Shim Binding**（项目含 `extern "C"` 类指针函数 → 传统 shim 包装）。详见 [docs/direct-vs-shim-binding.md](docs/direct-vs-shim-binding.md)。

---

## 依赖

Linux/macOS/Windows · g++/clang++ (C++11+) · Rust 1.82+ · libclang · hicc 0.2 / hicc-build 0.2

---

## 许可

MIT
