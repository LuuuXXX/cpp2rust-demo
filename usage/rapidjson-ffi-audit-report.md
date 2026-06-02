# cpp2rust-demo + rapidjson FFI 生成质量审计报告

> 生成时间：2026-06-02
> 输入：`verify-rapidjson-ffi.sh` 执行输出（`/tmp/rapidjson/.cpp2rust/rj_tests/rust/`）
> 工具版本：cpp2rust-demo (commit 697dd56c)

---

## 一、生成产物概览

| 指标 | 数值 |
|------|------|
| 捕获预处理文件数 | ~35 个 .cpp2rust |
| 生成 .rs 文件数 | ~42 个 |
| 单文件平均大小 | ~129 KB（最大 fwdtest.rs ~269 KB） |
| hicc 三段式结构 | `hicc::cpp!` / `hicc::import_class!` / `hicc::import_lib!` 均存在 |

目录结构：

```
/tmp/rapidjson/.cpp2rust/rj_tests/rust/
├── Cargo.toml
├── src -> src.2 (symlink)
├── src.2/
│   ├── lib.rs                          (3 行)
│   ├── gtest/googletest/               (gtest 框架绑定，非 rapidjson)
│   ├── perftest/                       (5 个性能测试文件)
│   └── unittest/                       (29 个单元测试文件)
```

---

## 二、可编译性评估：❌ 不可编译

生成代码**无法通过 `cargo check`**，存在以下编译阻断问题：

### 问题 1：`hicc::cpp!` 块被系统头文件内容严重污染

每个 .rs 文件中 `hicc::cpp!` 块前 600 行全是 glibc 系统类型定义：

```rust
hicc::cpp! {
    #include <sstream>
    #include "unittest.h"
    typedef long unsigned int size_t;          // glibc
    typedef __builtin_va_list __gnuc_va_list;  // 编译器内置
    typedef struct _IO_FILE FILE;              // stdio
    typedef unsigned char __u_char;            // sys/types.h
    // ... 重复 600+ 行系统 typedef
}
```

**根因**：`src/ast_parser.rs:403-409` 的 `collect_typedef()` 未做 `is_from_current_file` 过滤，
把预处理文件中所有 typedef（含系统头文件展开的）全部收集。

对比同文件中 `extract_class()` 有 `entity_is_from_current_file()` 过滤，typedef 收集遗漏了此步骤。

### 问题 2：`hicc::import_lib!` 中包含非法 Rust 类型语法

```rust
// 生成的代码中出现：
fn wcscpy(__dest: wchar_t *__restrict, __src: const wchar_t *__restrict) -> *mut wchar_t;
```

`wchar_t *__restrict`、`const wchar_t *__restrict` 不是合法 Rust 类型。
`__restrict` 是 C 限定符，`type_mapper` 未将其去除。

### 问题 3：缺少 `build.rs`

`Cargo.toml` 声明了 `hicc-build` 为 build-dependency，但项目未生成 `build.rs` 来调用它。

### 问题 4：`link_name` 值不正确

```rust
#![link_name = "unittest/documenttest"]  // 这是模块路径，不是链接库名
```

hicc-build 无法据此找到对应的 C++ 编译产物。

---

## 三、可读性评估：极差

| 维度 | 评分 | 说明 |
|------|------|------|
| 信号/噪声比 | ~5% | ~3000 行文件中仅 ~30 行与 rapidjson 相关 |
| 代码重复 | 极高 | 30 个 unittest 文件共享 95% 相同的系统 typedef |
| RapidJSON API 覆盖 | 0% | 未生成 Document / Value / Writer / Reader 等核心类绑定 |
| 绑定内容 | 错误 | 生成的是 `wcscpy` / `strlen` 等系统函数和 `cpu_set_t` 等系统类型 |

示例 — `documenttest.rs` 的 `import_class!` 和 `import_lib!` 内容：

```rust
hicc::import_class! {
    #[cpp(class = "cpu_set_t", destroy = "__sched_cpufree")]  // 系统类型，非 rapidjson
    class cpu_set_t {}
}

hicc::import_lib! {
    #![link_name = "unittest/documenttest"]
    class lconv;          // glibc locale 结构
    class timespec;       // 系统 time 结构
    class cpu_set_t;      // 系统 CPU 亲和性
    class stat;           // 系统 stat
    class re_pattern_buffer; // 系统 regex
    // ... 全是系统类型，无任何 rapidjson 类

    fn wcscpy(...) -> *mut wchar_t;  // 系统 wchar 函数
    fn strlen(...) -> usize;         // 系统 string 函数
    // ... 全是系统函数，无任何 rapidjson API
}
```

---

## 四、根因分析

### 4.1 处理目标错误（脚本层面）

`verify-rapidjson-ffi.sh` 配置了 `-DRAPIDJSON_BUILD_TESTS=ON`，
捕获的是 rapidjson 的 **gtest 测试文件**（`unittest/*.cpp`），
而非库本身的公开 API 头文件（`document.h`、`writer.h` 等）。

RapidJSON 是 **header-only** 库，没有独立的 .cpp 编译单元。
工具只能捕获参与编译的 .cpp 文件，所以只能捕获到测试文件。

