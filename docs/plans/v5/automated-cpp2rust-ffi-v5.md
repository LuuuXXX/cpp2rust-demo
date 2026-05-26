# C++ 到 Rust Safe FFI 自动化工具 - 方案 v5

## 1. 概述

### 1.1 核心思路

v5 通过 **LD_PRELOAD 编译拦截**机制，在真实编译过程中捕获 C++ 预处理代码，再由 libclang 解析 C++ AST，最终生成 hicc 宏格式的 Rust FFI 脚手架。

三条技术主线：
1. **拦截**：`LD_PRELOAD` + `__attribute__((constructor))` 注入构建过程，每个 `.cpp` 文件预处理后保存为 `.c2rust`，编译标志保存为 `.c2rust.opts`
2. **解析**：`clang` crate（直接 libclang 绑定）读取 `.c2rust` + `.c2rust.opts`，提取 C++ 类/方法/函数/枚举/模板实例化
3. **生成**：以编译单元为粒度，输出 `hicc::cpp! + import_class! + import_lib!` 三段式 Rust 代码

**关键理念**：C++ 模板的价值在于**实例化结果**，而非模板本身。转换时只关注实际被实例化的具体类型（如 `std::vector<int>`），不处理模板声明本身。

### 1.2 与 c2rust-demo（C 版本）的关系

v5 在架构上完全对齐 `references/c2rust-demo`，差异仅在于：C 版本面向 C 代码 + bindgen + `extern "C"`；v5 面向 C++ 代码 + libclang + hicc 宏。

| 模块 | c2rust-demo 来源 | v5 处理方式 |
|------|-----------------|-----------|
| `hook/hook.c` → `hook/hook.cpp` | `references/c2rust-demo/hook/hook.c` | **修改**：改编译器名 + 文件扩展名，其余逻辑不变 |
| `hook/Makefile` | `references/c2rust-demo/hook/Makefile` | **修改**：`hook.c` → `hook.cpp`，编译器改用 g++ |
| `src/capture.rs` | `references/c2rust-demo/src/capture.rs` | **直接复用**：构建 libhook.so、带 LD_PRELOAD 执行构建命令 |
| `src/layout.rs` | `references/c2rust-demo/src/layout.rs` | **直接复用**：`.c2rust/<feature>/` 目录结构管理 |
| `src/selector.rs` | `references/c2rust-demo/src/selector.rs` | **直接复用**：交互式文件选择（dialoguer） |
| `src/error.rs` | `references/c2rust-demo/src/error.rs` | **直接复用**：错误类型定义 |
| `src/main.rs` | `references/c2rust-demo/src/main.rs` | **微调**：CLI 名称改为 `cpp2rust-ffi`，帮助文本适配 |
| `src/ast_parser.rs` | —— | **从零实现**：clang crate 解析 C++ AST |
| `src/extractor/` | —— | **从零实现**：从 AST 提取类/方法/函数/枚举/模板信息 |
| `src/generator/` | —— | **从零实现**：生成 hicc 三段式 Rust 代码 |

### 1.3 版本定位

v5 是完全独立的新版本，**所有输入均通过 LD_PRELOAD 编译拦截**方式获取，不依赖任何手工提供的头文件或 AST 文件。

---

## 2. 快速开始

### 2.1 工作流程（与 c2rust-demo 完全一致的 CLI 设计）

```bash
# 在 C++ 项目根目录执行：

# Step 1: 拦截构建 + 生成 Rust 脚手架
cpp2rust-ffi init -- make -j4
# 或指定 feature 名称
cpp2rust-ffi init --feature mylib -- cmake --build . --target mylib

# Step 2: 将按符号文件合并为按模块文件（可选）
cpp2rust-ffi merge
cpp2rust-ffi merge --feature mylib
```

