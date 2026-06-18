# usage — 本地验证脚本与共享库

本目录提供 cpp2rust-demo 对 8 个真实 C/C++ 库的 **本地一键验证脚本**，与 CI 中的
`cargo test --test <lib>_e2e_test` 互补：CI 测试是 Rust 集成测试（依赖子模块），而
本目录的脚本是 **独立 Shell**，可在任意 Linux 终端直接执行，覆盖完整工作流：
环境检查 → 安装工具 → 子模块/系统头准备 → init → merge → cargo check → cargo test
→ FFI 审计 → 报告。

## 文件清单

| 脚本 | 目标库 | 库形态 | 子模块/系统依赖 |
|------|--------|--------|-----------------|
| [`verify-rapidjson-ffi.sh`](verify-rapidjson-ffi.sh) | rapidjson | extern-C shim 包装层 | vendored（`references/rapidjson-refactoring/`） |
| [`verify-tinyxml2-ffi.sh`](verify-tinyxml2-ffi.sh) | tinyxml2 | 单 cpp + 单 h，OOP 类层级 | `references/tinyxml2/` |
| [`verify-pugixml-ffi.sh`](verify-pugixml-ffi.sh) | pugixml | 单 cpp + 单 hpp，大量成员函数 | `references/pugixml/` |
| [`verify-nlohmann-json-ffi.sh`](verify-nlohmann-json-ffi.sh) | nlohmann/json | header-only，超大单头 + 重度模板 | `references/nlohmann-json/` |
| [`verify-fmtlib-ffi.sh`](verify-fmtlib-ffi.sh) | fmtlib/fmt | 多 .cc 文件 + extern template | `references/fmtlib/` |
| [`verify-magic-enum-ffi.sh`](verify-magic-enum-ffi.sh) | magic_enum | header-only，constexpr 模板元编程 | `references/magic_enum/` |
| [`verify-tomlplusplus-ffi.sh`](verify-tomlplusplus-ffi.sh) | toml++ | header-only，大型单头 | `references/tomlplusplus/` |
| [`verify-sqlite3-ffi.sh`](verify-sqlite3-ffi.sh) | sqlite3 | 系统 C 库，extern "C" API | `libsqlite3-dev` |
| [`verify-all-ffi.sh`](verify-all-ffi.sh) | 全部 8 个库的聚合入口 | — | — |
| [`lib/common.sh`](lib/common.sh) | 上述脚本共享的 shell 库 | — | — |

## 快速开始

### 一键跑全部 8 个库

```bash
# 系统依赖（Ubuntu/Debian，首次执行前安装一次）
sudo apt-get install -y clang libclang-dev g++ libstdc++-14-dev \
                        cmake binutils git curl libsqlite3-dev
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 跑全部（每个库独立日志写入 /tmp/cpp2rust-verify-<lib>.log）
bash usage/verify-all-ffi.sh
```

### 跑单个库

```bash
# 默认从 GitHub 安装 cpp2rust-demo；首次会编译几分钟
bash usage/verify-tinyxml2-ffi.sh

# 已 cargo install 过 cpp2rust-demo 或想用本地 target/release，置 SKIP_INSTALL=1
SKIP_INSTALL=1 bash usage/verify-tinyxml2-ffi.sh

# 自定义 feature 名（默认 <lib>_ffi）
FEATURE=my_tinyxml2 bash usage/verify-tinyxml2-ffi.sh
```

### 跑指定子集

```bash
# 聚合脚本接受库名参数
bash usage/verify-all-ffi.sh tinyxml2 fmtlib sqlite3
```

## 7 段式流程（所有 verify-<lib>-ffi.sh 同构）

```
§ 0-1.  环境检查（git/g++/cargo/nm + libclang）+ 安装 cpp2rust-demo
§ 2.    子模块初始化（git submodule）或系统头检查（sqlite3）
§ 3.    编译目标文件（供 §6 的 nm 符号交叉比对用）
§ 4.    cpp2rust-demo init —— LD_PRELOAD 拦截 + AST 解析 + FFI 脚手架生成
§ 5.    cpp2rust-demo merge —— 整理 src/ 目录结构
§ 5b-c. cargo check + cargo test —— 验证生成的 Rust 项目可编译链接
§ 6.    FFI 审计：import_lib!/import_class! 块存在性、link_name 一致性、
        struct/class 前缀清理、restrict 剥离、nm 符号交叉比对、降级标记统计
§ 7.    报告：捕获文件数、生成 Rust 文件数、FFI 函数绑定数、降级标记数
```