### 4.2 typedef 过滤缺失（工具层面 — P0）

**位置**：`src/ast_parser.rs:403-409`

```rust
// 当前代码 — 无过滤
fn collect_typedef(entity: &clang::Entity<'_>, ast: &mut CppAst) {
    let Some(name) = entity.get_name() else { return };
    let Some(range) = entity.get_range() else { return };
    let start = range.get_start().get_file_location().offset;
    let end = range.get_end().get_file_location().offset;
    ast.typedefs.push((name, start, end));  // 无条件收集
}
```

**对比** `extract_class()` 和 `extract_function()` 都有 `entity_is_from_current_file()` 检查。

**影响**：每个生成文件多出 ~600 行无用系统 typedef，文件膨胀 20 倍。

### 4.3 type_mapper 未处理 C 限定符（工具层面 — P1）

`cpp_to_rust()` / `cpp_to_rust_ffi()` 不处理 `__restrict`、`__restrict__` 等 C 限定符，
导致生成的 Rust 类型语法非法。

### 4.4 枚举收集同样可能未过滤（工具层面 — P2）

`collect_linkage_spec()` 和 `collect_namespace()` 内的 `EnumDecl` 收集
只检查了 `is_in_system_header()`，未检查 `is_from_current_file`。

---

## 五、可优化点清单

### A. cpp2rust-demo 工具本身需优化

| # | 优先级 | 优化项 | 涉及文件 | 修复建议 |
|---|--------|--------|----------|----------|
| A1 | **P0** | typedef 收集添加 `is_from_current_file` 过滤 | `src/ast_parser.rs:403-409` | 传入 `cpp_ranges`，用 `entity_is_from_current_file()` 过滤 |
| A2 | **P0** | typedef 收集添加 `is_in_system_header()` 过滤 | `src/ast_parser.rs:224-225` | 顶层循环已有此过滤，但 namespace/linkage_spec 内的 `collect_typedef` 调用需独立检查 |
| A3 | **P1** | type_mapper 处理 `__restrict` / `__restrict__` 限定符 | `src/extractor/type_mapper.rs` → `clean_type()` | strip 掉 `__restrict` 等非 Rust 合法限定符 |
| A4 | **P1** | 生成 `build.rs` 文件 | `src/generator/project_generator.rs` | 需生成调用 `hicc_build::build()` 的 build.rs |
| A5 | **P1** | `link_name` 应使用库名而非模块路径 | `src/extractor/mod.rs` → `build_lib_spec()` | 需根据实际链接目标推断库名 |
| A6 | **P2** | enum 收集添加 `is_from_current_file` 过滤 | `src/ast_parser.rs:383-386` | 与 typedef 同理，避免引入头文件中的枚举 |
| A7 | **P2** | `hicc::cpp!` 块中多文件共享的系统 include 去重 | `src/extractor/mod.rs` → `build_cpp_block()` | 提取公共 include 到共享模块 |
| A8 | **P2** | 支持 header-only 库模式（直接解析头文件） | `src/main.rs` + `src/capture.rs` | 对 rapidjson 等 header-only 库，支持创建 wrapper .cpp 并捕获 |

### B. verify-rapidjson-ffi.sh 脚本需优化

| # | 优化项 | 说明 |
|---|--------|------|
| B1 | 添加 rapidjson 库 API 头文件的专门 wrapper | 创建 `#include "rapidjson/document.h"` 等 wrapper .cpp，让工具捕获此文件 |
| B2 | 排除 gtest 编译单元 | 添加过滤规则，不捕获 `gtest/` 目录 |
| B3 | 添加编译验证步骤 | 生成后执行 `cargo check`，失败时输出明确错误 |

---

## 六、建议修复优先级

| 阶段 | 修复项 | 预期效果 |
|------|--------|----------|
| **第一步** | A1 + A2（typedef 过滤） | 文件从 ~3000 行压缩到 ~100 行，消除 95% 噪声 |
| **第二步** | A3（`__restrict` 处理） | 生成的 Rust 类型语法合法 |
| **第三步** | B1（创建 rapidjson wrapper .cpp） | 让工具处理正确的目标（库 API 而非测试） |
| **第四步** | A4 + A5（build.rs + link_name） | 生成的项目可编译 |
| **后续迭代** | A6 / A7 / A8 / B2 / B3 | 完善边界情况处理 |

---

## 七、附录：关键代码位置索引

| 文件 | 行号 | 说明 |
|------|------|------|
| `src/ast_parser.rs` | 403-409 | `collect_typedef()` — 缺少过滤 |
| `src/ast_parser.rs` | 411-426 | `extract_class()` — 有 `is_from_current_file` 过滤（可参考） |
| `src/ast_parser.rs` | 828-839 | `entity_is_from_current_file()` — 过滤辅助函数 |
| `src/ast_parser.rs` | 848-893 | `cpp_byte_ranges()` — 行号标记扫描逻辑 |
| `src/extractor/mod.rs` | 196-315 | `build_cpp_block()` — hicc::cpp! 块生成 |
| `src/extractor/type_mapper.rs` | — | `cpp_to_rust()` / `clean_type()` — 类型映射 |
| `src/generator/project_generator.rs` | — | 项目结构生成（缺 build.rs） |