`init` 命令内部流程（完整复用 c2rust-demo 的 `run_init`）：
1. 编译 `hook/libhook.so`（`capture::build_hook()`）
2. 以 `LD_PRELOAD + C2RUST_*` 环境变量运行用户构建命令（`capture::run_with_hook()`）
3. 扫描 `.c2rust/<feature>/c/` 下的 `.c2rust` 文件（`layout::scan_c2rust_files()`）
4. 交互式选择要处理的文件（`selector::InteractiveSelector`，CI 下自动全选）
5. **[C++ 专有]** 用 `clang` crate + `.c2rust.opts` 解析每个 `.c2rust` 文件
6. **[C++ 专有]** 生成 hicc 三段式 Rust 代码（每编译单元一个 `mod_xxx.rs`）

### 2.2 环境变量

| 变量 | 必填 | 来源 | 说明 |
|------|------|------|------|
| `C2RUST_PROJECT_ROOT` | 自动设置 | capture.rs | C++ 项目根目录（含 `.c2rust/`），由工具自动探测并设置 |
| `C2RUST_FEATURE_ROOT` | 自动设置 | capture.rs | 捕获产物输出目录 |
| `C2RUST_CC` | ❌ | hook.cpp | 覆盖编译器自动检测（默认识别 g++/clang++/c++ 及版本后缀） |
| `C2RUST_LD` | ❌ | hook.cpp | 覆盖链接器自动检测（默认识别 ld/lld） |
| `C2RUST_DEBUG` | ❌ | hook.cpp | 设为非空时输出 hook 调试日志到 stderr |

---

## 3. 目录结构

### 3.1 捕获产物（`init` 后）

```
<cpp-project>/
└── .c2rust/<feature>/
    ├── c/                              # hook 捕获的预处理文件
    │   └── src/
    │       ├── foo.cpp.c2rust          # g++ -E -C -P 预处理后的 C++ 代码
    │       ├── foo.cpp.c2rust.opts     # 编译标志（-I/-D/-U/-std=等），供 clang crate 使用
    │       ├── bar.cpp.c2rust
    │       └── bar.cpp.c2rust.opts
    │   └── targets.list                # 链接目标（动态库/可执行文件名）
    ├── meta/
    │   ├── build_cmd.txt               # 原始构建命令
    │   ├── selected_files.json         # 用户选中的 .c2rust 文件列表
    │   └── init-interface-report.md    # 类/函数/枚举接口报告
    └── rust/                           # 生成的 Rust 项目
        ├── Cargo.toml
        └── src/
            ├── lib.rs                  # 库入口（mod 声明）
            └── mod_src_foo/            # 每个 .cpp 文件一个模块目录
                ├── mod.rs              # hicc 三段式绑定代码
                └── mod.normalized      # 格式化前备份
```

### 3.2 合并产物（`merge` 后，追加）

```
.c2rust/<feature>/
├── meta/
│   └── merge-interface-report.md
└── rust/
    ├── src.1/           # init 原始输出备份
    ├── src -> src.2     # 符号链接
    └── src.2/           # 合并后：每个模块一个 .rs 文件
        ├── lib.rs
        └── mod_src_foo.rs
```

---

## 4. 技术架构

### 4.1 模块结构

```
cpp2rust-ffi/
├── hook/
│   ├── hook.cpp        # 【修改自 c2rust-demo/hook/hook.c】C++ 编译拦截器
│   └── Makefile        # 【修改自 c2rust-demo/hook/Makefile】
└── src/
    ├── main.rs          # 【微调自 c2rust-demo】CLI 入口（init / merge）
    ├── capture.rs       # 【直接复用 c2rust-demo】hook 构建与 LD_PRELOAD 执行
    ├── layout.rs        # 【直接复用 c2rust-demo】.c2rust/<feature>/ 目录管理
    ├── selector.rs      # 【直接复用 c2rust-demo】交互式文件选择（dialoguer）
    ├── error.rs         # 【直接复用 c2rust-demo】错误类型
    ├── ast_parser.rs    # 【从零实现】clang crate 解析 C++ AST
    ├── extractor/       # 【从零实现】从 AST 提取 C++ 信息
    │   ├── mod.rs
    │   ├── class.rs     # 类/结构体 + 成员函数提取
    │   ├── function.rs  # 全局函数提取
    │   └── enum_.rs     # 枚举提取
    └── generator/       # 【从零实现】生成 hicc 格式 Rust 代码
        ├── mod.rs
        ├── cpp_block.rs     # hicc::cpp! 块生成
        ├── class_block.rs   # hicc::import_class! 块生成
        └── lib_block.rs     # hicc::import_lib! 块生成
```

