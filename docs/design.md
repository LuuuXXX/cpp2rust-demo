# cpp2rust-demo 设计说明

## 目标

`cpp2rust-demo` 的 `init` 以真实编译链路为入口，自动捕获 C++ 编译单元并生成中间件，不再要求用户维护“手工头文件输入列表”。

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
        │   ├── class/mod.rs + cls_*.rs（类级 inventory/元信息）
        │   ├── method/mod.rs + mtd_*.rs（实例方法）
        │   ├── global/（可选，当前默认不生成）
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
  - `method/`：类实例方法（当前唯一承接 `import_class!`）
  - `class/`：类级 inventory/元信息（如 class 名称清单），不是方法绑定层
- `types/` 当前定位是 type inventory（类型清单），后续可演进为类型绑定层。
- `common/*` 当前定位是 shared inventory/shared context（共享清单/上下文），不是共享绑定层。
- `global/` 当前尚无独立 AST 产物，默认不生成该目录。

merge 语义边界（当前）：
- 参与 merged 输出的目录：`include/`、`types/`、`method/`、`free/`、`class/`。
- 其中：`method/` 贡献 `import_class!`；`free/` 贡献 `import_lib!`。
- `class/` 贡献类级语义元信息块（如 class 维度统计/清单）。
- `common/*` 贡献共享 inventory/context 块到全局 merged_ffi 输出，作为跨 group 的共享语义层。

## hicc 约束

Rust 侧项目搭建统一使用：

- `hicc`
- `hicc-build`
- `build.rs` 中的 `hicc_build::Build`

不再保留与 `hicc` 冲突的自定义构建链路。
