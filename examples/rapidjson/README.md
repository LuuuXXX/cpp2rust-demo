# RapidJSON 示例

本示例展示如何使用 `cpp2rust-demo`，基于真实的 C++ 开源项目
[Tencent/rapidjson](https://github.com/Tencent/rapidjson) 生成 Rust FFI 脚手架。

## 示例目标

本示例重点演示：

1. 如何在真实 C++ 项目中运行 `cpp2rust-demo init`
2. 如何通过交互式选择头文件控制生成范围
3. 如何执行 `merge` 生成最终的单一 FFI 文件

> **说明**：RapidJSON 是一个大量使用模板、头文件包含和复杂 C++ 特性的库。
> 本示例的目标是演示**生成初始 FFI 脚手架**的流程，而不是承诺对整个 RapidJSON 库进行完整的自动转换。
> 自动生成的结果通常需要根据实际使用场景进行手工调整。

---

## 准备工作

### 1. 确保已安装 cpp2rust-demo

从仓库根目录构建：

```bash
cargo build --release
# 将 target/release/cpp2rust-demo 加入 PATH，或直接用 cargo run -- 替代
```

### 2. 克隆 RapidJSON

```bash
git clone https://github.com/Tencent/rapidjson.git
cd rapidjson
```

后续所有命令均在 `rapidjson/` 根目录下执行。

---

## 推荐起步头文件

RapidJSON 是 header-only 库，所有实现都在 `include/rapidjson/` 中。
建议第一次只从单个核心头文件开始，而不是一次性全量选择：

| 头文件 | 内容 |
|--------|------|
| `include/rapidjson/document.h` | DOM 风格 API（`Document`、`Value`） |
| `include/rapidjson/reader.h` | SAX 风格读取器 |
| `include/rapidjson/writer.h` | JSON 序列化（`Writer`） |

第一次尝试推荐使用 `document.h`。

---

## 运行 init

以 `document.h` 为例，在 `rapidjson/` 根目录执行：

```bash
cpp2rust-demo init \
  --feature rapidjson-doc \
  --link rapidjson \
  -- clang++ -x c++ -std=c++14 -Iinclude -fsyntax-only include/rapidjson/document.h
```

参数说明：

- `--feature rapidjson-doc`：为本次生成指定一个特性名，输出目录为 `.cpp2rust/rapidjson-doc/`
- `--link rapidjson`：生成的 `import_lib!` 块中使用的链接库名
- `-- clang++ -x c++ -std=c++14 -Iinclude -fsyntax-only include/rapidjson/document.h`：
  直接用 clang++ 做语法检查，无需 RapidJSON 完整 CMake 构建

执行后，工具会：

1. 编译 `hook/libhook.so`
2. 以 `LD_PRELOAD` 注入上述 clang++ 命令，捕获涉及的头文件路径
3. 展示交互式多选菜单，让你选择哪些头文件参与 FFI 生成：

   ```
   ? Select headers to include in this feature (space to toggle, enter to confirm) ›
   ✔ /path/to/rapidjson/include/rapidjson/document.h
   ✔ /path/to/rapidjson/include/rapidjson/reader.h
     /path/to/rapidjson/include/rapidjson/internal/...
   ```

   建议第一次只保留 `document.h`，避免引入大量传递依赖的头文件。

4. 对选中的头文件执行 clang AST 解析
5. 生成 Rust FFI 代码

---

## 运行 merge

```bash
cpp2rust-demo merge --feature rapidjson-doc
```

`merge` 会将所有 `ffi_*.rs` 中的 `import_class!` 和 `import_lib!` 块合并为一个
`merged_ffi.rs` 文件。

---

## 输出位置

生成结果位于：

```text
.cpp2rust/rapidjson-doc/
├── ast/
│   └── document.ast.json         # clang AST 解析结果（JSON，用于调试）
├── meta/
│   ├── build_cmd.txt             # 本次捕获使用的构建命令
│   ├── captured_headers.list     # LD_PRELOAD hook 捕获到的所有头文件
│   ├── selected_headers.json     # 最终选中的头文件
│   ├── headers.json              # 用于 AST 解析与代码生成的头文件集合
│   └── init-interface-report.md  # 接口报告
└── rust/
    ├── Cargo.toml
    ├── build.rs
    └── src/
        ├── lib.rs
        ├── ffi_document.rs       # document.h 对应的 FFI 代码
        └── merged_ffi.rs         # merge 后的单一 FFI 文件
```

查看生成的 FFI：

```bash
cat .cpp2rust/rapidjson-doc/rust/src/merged_ffi.rs
```

---

## 逐步扩大范围

RapidJSON 头文件之间包含关系较复杂，建议分步尝试：

1. **第一次**：只保留 `document.h`，观察输出结构和代码质量
2. **第二次**：同时加入 `writer.h`，尝试写入场景：

   ```bash
   cpp2rust-demo init \
     --feature rapidjson-rw \
     --link rapidjson \
     -- sh -c "clang++ -x c++ -std=c++14 -Iinclude -fsyntax-only include/rapidjson/document.h && \
               clang++ -x c++ -std=c++14 -Iinclude -fsyntax-only include/rapidjson/writer.h"
   cpp2rust-demo merge --feature rapidjson-rw
   ```

3. **第三次**：根据实际需求逐步扩大头文件范围

---

## 注意事项

- `cpp2rust-demo` 当前仅支持 **Linux** 环境。
- RapidJSON 大量使用 C++ 模板（`GenericDocument`、`GenericValue` 等），
  自动生成的 FFI 只能覆盖工具能静态分析到的非模板声明，模板实例化需要手工处理。
- 生成结果中如出现空的 `import_lib!` 块或缺少某些方法，属于正常现象——
  这类特性目前需要手工补充。
- 建议将生成结果视为 **FFI 初始脚手架**，在此基础上手工完善，而非直接发布为成品绑定。