### 4.2 三阶段处理流程

```
┌──────────────────────────────────────────────────────────────────┐
│ Phase 1: 编译拦截 (hook.cpp)                                     │
│   LD_PRELOAD 注入 → 识别 g++/clang++/c++ → 发现 .cpp 文件        │
│   → g++ -E -C -P 预处理 → 保存 .c2rust + .c2rust.opts            │
│   → 识别链接器 → 保存 targets.list                                │
└──────────────────────────────────────────────────────────────────┘
                               ↓
┌──────────────────────────────────────────────────────────────────┐
│ Phase 2: AST 解析 (ast_parser.rs + extractor/)                  │
│   读取 .c2rust.opts → 解析编译标志(-I/-D/-std=等)                 │
│   → clang::Index::parser().arguments(opts).parse()              │
│   → 遍历 AST → 提取 ClassDecl/Method/FunctionDecl/EnumDecl       │
│   → 识别模板实例化（entity.get_template().is_some()）             │
└──────────────────────────────────────────────────────────────────┘
                               ↓
┌──────────────────────────────────────────────────────────────────┐
│ Phase 3: 代码生成 (generator/)                                   │
│   每个 .c2rust 文件 → 对应 mod_xxx/mod.rs                        │
│   三段式：hicc::cpp! + hicc::import_class! + hicc::import_lib!   │
│   → lib.rs（mod 声明） + 各 mod.rs                               │
└──────────────────────────────────────────────────────────────────┘
```

---

## 5. Hook 机制

### 5.1 hook.cpp 与 hook.c 的差异（最小化修改原则）

`hook.c` 的核心逻辑（LD_PRELOAD 注入、`/proc/self/cmdline` 读取、`fork+execvp` 预处理、路径处理、目录创建、`.opts` 文件保存、`targets.list` 维护）**全部保留不变**。仅修改以下两处：

| 修改点 | hook.c 原值 | hook.cpp 新值 |
|--------|------------|--------------|
| 编译器名列表 `cc_names[]` | `{"gcc", "clang", "cc"}` | `{"g++", "clang++", "c++"}` |
| 文件扩展名判断 `is_cfile()` | `strcmp(&file[len-2], ".c") == 0` | 匹配 `.cpp`, `.cc`, `.cxx`, `.c++`, `.C`, `.cp` |

**保留不变的重要特性**：
- `is_name_match()`：支持版本后缀（`g++-13`、`clang++-15` 等），无需修改
- `save_options()`：保存 `-I/-D/-U/-include/-isystem/-iquote/-std=/-fshort-enums` 到 `.c2rust.opts`——**这是 clang crate 解析 C++ 代码的关键输入**
- `discover_target()`：收集 `targets.list`，供后续链接使用，无需修改
- `preprocess_cfile()`：`fork + execvp g++ -E -C -P`，路径处理逻辑不变

### 5.2 Makefile 适配

```makefile
# 将 hook.c 改为 hook.cpp，编译器改用 g++：
libhook.so: hook.cpp
    g++ -shared -fPIC -O2 -o $@ $< -ldl
```

### 5.3 预处理输出说明

```
.c2rust/<feature>/c/src/foo.cpp.c2rust       # g++ -E -C -P 宏展开后的 C++ 代码
.c2rust/<feature>/c/src/foo.cpp.c2rust.opts  # 原始编译标志（含 -I/-D/-std= 等）
```

`-P` 选项去除行号信息（`# line N "file"` 标记），避免展开文件中的路径指向系统头文件，简化 clang crate 后续解析。

---

## 6. AST 解析

### 6.1 opts 文件的利用

`.c2rust.opts` 文件（由 `save_options()` 写入）格式为空格分隔的带引号参数，如：
```
"-I/usr/include" "-I./include" "-DNDEBUG" "-std=c++17"
```