## 工作目录策略（重要）

cpp2rust-demo 通过 LD_PRELOAD hook 捕获 `.cpp2rust` 文件，路径相对 project_root
保存。若 project_root 是仓库根、源文件位于 `references/<lib>/foo.cpp` 时，unit_path
会变成 `references/<lib>/foo`，**含路径分隔符 `/`**，会污染 `link_name` 并破坏
hicc 命名空间拼接宏。

**解决方案**：所有脚本从 PROJECT_ROOT（即库自身根目录，如 `references/tinyxml2/`）
运行 init/merge，使捕获的相对路径只剩 basename（如 `tinyxml2.cpp.cpp2rust` →
unit_path = `tinyxml2`），link_name 自然干净。代价是 `.cpp2rust/` 输出落在
PROJECT_ROOT 内 —— 已通过根 `.gitignore` 的 `/references/**/.cpp2rust/` 规则
忽略，不污染子模块状态。

对 header-only 库（nlohmann-json / magic_enum / tomlplusplus），脚本会自动在
PROJECT_ROOT 生成一个临时 `_cpp2rust_<lib>_driver.cpp`（仅 `#include <header>`）
作为预处理入口，trap 清理。

## 输出目录结构

以 tinyxml2 为例：

```
references/tinyxml2/.cpp2rust/tinyxml2_ffi/
├── c/                   # 捕获的 .cpp2rust 预处理文件
├── meta/                # build_cmd.txt / build-meta.json / init-report.md /
│                        # merge-report.md / api-manifest.md / selected_files.json
└── rust/                # 生成的 Rust 项目
    ├── Cargo.toml       # 含 hicc + hicc-build + hicc-std 依赖
    ├── build.rs         # 自动注入 cc_build.include / .file / .std
    ├── src.1/           # init 原始输出（merge 时备份）
    ├── src/             # merge 后的最终输出
    │   ├── lib.rs       # pub mod ... 路由
    │   └── tinyxml2.rs  # hicc::cpp! + import_class! + import_lib! 三段式
    └── tests/
        └── smoke.rs     # 自动生成的冒烟测试（零参构造 + 标量 setter/getter 双值往返）
```

## 退出码与错误处理

| 退出码 | 含义 |
|--------|------|
| 0 | 该库的「init + merge + cargo check + cargo test」全流程通过 |
| 1 | 至少有一个非致命检查（通常是 cargo check/test）失败 |
| 2 | 仅 `verify-all-ffi.sh`：传入了未知的库名 |

策略：
- **致命错误**（init/merge 失败、子模块缺失等）→ 立即 `exit 1`
- **非致命错误**（cargo check/test 失败、FFI 审计告警等）→ 累计到末尾统一退出

cargo check / test 失败通常是 **cpp2rust-demo 对该库的已知限制**（如抽象类
`make_unique` 工厂、复杂模板方法签名、`hicc::MethodsType` 特化冲突等），而非脚本
bug。失败信息会清晰标注 `[WARN]` 并在末尾汇总。

## 各库当前通过情况（基线）

| 库 | init+merge | cargo check | cargo test | 备注 |
|----|------------|-------------|-----------|------|
| rapidjson | ✅ | ✅ | ✅ | extern-C shim 完全在 cpp2rust-demo 支持范围内 |
| tinyxml2 | ✅ | ✅ | ✅ | 抽象类 MemPool 自动跳过；XMLPrinter ↔ XMLDocument 环引用自动破环 |
| pugixml | ✅ | ✅ | ✅ | `.cpp` 内部类与 impl 命名空间自动过滤；字符串重载函数 `as_wide` 等脚本侧过滤 |
| nlohmann-json | ✅ | ✅ | ✅ | header-only 模板库，detail 命名空间自动过滤后剩余类少但可编译 |
| fmtlib | ✅ | ✅ | ✅ | 多 .cc 文件项目，build.rs 追加 `-Wl,--allow-multiple-definition` 容忍 format-inl.h 双重定义 |
| magic_enum | ✅ | ✅ | ✅ | header-only + constexpr，customize 等内部命名空间自动过滤 |
| tomlplusplus | ✅ | ✅ | ✅ | 大型 header-only，ref-qualifier 重载方法（begin/end/prune/flatten）脚本侧过滤 |
| sqlite3 | ✅ | ✅ | ✅ | 系统 C 库，libsqlite3.so 中无 sqlite3_win32_* 等符号自动过滤；build.rs 追加 `-lsqlite3` |

