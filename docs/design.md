# cpp2rust-demo 设计说明

## 目标

`cpp2rust-demo` 的 `init` 以真实编译链路为入口，自动捕获 C++ 编译单元并生成中间件，不再要求用户维护“手工头文件输入列表”。

它定位为 **hicc FFI 脚手架生成器**，不承诺完整 C++ 语义翻译。

## init 流程

1. 执行 `init -- <BUILD_CMD...>` 并保存 `build_cmd.txt`
2. 编译 `hook/libhook.so`
3. 通过 `LD_PRELOAD` 注入构建，拦截编译器调用
4. 为项目内参与编译的 C++ 编译单元（`.cc/.cpp/.cxx/.c++/.C`）生成 `.cpp2rust` 预处理中间件与 `.opts`（例如 `a.cpp -> a.cpp.cpp2rust`）；同时在 capture 目录下创建同名符号链接（`a.cpp → a.cpp.cpp2rust`），使生成的 `hicc::cpp! { #include "a.cpp" }` 可由 hicc-build 解析
5. 扫描 `.cpp2rust/<feature>/cpp/**/*.cpp2rust`
6. 交互式选择参与转换的中间件文件（非交互自动全选）
7. 对选中文件执行 `clang -ast-dump=json`
8. 抽取函数/类/方法与类型信息，为每个选中的翻译单元生成一个平铺的 `<stem>.rs`（1:1 映射，包含完整 hicc 脚手架）
9. 生成 `Cargo.toml` / `build.rs` / `src/lib.rs` 与接口报告

说明：
- 自动捕获路径不直接记录头文件（`.h/.hpp/.hh/.hxx`）。
- 头文件内容通过捕获到的编译单元在预处理阶段展开，再由后续 AST/hicc 流程提取接口信息。
- 对 header-only 库建议使用 synthetic translation unit（`entry.cpp` 仅 `#include` 目标头文件）触发流程。
- 可通过 `init --no-link`（别名 `--header-only`）启用 no-link 模式，避免 `build.rs` 强制链接不存在的目标库。

## 目录结构

```text
.cpp2rust/<feature>/
├── cpp/      # *.cpp2rust + *.cpp2rust.opts + *.cpp symlinks (指向对应 .cpp2rust)
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
        │   ├── includes.rs  # 中间件路径元数据（不参与 merge 输出）
        │   └── types.rs     # 聚合枚举定义与类型别名（参与 merge 输出）
        ├── <stem>.rs          # 平铺模块（1:1 对应翻译单元），含完整 hicc 脚手架
        │                      # （hicc::cpp! + import_class! + import_lib! + 元数据常量）
        ├── <stem>.meta.json   # 该翻译单元的元数据清单
        ├── (merge 后) src -> src.2
        ├── src.1/
        └── src.2/
            ├── lib.rs          # 合并后 FFI 入口（汇聚所有翻译单元）
            └── <stem>.rs       # 各翻译单元参考文件（不直接编译）
```

## merge 流程

`merge` 会读取 `rust/src/` 中的平铺 `<stem>.rs` 文件并合并：

- 将所有翻译单元的 `hicc::cpp!` / `hicc::import_class!` / `hicc::import_lib!` 内容聚合，直接写入 `rust/src.2/lib.rs`
- 同时按 stem 生成 `rust/src.2/<stem>.rs`（参考文件，不直接编译）
- **不再生成单独的 `merged_ffi.rs`**——因编译单元已 1:1 平铺，`lib.rs` 即全局视图
- 非业务元数据（`CPP_TYPES`、`CPP_RUST_TYPE_MAPPINGS`、`CLASS_NAMES` 等）**不**写入 `lib.rs`，仅枚举定义与类型别名（业务代码）保留
- 完成后将 init 原始 `rust/src` 备份为 `rust/src.1`，并将 `rust/src` 切换为指向 `src.2` 的符号链接
- `build.rs` 持续引用 `src/lib.rs`，依赖该活跃视图机制在 merge 后自动指向 `src.2/lib.rs`
- 使用 `merge --output <dir>` 导出时，同时将 `meta/operator_shims.hpp` 与 `meta/init-interface-report.md` 复制到 `<output>/meta/`

## v1 能力边界（当前实现）

