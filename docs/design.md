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

## RapidJSON 类场景建议

RapidJSON 等 header-only + 模板/重载密集库，当前仍属于"有限支持"场景：

- **根本瓶颈**：RapidJSON 的核心类型（如 `GenericDocument<Encoding, Allocator>`）均为 `ClassTemplateDecl`（模板类），尚未被 cpp2rust-demo 提取（暂 defer）。
- **虚函数场景**已改善：非模板类的虚函数（包括纯虚接口）可正常生成 hicc 绑定。
- **operator 重载**仍需手写 C++ shim：报告中的「Operator Overload Shim Hints」章节提供了具体写法指导。
- **当前适用场景**：面向非模板 C++ 库生成完整可编译的 hicc FFI 脚手架；模板类建议通过显式实例化或 C++ shim 暴露稳定 ABI 后再绑定。