**全 8 库 100% 通过** — 任何 cargo check / cargo test 失败都会导致对应 verify-<lib>-ffi.sh
退出码非零，进而被聚合脚本 `verify-all-ffi.sh` 与 CI 工作流 `usage-verify-all.yml`
捕获。

## 环境变量

| 变量 | 默认值 | 作用 |
|------|--------|------|
| `SKIP_INSTALL` | `0` | `1` = 跳过 `cargo install cpp2rust-demo`（已安装时加速） |
| `OFFLINE` | `0` | `1` = 跳过子模块网络初始化（已在本地就绪时使用） |
| `FEATURE` | `<lib>_ffi` | 自定义 cpp2rust-demo feature 名 |
| `CXX_STD` | `c++17` | 覆盖 C++ 编译标准 |
| `SQLITE3_HEADER` | `/usr/include/sqlite3.h` | sqlite3 头文件路径（仅 sqlite3 脚本） |

## 故障排查

**Q: 出现「未找到命令 cpp2rust-demo」**

```bash
# 安装或加入 PATH
cargo install --git https://github.com/LuuuXXX/cpp2rust-demo --locked cpp2rust-demo
export PATH="$HOME/.cargo/bin:$PATH"
```

**Q: 子模块目录为空**

```bash
git submodule update --init references/<lib>
# 或一次性初始化全部 E2E 子模块
make submodules
```

**Q: cargo check 报 `redefinition of struct hicc::MethodsType<...>`**

这是 cpp2rust-demo 对 C++ 类的 `link_name` 处理已知短板，多见于带命名空间 + 重载
方法密集的库。脚本已通过「工作目录策略」规避了 link_name 含 `/` 的子问题；如仍有
该错误，属于 cpp2rust-demo 与 hicc 的 MethodsType 特化策略问题，欢迎提 issue。

**Q: nlohmann-json / magic_enum / tomlplusplus 没有原生 .cpp，怎么拦截？**

脚本会自动在 PROJECT_ROOT 生成一个临时 `_cpp2rust_<lib>_driver.cpp`，内容仅
`#include <header>` + 一个 `extern "C" int cpp2rust_driver_main()`，用于触发
预处理拦截。trap 在脚本退出时清理。

**Q: 脚本输出的 .cpp2rust/ 污染了子模块状态？**

不会。根 `.gitignore` 已包含 `/references/**/.cpp2rust/` 规则。如手动 cd 进入
子模块 `git status`，确实会看到该目录，但父仓库 `git status` 不会显示。

## 与 `tests/<lib>_e2e_test.rs` 的关系

| 维度 | `tests/<lib>_e2e_test.rs` | `usage/verify-<lib>-ffi.sh` |
|------|---------------------------|------------------------------|
| 形式 | Rust 集成测试（`cargo test`） | 独立 Shell 脚本 |
| 入口 | 绕过 CLI，直接调库 API | 走完整 CLI（`cpp2rust-demo init` + `merge`） |
| 编译验证 | 仅 `cargo check` Rust 侧 | 含 `cargo check` + `cargo test`（build.rs 编译 C++） |
| 子模块依赖 | 必须 `git submodule update --init` | 自动初始化（首次） |
| 输出位置 | TempDir（脚本结束清理） | `<project_root>/.cpp2rust/<feature>/`（持久化） |
| 适用场景 | CI 门禁、回归测试 | 本地探索、终端用户验证、CI 外复现 |

两者互补：Rust 集成测试是「**工具内部 API**」的回归门禁；Shell 脚本是「**用户
CLI 流程**」的端到端验证，更贴近真实使用方式。