解析时需：
1. 读取并解析 `.c2rust.opts` 文件，提取标志列表
2. 将这些标志作为 `arguments()` 传入 `clang::Index::parser()`
3. 额外传入 `-x c++`（强制以 C++ 模式解析 `.c2rust` 文件）和 `-fno-builtin`

```rust
// ast_parser.rs
use clang::{Clang, EntityKind, Index};

pub fn parse_preprocessed(c2rust_file: &Path, opts_file: &Path) -> Result<CppAst> {
    let clang = Clang::new()?;
    let index = Index::new(&clang, false, false);

    // 读取并解析 .c2rust.opts
    let flags = parse_opts_file(opts_file)?;  // Vec<String>

    let tu = index
        .parser(c2rust_file)
        .arguments(&flags)
        .detailed_preprocessing(false)
        .skip_function_bodies(true)   // 只解析声明，不解析函数体，加快速度
        .parse()?;

    // 遍历顶层 AST 节点
    for entity in tu.get_entity().get_children() {
        // 只处理来自本文件的节点（过滤掉系统头文件展开内容）
        if !entity.get_location().map_or(false, |l| l.is_in_main_file()) {
            continue;
        }
        extract_entity(&entity, &mut ast)?;
    }
    Ok(ast)
}
```

### 6.2 支持的 C++ AST 节点

| C++ 概念 | `clang` crate `EntityKind` | v5 用途 |
|----------|--------------------------|--------|
| 类/结构体 | `ClassDecl` / `StructDecl` | 生成 `import_class!` 块 |
| 成员函数 | `Method` | 生成 `import_class!` 中的方法绑定 |
| 构造函数 | `Constructor` | 生成工厂函数（`cpp!` 块内 C++ wrapper） |
| 析构函数 | `Destructor` | 生成删除函数（`cpp!` 块内 C++ wrapper） |
| 全局函数 | `FunctionDecl` | 生成 `import_lib!` 中的函数绑定 |
| 枚举 | `EnumDecl` | 生成 Rust `enum` 或常量 |
| 命名空间 | `Namespace` | 用于生成 Rust 模块层次 |
| 模板全实例化 | `ClassDecl` + `entity.get_template().is_some()` | 作为普通类处理（如 `vector<int>`） |
| 模板偏特化 | `ClassTemplatePartialSpecialization` | 按普通类处理 |
| 模板声明本身 | `ClassTemplate` / `FunctionTemplate` | **跳过**（不处理模板源码） |

> **过滤原则**：只处理 `entity.get_location().is_in_main_file() == true` 的节点，排除从系统头文件展开进来的 STL/标准库定义（避免解析量爆炸）。

### 6.3 模板实例化识别

预处理后的 `.c2rust` 文件中，`std::vector<int>` 等模板实例化会被展开为具体的 `ClassDecl`。识别方式：
```rust
if entity.get_kind() == EntityKind::ClassDecl
    && entity.get_template().is_some()
{
    // 这是一个模板实例化结果（如 vector<int>）
    // entity.get_display_name() → "vector<int>"
    // entity.get_template().get_display_name() → "vector"
}
```

---

## 7. 代码生成

### 7.1 每编译单元对应一个模块

对齐 c2rust-demo 的 `mod_xxx/` 设计，每个 `.cpp` 文件对应一个 Rust 模块：

```
foo.cpp → .c2rust/<feature>/c/src/foo.cpp.c2rust
       → .c2rust/<feature>/rust/src/mod_src_foo/mod.rs
```

模块名生成规则（与 c2rust-demo 一致）：将文件路径中的非字母数字字符替换为下划线，前缀加 `mod_`。

### 7.2 三段式输出格式

