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
8. 抽取函数/类/方法与类型信息，生成按 `mod_<group>` 组织的语义模块（include/types/free/class/method）
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

- 当前语义拆分的实际绑定内容主要是：
  - `include/`：`hicc::cpp!` include 上下文
  - `free/`：自由函数与静态方法
  - `method/`：类实例方法（包含 virtual 与 abstract 两种路径）
- `class/`：类级语义结构层（类名、方法计数、类-方法关系 + 访问函数），不是方法绑定层；仅保留于 per-group 产物供检阅，不参与 merge 输出。
- `types/`：类型语义层（类型清单 + C++→Rust 映射 + 查询函数），per-group 块参与 merge 语义组织。
- `common/*`：共享语义层；`types.rs` 中的枚举定义与类型别名（业务代码）参与 merge 输出；`CPP_TYPES` 等元数据常量与 `includes.rs`（路径元数据）**不**参与 merge 输出。
- `global/`：当前版本不生成，保留为预留扩展点。

**虚函数与抽象类支持**：

| 场景 | 生成方式 |
|------|---------|
| 非纯 virtual 方法（有实现）| 直接提取为 `#[cpp(method = "...")]`，hicc 通过 vtable 透明调用 |
| 全纯虚类（所有公有方法均为 `= 0`）| 提取为 `hicc::import_class!` 中的 `#[interface]` trait |
| 混合类（有普通方法 + 纯虚方法）| 普通方法正常提取；纯虚方法提取为 companion `#[interface]` trait，混合类自动继承该接口 |
| operator 重载 | 跳过，但接口报告新增「Operator Overload Shim Hints」指导手写 C++ shim |

抽取阶段会跳过并报告：constructor、destructor、operator overload、无法解锁的 template declarations、部分 unsupported_type。

merge 语义边界（当前）：
- 参与 `lib.rs` 输出的内容：hicc::cpp! includes、枚举定义、类型别名、`import_class!`、`import_lib!`。
- `method/` 贡献 `import_class!`（包括 `#[interface]`）；`free/` 贡献 `import_lib!`。
- `class/` 仅生成在 per-group `src.1/<stem>/class/` 中供检阅，**不**进入 merge 输出（CLASS_NAMES、CLASS_METHOD_COUNTS 等元信息常量对 FFI 绑定无意义）。
- `common/types.rs` 中的**枚举定义和类型别名**写入 `lib.rs`；`CPP_TYPES`、`CPP_RUST_TYPE_MAPPINGS` 等元数据和 `common/includes.rs`（路径元数据）**不**进入 merge 输出。
- `global/` 当前不参与 merge 产物，为预留扩展点。

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
| 多重继承 | ✅（骨架）/ ❌（运行时） | `method/mtd_*.rs` | 所有 public 基类均提取，生成 `class C: A, B`；hicc 不支持多重继承运行时语义，骨架无法直接使用 |
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
- **operator 重载**：hicc 不支持运算符名称，需手写 C++ shim；工具自动生成 `operator_shims.hpp` starter 辅助填写。
- **多翻译单元**：使用 `init` 多编译单元模式一次捕获全部头文件，通过 `merge` 合并为统一 FFI，见 `examples/rapidjson/08-multi-tu/`。