- 每个翻译单元的全部绑定内容集中在一个平铺的 `<stem>.rs` 文件中，按以下顺序排列：
  1. `hicc::cpp!`：中间件 include 上下文
  2. `#[repr(C)] pub enum`：提取的 C++ 枚举定义
  3. `pub type`：typedef / using 别名
  4. `hicc::import_class!`：每个 C++ 类一个块（含 `#[interface]` trait）
  5. `hicc::import_lib!`：自由函数、静态方法、构造工厂函数、全局变量
  6. 类级元数据常量（`CLASS_COUNT`、`CLASS_NAMES` 等，供检阅）
  7. C++ 类型元数据（`CPP_TYPES`、`CPP_RUST_TYPE_MAPPINGS` 等，供检阅）
  8. 激活的 operator shim Rust 绑定 + 自动插入的 `hicc::cpp!{#include "operator_shims.hpp"}` 块（如有运算符重载）
  9. 注释掉的 `@dynamic_cast` 骨架（如有继承关系）
  10. 注释掉的 `@placement_new` 骨架（如有构造函数）
- `common/types.rs`：跨 TU 聚合的枚举定义与类型别名（业务代码）参与 merge 输出；
  `CPP_TYPES`、`CLASS_NAMES` 等元数据常量与 `common/includes.rs`（路径元数据）**不**参与 merge 输出。

**虚函数与抽象类支持**：

| 场景 | 生成方式 |
|------|---------|
| 非纯 virtual 方法（有实现）| 直接提取为 `#[cpp(method = "...")]`，hicc 通过 vtable 透明调用 |
| 全纯虚类（所有公有方法均为 `= 0`）| 提取为 `hicc::import_class!` 中的 `#[interface]` trait |
| 混合类（有普通方法 + 纯虚方法）| 普通方法正常提取；纯虚方法提取为 companion `#[interface]` trait，混合类自动继承该接口 |
| operator 重载 | 跳过提取到 `import_class!`，但自动生成完整 `operator_shims.hpp` C++ shim 函数体，并在 `<stem>.rs` 中插入激活的 `import_lib!` 绑定；`hicc::cpp!` include 和 build.rs include 路径均自动配置 |

抽取阶段会跳过并报告：constructor、destructor、operator overload、无法解锁的 template declarations、部分 unsupported_type。

merge 语义边界（当前）：
- 参与 `lib.rs` 输出的内容：`hicc::cpp!` includes、枚举定义、类型别名、`import_class!`、`import_lib!`。
- 类级元数据常量（`CLASS_NAMES`、`CLASS_METHOD_COUNTS` 等）以及类型元数据（`CPP_TYPES`、`CPP_RUST_TYPE_MAPPINGS` 等）**不**进入 merge 输出（对 FFI 绑定无意义）。
- `common/types.rs` 中的**枚举定义和类型别名**写入 `lib.rs`；`common/includes.rs`（路径元数据）**不**进入 merge 输出。

## hicc 约束

Rust 侧项目搭建统一使用：

- `hicc`
- `hicc-build`
- `build.rs` 中的 `hicc_build::Build`

## 完整能力矩阵

下表覆盖 hicc 所有已知的绑定能力。标记说明：
- ✅ **已支持**：cpp2rust-demo 自动提取，无需用户干预。
- ⚠️ **有条件支持（ToolConservative）**：满足条件时自动支持；不满足时跳过并在报告中标记为 `tool_conservative`（可通过用户操作解锁）。
- ❌ **需要 C++ shim（HiccLimitation）**：hicc 本身不支持，需要手写 C++ 包装函数；cpp2rust-demo 会生成 starter shim 文件辅助用户补全。