```rust
// ===== 段 1: C++ 实现 wrapper（hicc::cpp! 块）=====
// 为不能直接 FFI 的操作（构造/析构/运算符）提供 C-ABI 桥接
hicc::cpp! {
    #include "foo.h"

    // 构造函数 wrapper（因为 C++ ctor 无法直接 import_lib!）
    Foo* foo_new(int value) { return new Foo(value); }
    void foo_delete(Foo* self) { delete self; }
}

// ===== 段 2: 类方法绑定（hicc::import_class! 块）=====
hicc::import_class! {
    #[cpp(class = "Foo")]
    class Foo {
        #[cpp(method = "int getValue() const")]
        fn getValue(&self) -> i32;

        #[cpp(method = "void setValue(int)")]
        fn setValue(&mut self, value: i32);
    }
}

// ===== 段 3: 全局函数 + 构造函数 wrapper 绑定（hicc::import_lib! 块）=====
hicc::import_lib! {
    #![link_name = "foo"]

    class Foo;

    #[cpp(func = "Foo* foo_new(int value)")]
    fn foo_new(value: i32) -> *mut Foo;

    #[cpp(func = "void foo_delete(Foo* self)")]
    unsafe fn foo_delete(self_: *mut Foo);
}
```

### 7.3 降级处理（TODO 注释标记）

与 v4 一致，以下特性降级处理并插入 TODO 注释：

| 特性 | 处理方式 | TODO tag |
|------|---------|----------|
| 运算符重载（例 019） | 生成 named shim 函数（`foo_add`, `foo_eq` 等），提示可手动实现 `std::ops` trait | `// cpp2rust-todo[OP]` |
| 友元函数（例 020） | 直接放入 `import_lib!`，与普通全局函数相同 | `// cpp2rust-todo[FR]` |
| typeid/RTTI（例 023） | 注入 Rust 枚举模拟类型标识 | `// cpp2rust-todo[RTTI]` |
| Lambda（例 039） | 跳过 lambda 本体，提示手动转为 Rust 闭包 | `// cpp2rust-todo[LM]` |
| 可变参数（例 005, 028） | 跳过 variadic 函数 | `// cpp2rust-todo[VA]` |

---

## 8. C++ 特性支持

### 8.1 总览（对应 `examples/` 中的 48 个示例）

| 类别 | 示例编号 | 数量 | ✅ 自动生成 | ⚠️ 降级处理 |
|------|---------|------|-----------|-----------|
| 基础类型与函数 | 001-005 | 5 | 4 | 1（005 variadic） |
| 类与对象 | 006-012 | 7 | 7 | 0 |
| 面向对象特性 | 013-018 | 6 | 6 | 0 |
| 运算符与特殊函数 | 019-023 | 5 | 2 | 3（019 OP / 020 FR / 023 RTTI） |
| 模板实例化 | 024-028 | 5 | 4 | 1（028 variadic template） |
| 智能指针与内存 | 029-033 | 5 | 5 | 0 |
| STL 容器 | 034-038 | 5 | 5 | 0 |
| 函数对象 | 039-042 | 4 | 3 | 1（039 lambda） |
| 其他高级特性 | 043-048 | 6 | 6 | 0 |
| **总计** | | **48** | **42** | **6** |

### 8.2 ⚠️ 降级处理详情

| 示例 | C++ 特性 | 降级方案 | TODO Tag |
|------|---------|---------|----------|
| 005 | 可变参数函数 | 跳过，不生成 FFI | `[VA]` |
| 019 | 运算符重载 | Named shim 函数（`operator_add` 等） | `[OP]` |
| 020 | 友元函数 | 直接入 `import_lib!` | `[FR]` |
| 023 | typeid/RTTI | 注入枚举模拟 | `[RTTI]` |
| 028 | 可变参数模板 | 跳过实例化，不生成 FFI | `[VA]` |
| 039 | Lambda | 跳过，提示手动转 Rust 闭包 | `[LM]` |

---

## 9. 实现计划

### 9.1 Phase 顺序

