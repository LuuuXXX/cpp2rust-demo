# cpp2rust-demo 设计说明

## 目标

`cpp2rust-demo` 的 `init` 以真实编译链路为入口，自动捕获 C++ 编译单元并生成中间件，不再要求用户维护“手工头文件输入列表”。

它定位为 **hicc FFI 脚手架生成器**，不承诺完整 C++ 语义翻译。

## init 流程

1. 执行 `init -- <BUILD_CMD...>` 并保存 `build_cmd.txt`
2. 编译 `hook/libhook.so`
3. 通过 `LD_PRELOAD` 注入构建，拦截编译器调用
4. 为项目内参与编译的 C++ 编译单元（`.cc/.cpp/.cxx/.c++/.C`）生成 `.cpp2rust` 预处理中间件与 `.opts`（例如 `a.cpp -> a.cpp.cpp2rust`）
5. 扫描 `.cpp2rust/<feature>/cpp/**/*.cpp2rust`
6. 交互式选择参与转换的中间件文件（非交互自动全选）
7. 对选中文件执行 `clang -ast-dump=json`
8. 抽取函数/类/方法与类型信息，生成按 `mod_<group>` 组织的语义模块（include/types/free/class/method/global）
9. 生成 `Cargo.toml` / `build.rs` / `src/lib.rs` 与接口报告

说明：
- 自动捕获路径不直接记录头文件（`.h/.hpp/.hh/.hxx`）。
- 头文件内容通过捕获到的编译单元在预处理阶段展开，再由后续 AST/hicc 流程提取接口信息。
- 对 header-only 库建议使用 synthetic translation unit（`entry.cpp` 仅 `#include` 目标头文件）触发流程。
- 可通过 `init --no-link`（别名 `--header-only`）启用 no-link 模式，避免 `build.rs` 强制链接不存在的目标库。

## 目录结构

```text
.cpp2rust/<feature>/
├── cpp/      # *.cpp2rust + *.cpp2rust.opts
├── ast/      # *.ast.json
├── meta/
│   ├── build_cmd.txt
│   ├── selected_files.json
│   ├── headers.json
│   ├── init-interface-report.md
│   └── merge-report.md
└── rust/
    ├── Cargo.toml
    ├── build.rs
    └── src/
        ├── lib.rs
        ├── common/
        │   ├── mod.rs
        │   ├── includes.rs
        │   └── types.rs
        ├── mod_<group>/
        │   ├── mod.rs
        │   ├── include/mod.rs
        │   ├── types/mod.rs
        │   ├── free/mod.rs + fn_*.rs
        │   ├── class/mod.rs + cls_*.rs（类级语义结构/元信息）
        │   ├── method/mod.rs + mtd_*.rs（实例方法）
        │   └── meta.json
        ├── (merge 后) -> src.2
        ├── src.1/
        └── src.2/
            ├── lib.rs
            ├── mod_<group>.rs
            └── merged_ffi.rs
```

## merge 流程

`merge` 会读取 `rust/src/mod_<group>/` 并合并：

- 按 group 生成 `rust/src.2/mod_<group>.rs`
- 额外生成全局 `rust/src.2/merged_ffi.rs`
- 同时生成 `rust/src.2/lib.rs`
- 完成后将 init 原始 `rust/src` 备份为 `rust/src.1`，并将 `rust/src` 切换为指向 `src.2` 的符号链接
- `build.rs` 持续引用 `src/...` 路径，依赖该活跃视图机制在 merge 后自动指向 `src.2` 产物

## v1 能力边界（当前实现）

- 当前语义拆分的实际绑定内容主要是：
  - `include/`：`hicc::cpp!` include 上下文
  - `free/`：自由函数与静态方法
  - `method/`：类实例方法（包含 virtual 与 abstract 两种路径）
- `class/`：类级语义结构层（类名、方法计数、类-方法关系 + 访问函数），不是方法绑定层。
- `types/`：类型语义层（类型清单 + C++→Rust 映射 + 查询函数），参与 merge 语义组织。
- `common/*`：共享语义层（共享 include/type 索引 + 查询函数），参与全局 merge 语义组织。
- `global/`：本 PR 明确 defer，不属于当前完整语义结构承诺范围。

**虚函数与抽象类支持（新增）**：