| C++ 特性 | 状态 | 输出位置 | 说明 |
|----------|------|---------|------|
| 自由函数（非模板） | ✅ | `<stem>.rs` | `import_lib!` + `#[cpp(func = "...")]` |
| 函数重载 | ✅ | `<stem>.rs` | 自动追加 `_2`, `_3`, … 后缀 |
| 命名空间函数 | ✅ | `<stem>.rs` | 限定名嵌入 `#[cpp(func = "ns::foo(...)")]` |
| 类实例方法（含 `void*` 等指针返回类型） | ✅ | `<stem>.rs` | `import_class!` + `#[cpp(method = "...")]`；指针返回类型（如 `void*(size_t)`）与非指针返回类型完全相同路径提取 |
| `const` 方法 | ✅ | `<stem>.rs` | 映射为 `fn foo(&self)` |
| 非 `const` 方法 | ✅ | `<stem>.rs` | 映射为 `fn foo(&mut self)` |
| 非纯 `virtual` 方法 | ✅ | `<stem>.rs` | hicc 通过 vtable 透明调用 |
| 全纯虚类（抽象接口）| ✅ | `<stem>.rs` | `#[interface]` trait 语法 |
| 混合类（部分纯虚）| ✅ | `<stem>.rs` | 普通方法正常提取；纯虚方法生成 companion interface |
| 构造函数 | ✅ | `<stem>.rs` | 主构造函数 `ctor="..."`；额外构造函数为工厂函数 |
| `static` 方法 | ✅ | `<stem>.rs` | `import_lib!` + `#[cpp(func = "ClassName::method(...)")]` |
| public 继承 | ✅ | `<stem>.rs` | `class Derived: Base` 语法 |
| `@make_proxy` | ✅ | `<stem>.rs` | 全纯虚类自动生成；支持 Rust 实现 C++ 接口 |
| 全局变量 | ✅ | `<stem>.rs` | `#[cpp(data = "...")]` 绑定 |
| 枚举（`enum`/`enum class`）| ✅ | `<stem>.rs` | `#[repr(C)] enum` |
| `typedef`/`using` 别名 | ✅ | `<stem>.rs` | 注册到 AliasRegistry，解锁模板提取 |
| 模板特化（有别名） | ⚠️ | `<stem>.rs` | 需要 `typedef`/`using` 别名；见下方 AliasRegistry 指南 |
| 模板类（无别名） | ⚠️ | — | 跳过并标记 `tool_conservative`；添加别名后可解锁 |
| 运算符重载 | ✅ | `<stem>.rs`（激活的 `import_lib!` 绑定）+ `meta/operator_shims.hpp` | 自动生成完整 C++ shim 函数体；`hicc::cpp!` include、Rust 绑定、build.rs include 路径均全自动配置；标准运算符无需用户干预 |
| 析构函数 | ❌ | — | hicc 不支持显式析构；跳过并标记 `hicc_limitation` |
| 友元函数 | ❌ | — | AST 不可靠提取；跳过 |
| 多重继承 | ✅（骨架）/ ❌（运行时） | `<stem>.rs` | 所有 public 基类均提取，生成 `class C: A, B`；hicc 不支持多重继承运行时语义，骨架无法直接使用 |
| 函数指针参数 | ❌ | — | 含 `(*)` 的类型跳过，标记 `hicc_limitation` |
| `std::` 容器参数 | ⚠️ | — | 无别名时跳过；为容器类型添加 `using` 别名可解锁 |
| variadic 函数 (`...`) | ❌ | — | 跳过，标记 `hicc_limitation` |
| `auto`/`decltype` 返回 | ❌ | — | 跳过，标记 `hicc_limitation` |

## 解锁模板类提取（AliasRegistry 指南）

cpp2rust-demo 通过 **AliasRegistry** 解锁模板特化的 FFI 提取。

工作原理：当 clang 在 entry.cpp 中看到 `typedef`/`using` 别名声明时，工具自动注册三张映射：
- 裸模板名 → **所有**别名列表（如 `"GenericDocument"` → `["Document", "FastDocument"]`，1:N）
- 别名 → 完整限定类型（如 `"Document"` → `"rapidjson::GenericDocument<...>"`）
- 完整限定类型 → 首个别名（精确反向查找，供不同特化各取自己的别名）

同一模板的**不同特化可以各自拥有独立别名**，提取时精确匹配完整特化类型，两个特化各自生成独立 Rust struct：
```cpp
using IntBox = Box<int>;          // → struct IntBox
using StrBox = Box<std::string>;  // → struct StrBox（独立提取，不会覆盖 IntBox）
```

提取时，若方法参数类型含 `<`，则优先按完整限定类型精确查找别名；若无完整类型信息则退而查裸模板名的首个别名；两者均无则跳过并标记 `tool_conservative`。

**解锁方式**：在 entry.cpp 中 `#include` 包含 `typedef`/`using` 别名的头文件，或手动添加：
```cpp
using FastDoc = rapidjson::GenericDocument<rapidjson::UTF8<char>,
                    rapidjson::MemoryPoolAllocator<rapidjson::CrtAllocator>>;
```

**常见问题**：

| 现象 | 解决方案 |
|------|---------|
| 报告中出现大量 `tool_conservative` 跳过 | 在 entry.cpp 添加 `typedef`/`using` 别名 |
| 方法被跳过但参数类型看起来是已知类 | 检查 entry.cpp 是否 `#include` 了定义别名的头文件 |
| `class Derived: Base` 没有出现 | 为基类模板添加别名 |

## RapidJSON 类场景建议

- **模板别名**：RapidJSON 核心类型（`Document`、`Value`、`Writer` 等）的 `typedef` 别名已内置于头文件，`#include` 后工具自动提取。
- **虚函数**：非模板类的虚函数（含纯虚接口）可正常生成 hicc 绑定。
- **operator 重载**：hicc 不支持运算符名称；工具自动生成完整 `operator_shims.hpp` C++ shim 函数体，并在 `<stem>.rs` 中插入激活的 `import_lib!` 绑定——标准运算符（`=`、`[]`、`==`、`!=`、`<`、`+` 等）无需用户干预。
- **多翻译单元**：使用 `init` 多编译单元模式一次捕获全部头文件，通过 `merge` 合并为统一 FFI，见 `examples/rapidjson/08-multi-tu/`。