| 阶段 | 内容 | 策略 | 优先级 | 依赖 |
|------|------|------|--------|------|
| Phase 0 | Hook 机制 + 脚手架 | 修改 hook.c → hook.cpp；复制/微调其余模块 | P0 | — |
| Phase 1 | opts 文件解析 + clang 环境搭建 | 新建，验证 clang crate 能正确解析 .c2rust 文件 | P0 | Phase 0 |
| Phase 2 | AST 提取器（class/function/enum） | 新建，处理非模板 C++ 节点 | P0 | Phase 1 |
| Phase 3 | 模板实例化识别 | 新建，追踪 get_template() | P0 | Phase 1 |
| Phase 4 | hicc 代码生成器 | 新建，输出三段式 Rust 代码 | P0 | Phase 2, 3 |
| Phase 5 | 降级处理器（OP/FR/RTTI/LM/VA） | 新建，插入 TODO 注释 | P1 | Phase 2 |
| Phase 6 | merge 命令适配 | 微调 c2rust-demo/src/split/merge.rs（模块合并逻辑） | P1 | Phase 4 |
| Phase 7 | 集成测试（以 examples/ 为输入） | 新建，验证 48 个示例的生成结果 | P1 | Phase 4-6 |

### 9.2 各 Phase 详细任务

**Phase 0 - 基础设施（修改 + 复制）**：
1. [ ] 创建 `hook/hook.cpp`（基于 `references/c2rust-demo/hook/hook.c`）：
   - 修改 `cc_names[]`：`{"gcc", "clang", "cc"}` → `{"g++", "clang++", "c++"}`
   - 修改 `is_cfile()`：添加对 `.cpp`/`.cc`/`.cxx`/`.c++`/`.C`/`.cp` 的匹配
   - 其余逻辑（`save_options`、`preprocess_cfile`、`discover_target` 等）**原样保留**
2. [ ] 修改 `hook/Makefile`：`hook.c` → `hook.cpp`，编译器改为 g++
3. [ ] 复制 `references/c2rust-demo/src/capture.rs` → `src/capture.rs`（无需修改）
4. [ ] 复制 `references/c2rust-demo/src/layout.rs` → `src/layout.rs`（无需修改）
5. [ ] 复制 `references/c2rust-demo/src/selector.rs` → `src/selector.rs`（无需修改）
6. [ ] 复制 `references/c2rust-demo/src/error.rs` → `src/error.rs`（无需修改）
7. [ ] 创建 `src/main.rs`（基于 c2rust-demo 的 `run_init`/`run_merge` 框架，改二进制名为 `cpp2rust-ffi`）

**Phase 1 - opts 解析 + clang 环境**：
1. [ ] 实现 `parse_opts_file(path: &Path) -> Result<Vec<String>>`：读取 `.c2rust.opts`，解析引号包围的参数
2. [ ] 验证 `clang::Index::parser().arguments(&opts).parse()` 能成功解析 `.c2rust` 文件
3. [ ] 确认 `is_in_main_file()` 过滤有效（排除系统头文件展开的噪音节点）
4. [ ] 建立 `CppUnit` 数据结构（表示一个编译单元的提取结果）

**Phase 2 - AST 提取器**：
1. [ ] 实现 `extractor::class::extract(entity)` → `ClassInfo`（含方法列表、构造函数、析构函数）
2. [ ] 实现 `extractor::function::extract(entity)` → `FunctionInfo`（全局函数）
3. [ ] 实现 `extractor::enum_::extract(entity)` → `EnumInfo`
4. [ ] 处理命名空间前缀（`Namespace` 实体的递归遍历）

**Phase 3 - 模板实例化**：
1. [ ] 实现 `extractor::template::extract(entity)` → 识别 `entity.get_template().is_some()`
2. [ ] 记录实例化结果（display_name 如 `"vector<int>"`），作为普通类处理

**Phase 4 - hicc 代码生成器**：
1. [ ] 实现 `generator::cpp_block::generate(class_info)` → `hicc::cpp! { ... }` 字符串（含构造/析构 wrapper）
2. [ ] 实现 `generator::class_block::generate(class_info)` → `hicc::import_class! { ... }` 字符串
3. [ ] 实现 `generator::lib_block::generate(fn_infos)` → `hicc::import_lib! { ... }` 字符串
4. [ ] 实现 `generator::project::generate_mod(unit)` → 写入 `mod_xxx/mod.rs`
5. [ ] 实现 `generator::project::generate_lib(mods)` → 写入 `lib.rs`