| 场景 | 生成方式 |
|------|---------|
| 非纯 virtual 方法（有实现）| 直接提取为 `#[cpp(method = "...")]`，hicc 通过 vtable 透明调用 |
| 全纯虚类（所有公有方法均为 `= 0`）| 提取为 `hicc::import_class!` 中的 `#[interface]` trait |
| 混合类（有普通方法 + 纯虚方法）| 普通方法正常提取；纯虚方法记录为 skipped（保守处理）|
| operator 重载 | 跳过，但接口报告新增「Operator Overload Shim Hints」指导手写 C++ shim |

抽取阶段仍会跳过并报告：constructor、destructor、operator overload、template declarations、部分 unsupported_type、混合类中的纯虚方法。

merge 语义边界（当前）：
- 参与 merged 输出的目录：`include/`、`types/`、`method/`、`free/`、`class/`。
- 其中：`method/` 贡献 `import_class!`（包括 `#[interface]`）；`free/` 贡献 `import_lib!`。
- `class/` 贡献类级语义结构块（如 class 维度统计、类-方法关系）。
- `common/*` 贡献共享语义块到全局 merged_ffi 输出，作为跨 group 的共享语义层。

## hicc 约束

Rust 侧项目搭建统一使用：

- `hicc`
- `hicc-build`
- `build.rs` 中的 `hicc_build::Build`

不再保留与 `hicc` 冲突的自定义构建链路。

## 完整能力矩阵

下表覆盖 hicc 所有已知的绑定能力。标记说明：
- ✅ **已支持**：cpp2rust-demo 自动提取，无需用户干预。
- ⚠️ **有条件支持（ToolConservative）**：满足条件时自动支持；不满足时跳过并在报告中标记为 `tool_conservative`（可通过用户操作解锁）。
- ❌ **需要 C++ shim（HiccLimitation）**：hicc 本身不支持，需要手写 C++ 包装函数；cpp2rust-demo 会生成 starter shim 文件辅助用户补全。

| C++ 特性 | 状态 | 输出位置 | 说明 |
|----------|------|---------|------|
| 自由函数（非模板） | ✅ | `free/fn_*.rs` | `import_lib!` + `#[cpp(func = "...")]` |
| 函数重载 | ✅ | `free/fn_*.rs` | 自动追加 `_2`, `_3`, … 后缀 |
| 命名空间函数 | ✅ | `free/fn_*.rs` | 限定名嵌入 `#[cpp(func = "ns::foo(...)")]` |
| 类实例方法 | ✅ | `method/mtd_*.rs` | `import_class!` + `#[cpp(method = "...")]` |
| `const` 方法 | ✅ | `method/mtd_*.rs` | 映射为 `fn foo(&self)` |
| 非 `const` 方法 | ✅ | `method/mtd_*.rs` | 映射为 `fn foo(&mut self)` |
| 非纯 `virtual` 方法 | ✅ | `method/mtd_*.rs` | hicc 通过 vtable 透明调用 |
| 全纯虚类（抽象接口）| ✅ | `method/mtd_*.rs` | `#[interface]` trait 语法 |
| 混合类（部分纯虚）| ✅ | `method/mtd_*.rs` | 普通方法正常提取；纯虚方法生成 companion interface |
| 构造函数 | ✅ | `method/mtd_*.rs` | 主构造函数 `ctor="..."`；额外构造函数为工厂函数 |
| `static` 方法 | ✅ | `free/fn_*.rs` | `import_lib!` + `#[cpp(func = "ClassName::method(...)")]` |
| public 继承 | ✅ | `method/mtd_*.rs` | `class Derived: Base` 语法 |
| `@make_proxy` | ✅ | `free/fn_*.rs` | 全纯虚类自动生成；支持 Rust 实现 C++ 接口 |
| 全局变量 | ✅ | `free/fn_*.rs` | `#[cpp(data = "...")]` 绑定 |
| 枚举（`enum`/`enum class`）| ✅ | `types/mod.rs` | `#[repr(C)] enum` |
| `typedef`/`using` 别名 | ✅ | `types/mod.rs` | 注册到 AliasRegistry，解锁模板提取 |
| 模板特化（有别名） | ⚠️ | `method/mtd_*.rs` | 需要 `typedef`/`using` 别名；见下方 AliasRegistry 指南 |
| 模板类（无别名） | ⚠️ | — | 跳过并标记 `tool_conservative`；添加别名后可解锁 |
| 运算符重载 | ⚠️ | `free/shim_ops.rs` | 生成 `operator_shims.hpp` starter；需用户补全实现后使用 |
| 析构函数 | ❌ | — | hicc 不支持显式析构；跳过并标记 `hicc_limitation` |
| 友元函数 | ❌ | — | AST 不可靠提取；跳过 |
| 多重继承 | ❌ | — | 仅处理首个 public 基类；其余跳过 |
| 函数指针参数 | ❌ | — | 含 `(*)` 的类型跳过，标记 `hicc_limitation` |
| `std::` 容器参数 | ⚠️ | — | 无别名时跳过；为容器类型添加 `using` 别名可解锁 |
| variadic 函数 (`...`) | ❌ | — | 跳过，标记 `hicc_limitation` |
| `auto`/`decltype` 返回 | ❌ | — | 跳过，标记 `hicc_limitation` |

## 解锁模板类提取（AliasRegistry 指南）

cpp2rust-demo 通过 **AliasRegistry** 解锁模板特化的 FFI 提取。工作原理：

### 第一步：在 entry.cpp 中添加 `using` 别名声明

```cpp
// entry.cpp — header-only library entry point
#include "rapidjson/document.h"

// RapidJSON 的头文件中已内置以下 typedef，clang 可见后自动注册：
//   typedef GenericDocument<UTF8<char>> Document;
//   typedef GenericValue<UTF8<char>>    Value;
//   typedef GenericMember<UTF8<char>>   Member;

// 如果需要额外别名（例如自定义 allocator 特化），在此添加：
// using FastDocument = rapidjson::GenericDocument<rapidjson::UTF8<char>,
//                          rapidjson::MemoryPoolAllocator<rapidjson::CrtAllocator>>;
```

### 第二步：工具自动建立两张映射表

```
template_to_alias:
  "GenericDocument" → "Document"
  "GenericValue"    → "Value"
  "GenericMember"   → "Member"

alias_to_type:
  "Document" → "rapidjson::GenericDocument<rapidjson::UTF8<char>, ...>"
  "Value"    → "rapidjson::GenericValue<rapidjson::UTF8<char>>"
```

**注意**：`bare_template_name()` 函数负责从完整限定类型名提取裸模板名（先剥离 `<>` 模板参数，再剥离命名空间），确保 `"rapidjson::GenericDocument<rapidjson::UTF8<char>, rapidjson::CrtAllocator>"` → `"GenericDocument"` 而不是错误的 `"CrtAllocator>"`。

### 第三步：类型门自动放行已别名的参数类型

在处理方法时，若参数类型为 `rapidjson::GenericValue<...>`，类型门检查：
1. 类型含 `<` → 进入模板路径
2. 调用 `bare_template_name()` 获得 `"GenericValue"`
3. 查询 `AliasRegistry`：`"GenericValue"` 已注册别名 → **放行**
4. 方法被提取，参数类型在生成代码中用别名名 `Value` 表示

### 常见问题

| 现象 | 原因 | 解决方案 |
|------|------|---------|
| 报告中出现大量 `tool_conservative` 跳过 | 模板类无别名 | 在 entry.cpp 添加 `typedef`/`using` 别名 |
| 方法被跳过，但参数类型看起来是已知类 | 参数是命名空间限定的模板类型，别名未注册 | 检查 entry.cpp 是否包含定义别名的头文件 |
| `class Derived: Base` 没有出现 | 基类是模板，别名未注册 | 为基类模板添加别名 |

## RapidJSON 类场景建议

RapidJSON 等 header-only + 模板/重载密集库，经过 AliasRegistry 改进后，支持程度显著提升：

- **模板别名已支持**：RapidJSON 的核心类型（`Document`、`Value`、`Writer` 等）通过 `typedef` 别名可自动提取。
- **虚函数场景**：非模板类的虚函数（包括纯虚接口）可正常生成 hicc 绑定。
- **operator 重载**仍需手写 C++ shim：报告中的「Operator Overload Shim Hints」章节提供了具体写法指导，`operator_shims.hpp` starter 文件会自动生成。
- **多翻译单元场景**：使用 `init` 的多编译单元模式捕获全部头文件，再通过 `merge` 合并为统一 FFI，见 `examples/rapidjson-08-multi-tu/`。