**Phase 5 - 降级处理**：
1. [ ] 跳过 variadic 函数并插入 `// cpp2rust-todo[VA]: ...` 注释
2. [ ] 运算符重载 → 生成 named shim + `// cpp2rust-todo[OP]: ...`
3. [ ] 友元函数 → 直接 `import_lib!` + `// cpp2rust-todo[FR]: ...`
4. [ ] typeid/RTTI → 枚举注入 + `// cpp2rust-todo[RTTI]: ...`
5. [ ] Lambda → 跳过 + `// cpp2rust-todo[LM]: ...`

**Phase 6 - merge 命令适配**：
1. [ ] 参考 `references/c2rust-demo/src/split/merge.rs`，适配 hicc 格式的模块合并逻辑
2. [ ] 每个模块目录下是单个 `mod.rs`，合并时直接搬运（不同于 C 版本的 `fun_*.rs/var_*.rs`）
3. [ ] 跨模块重复的 `import_lib!` 声明上移到 `lib.rs`（FFI 去重）

**验收标准（Phase 0-4 完成后）**：
```bash
# 以 examples/006_class_basic 为测试目标
cd examples/006_class_basic
cpp2rust-ffi init -- make
# 预期输出：
# .c2rust/default/c/main.cpp.c2rust
# .c2rust/default/c/main.cpp.c2rust.opts
# .c2rust/default/rust/src/mod_main/mod.rs  ← 包含三段式 hicc 代码
```

---

## 10. 技术依赖

### 10.1 Rust Crates

```toml
[dependencies]
clap = { version = "4", features = ["derive"] }  # CLI（继承自 c2rust-demo）
clang = "2"           # libclang 绑定（C++ AST 解析）—— v5 新增
anyhow = "1"          # 错误处理（继承）
serde = { version = "1", features = ["derive"] }  # JSON 序列化（继承）
serde_json = "1"      # JSON（继承，用于 selected_files.json）
walkdir = "2"         # 目录遍历（继承）
dialoguer = "0.11"    # 交互式选择（继承，selector.rs 依赖）
quote = "1"           # Rust token 生成（代码生成用）
proc-macro2 = "1"     # Rust token 类型（代码生成用）
prettyplease = "0.2"  # Rust 代码格式化（生成结果美化）

# 注：不需要 cc = "1" 作为 build-dependency，
# 因为 capture::build_hook() 直接调用 Makefile，与 c2rust-demo 完全一致
```

### 10.2 系统依赖

```bash
# 构建 hook
apt-get install g++ make

# AST 解析
apt-get install clang libclang-dev libstdc++-dev

# 运行工具（libclang 动态链接）
apt-get install libclang1
```

> **注意**：不需要 `bindgen-cli`（c2rust-demo 需要，v5 不需要，用 clang crate 替代）。

---

## 11. 风险评估

| 风险 | 可能影响 | 缓解措施 |
|------|---------|---------|
| 系统头文件被预处理展开（`-E` 展开量庞大） | `.c2rust` 文件过大（数万行），clang 解析慢 | `is_in_main_file()` 过滤；`skip_function_bodies(true)` 加速 |
| `.c2rust.opts` 解析不完整（如相对路径 `-I`） | clang 找不到头文件，解析报错 | opts 中路径为绝对路径（由 `realpath` 保证），仅需正确解析引号 |
| C++ 模板声明与实例化混淆 | 生成多余或错误的绑定代码 | 过滤 `ClassTemplate` 节点，只处理 `ClassDecl` |
| 跨编译单元的隐式模板实例化缺失 | 部分类型未被捕获 | 合并多个 `.c2rust` 的 AST 分析结果 |
| clang crate v2 API 变化 | 编译失败 | 锁定版本；代码中集中封装 clang 调用，降低改动面 |
| C++ 方法签名含复杂类型（nested template、callback） | 生成的 hicc 注解不合法 | 降级为 `// cpp2rust-todo[COMPLEX]`，保留签名文本 |
| hook.cpp 中 g++ 不在 PATH | LD_PRELOAD 逻辑不触发 | 保留 `C2RUST_CC` 环境变量覆盖机制（与 hook.c 一致） |
