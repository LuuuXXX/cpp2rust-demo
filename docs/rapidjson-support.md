# RapidJSON 支持文档

本文档描述 `cpp2rust-demo` 对 [Tencent/rapidjson](https://github.com/Tencent/rapidjson)（MIT 协议，header-only C++ JSON 库）的完整绑定方案，包括：

- 支持与不支持的特性说明
- 完整的本地复现验证流程
- CI 对应关系与产物结构说明

---

## 一、RapidJSON 概况与绑定策略

RapidJSON 是纯 header-only 库，无预编译二进制，主要特点：

- 大量模板类（`GenericDocument<Enc,Alloc,StackAlloc>`、`GenericValue<Enc,Alloc>` 等）
- 丰富的 `typedef` 别名（`Document`、`Value`、`Writer<StringBuffer>` 等）
- 运算符重载（`operator[]`、`operator=`、`operator bool` 等）
- 纯虚接口（自定义 allocator）
- 多头文件组织，实际项目往往多翻译单元

**绑定策略**：

1. 使用 `--no-link` 模式（无需链接库）
2. 为每个 RapidJSON 头文件创建一个 synthetic 编译单元（`entry-xxx.cpp`），触发预处理展开
3. 多次 `init` 累积到同一 feature，`merge` 合并为全局视图
4. 模板类依赖 RapidJSON 内置的 `typedef` 别名（无需额外 `using` 声明）
5. 运算符通过 `operator_shims.hpp` 三步工作流处理

---

## 二、支持特性矩阵（RapidJSON 维度）

| RapidJSON 组件 | C++ 特性 | 绑定状态 | 输出位置 |
|---------------|---------|---------|---------|
| `ParseErrorCode` 枚举 | `enum` | ✅ | `types/mod.rs` |
| `Type` 枚举（Value 类型） | `enum` | ✅ | `types/mod.rs` |
| `Document` (GenericDocument 别名) | 模板特化 + typedef | ✅ | `method/mtd_*.rs` |
| `Value` (GenericValue 别名) | 模板特化 + typedef | ✅ | `method/mtd_*.rs` |
| `Writer<StringBuffer>` 别名 | 模板特化 + typedef | ✅ | `method/mtd_*.rs` |
| `PrettyWriter<StringBuffer>` 别名 | 模板特化 + typedef | ✅ | `method/mtd_*.rs` |
| `StringBuffer` | 普通类 | ✅ | `method/mtd_*.rs` |
| `Reader` 相关方法 | 普通类/方法 | ✅ | `method/mtd_*.rs` |
| `Pointer` | 模板特化 + typedef | ✅ | `method/mtd_*.rs` |
| 自定义 Allocator 接口 | 纯虚类 + `@make_proxy` | ✅ | `method/mtd_*.rs` + `free/fn_*.rs` |
| 非虚方法（`isNull()`, `getInt()` 等） | 普通方法 | ✅ | `method/mtd_*.rs` |
| `const` 方法 | `const` 方法 | ✅ | `method/mtd_*.rs` |
| 全局函数（`parseErrorName` 等） | 自由函数 | ✅ | `free/fn_*.rs` |
| `operator[]`、`operator=` 等 | 运算符重载 | 🔧 shim | `free/shim_ops.rs` + `meta/operator_shims.hpp` |
| 析构函数 | 析构函数 | ❌ 跳过 | — |
| `std::basic_ostream` 参数（`operator<<`）| `std::` 类型 | ❌ 跳过 | — |
| `std::allocator` 模板参数 | 复杂模板参数 | ⚠️ 依赖别名 | — |
| 多重继承（`SchemaValidator` 等） | 多重继承 | ⛔ 首个 base | — |

---

## 三、本地复现验证流程

### 前置条件

```bash
# 工具依赖
# - Linux（LD_PRELOAD 机制）
# - Rust/Cargo（rust-version >= 1.82）
# - clang（>= 9，推荐 14+）
# - gcc, make（用于 build hook）
# - git

# 验证工具是否齐全
rustc --version    # rustc 1.82.0 或更新
clang --version    # clang 9+ 推荐
clang++ --version
git --version
```

### 步骤 1：构建 cpp2rust-demo

```bash
# 进入仓库根目录
cd /path/to/cpp2rust-demo

# Release 构建（推荐，与 CI 一致）
cargo build --release

# 验证二进制
./target/release/cpp2rust-demo --version
```

### 步骤 2：克隆 Tencent/rapidjson

```bash
# 浅克隆（节省时间）
git clone --depth=1 https://github.com/Tencent/rapidjson.git /tmp/rapidjson

# 进入 rapidjson 目录（后续所有操作均在此目录）
cd /tmp/rapidjson
```

### 步骤 3：准备 Synthetic 编译单元

RapidJSON 是 header-only 库，需要为每个头文件创建一个"入口" `.cpp` 文件：

```bash
cd /tmp/rapidjson

# 覆盖所有之前的运行产物（保证幂等）
rm -rf .cpp2rust

# 为六个头文件维度创建编译单元
cat > document.cpp     <<CPP
#include "rapidjson/document.h"
CPP
cat > reader.cpp       <<CPP
#include "rapidjson/reader.h"
CPP
cat > writer.cpp       <<CPP
#include "rapidjson/writer.h"
CPP
cat > prettywriter.cpp <<CPP
#include "rapidjson/prettywriter.h"
CPP
cat > pointer.cpp      <<CPP
#include "rapidjson/pointer.h"
CPP
cat > schema.cpp       <<CPP
#include "rapidjson/schema.h"
CPP
```

### 步骤 4：运行 `cpp2rust-demo init`

六个编译单元一次性传给 init（通过 `sh -c "cmd1 && cmd2 && ..."` 链接）：

> **`--link` 与 `--no-link` 说明**：两个标志用途不同，同时使用不矛盾。
> `--link rapidjson` 设置生成代码中 `import_lib!` 的 `link_name`（逻辑库名）；
> `--no-link` 告知工具不向 `build.rs` 注入 `cargo::rustc-link-lib=rapidjson`，
> 因为 RapidJSON 是 header-only 库，不需要预编译目标可链接。

```bash
CPP2RUST=/path/to/cpp2rust-demo/target/release/cpp2rust-demo
FEATURE="default"

cd /tmp/rapidjson

"${CPP2RUST}" init \
    --feature "${FEATURE}" \
    --link rapidjson \
    --no-link \
    -- sh -c "
        clang++ -x c++ -std=c++11 -fsyntax-only -Iinclude document.cpp &&
        clang++ -x c++ -std=c++11 -fsyntax-only -Iinclude reader.cpp &&
        clang++ -x c++ -std=c++11 -fsyntax-only -Iinclude writer.cpp &&
        clang++ -x c++ -std=c++11 -fsyntax-only -Iinclude prettywriter.cpp &&
        clang++ -x c++ -std=c++11 -fsyntax-only -Iinclude pointer.cpp &&
        clang++ -x c++ -std=c++11 -fsyntax-only -Iinclude schema.cpp
    " < /dev/null
```

> **说明**：`< /dev/null` 使 init 在非交互模式下自动全选所有中间件文件。  
> 交互终端下，`init` 会弹出复选界面（`Space` 勾选，`Enter` 确认）。

**预期输出**（节选）：
```
[init] Hook compiled: .cpp2rust/default/hook/libhook.so
[init] Running build command via LD_PRELOAD...
[init] Captured 6 translation unit(s)
[init] Selected 6 middleware file(s)
[init] AST dump: document.cpp.cpp2rust ... OK
[init] AST dump: reader.cpp.cpp2rust   ... OK
...
[init] Generated: .cpp2rust/default/rust/src/mod_document/
[init] Generated: .cpp2rust/default/rust/src/mod_reader/
...
[init] Interface report: .cpp2rust/default/meta/init-interface-report.md
```

### 步骤 5：运行 `cpp2rust-demo merge`

```bash
cd /tmp/rapidjson
"${CPP2RUST}" merge --feature "${FEATURE}"
```

**预期输出**（节选）：
```
[merge] Reading 6 group(s) from .cpp2rust/default/rust/src/
[merge] Writing merged output to .cpp2rust/default/rust/src.2/
[merge] Symlink: .cpp2rust/default/rust/src -> src.2
[merge] Merge report: .cpp2rust/default/meta/merge-report.md
```

### 步骤 6：验证产物

#### 6.1 目录结构验证

```bash
cd /tmp/rapidjson

# src 应为指向 src.2 的符号链接
ls -la .cpp2rust/default/rust/src
# 预期：.cpp2rust/default/rust/src -> src.2

# init 原始产物备份
ls .cpp2rust/default/rust/src.1/
# 预期包含：mod_document/ mod_reader/ mod_writer/ ... common/ lib.rs

# merge 产物
ls .cpp2rust/default/rust/src.2/
# 预期：lib.rs  mod_document.rs  mod_reader.rs  mod_writer.rs
#         mod_prettywriter.rs  mod_pointer.rs  mod_schema.rs  merged_ffi.rs
```

#### 6.2 关键文件存在性检查

```bash
OUT=".cpp2rust/default"

# 验证中间件文件
for case in document reader writer prettywriter pointer schema; do
    test -f "${OUT}/cpp/${case}.cpp.cpp2rust"       && echo "[OK] ${case}.cpp2rust"
    test -f "${OUT}/cpp/${case}.cpp.cpp2rust.opts"  && echo "[OK] ${case}.cpp2rust.opts"
done

# 验证 init 产物（src.1 备份）
for case in document reader writer prettywriter pointer schema; do
    test -f "${OUT}/rust/src.1/mod_${case}/include/mod.rs"     && echo "[OK] mod_${case}/include"
    test -f "${OUT}/rust/src.1/mod_${case}/free/fn_${case}.rs" && echo "[OK] mod_${case}/free"
    test -f "${OUT}/rust/src.1/mod_${case}/meta.json"           && echo "[OK] mod_${case}/meta.json"
done

# 验证 merge 产物
test -f "${OUT}/rust/src/merged_ffi.rs"   && echo "[OK] merged_ffi.rs"
test -f "${OUT}/meta/init-interface-report.md" && echo "[OK] interface report"
test -f "${OUT}/meta/merge-report.md"          && echo "[OK] merge report"
```

#### 6.3 内容正确性验证

```bash
OUT=".cpp2rust/default"

# merged_ffi.rs 应包含 import_lib! 和 link_name
grep -q "import_lib!"         "${OUT}/rust/src/merged_ffi.rs" && echo "[OK] import_lib!"
grep -q 'link_name = "rapidjson"' "${OUT}/rust/src/merged_ffi.rs" && echo "[OK] link_name"

# merged_ffi.rs 应包含六个中间件的 include
for case in document reader writer prettywriter pointer schema; do
    grep -q "#include \"${case}.cpp.cpp2rust\"" "${OUT}/rust/src/merged_ffi.rs" \
        && echo "[OK] include ${case}"
done

# 接口报告应包含 Type Aliases 和模板别名
grep -q "Type Aliases" "${OUT}/meta/init-interface-report.md" && echo "[OK] Type Aliases section"
grep -q "Document"     "${OUT}/meta/init-interface-report.md" && echo "[OK] Document alias"
grep -q "Value"        "${OUT}/meta/init-interface-report.md" && echo "[OK] Value alias"

# 验证各 group 的 import_lib! 包含 link_name
for case in document reader writer prettywriter pointer schema; do
    grep -q 'link_name = "rapidjson"' "${OUT}/rust/src.1/mod_${case}/free/fn_${case}.rs" \
        && echo "[OK] mod_${case} link_name"
done
```

### 步骤 7：使用自动化脚本（一键复现）

上述步骤的核心流程（init → merge → 验证）已集成到 `scripts/validate-rapidjson.sh`，可直接执行：

```bash
# 在仓库根目录执行（debug 构建）
./scripts/validate-rapidjson.sh

# 或使用 release 构建（与 CI 一致）
./scripts/validate-rapidjson.sh --release
```

脚本会自动完成构建 → 克隆 rapidjson → init → merge → 验证全流程，最终输出：
```
✓ All validation checks passed.
```

> **与手动步骤的差异**：脚本的 `init` 命令**不传 `--no-link`**（即会向 `build.rs` 注入 `cargo::rustc-link-lib=rapidjson`）。这是 CI 有意为之：脚本只验证接口提取和绑定生成的正确性，不实际执行 `cargo build`，因此是否注入链接指令不影响结果。若你需要将生成的 crate 用于 header-only 场景（不链接预编译库），应在自己的 entry.cpp 流程中加上 `--no-link`，如手动步骤 4 所示。

---

## 四、预期产物示例

### `merged_ffi.rs`（节选）

```rust
// merged_ffi.rs — auto-generated by cpp2rust-demo merge

// ===== Common includes =====
hicc::cpp! {
    #include "document.cpp.cpp2rust"
    #include "reader.cpp.cpp2rust"
    #include "writer.cpp.cpp2rust"
    #include "prettywriter.cpp.cpp2rust"
    #include "pointer.cpp.cpp2rust"
    #include "schema.cpp.cpp2rust"
}

// ===== mod_document =====
hicc::import_lib! {
    #![link_name = "rapidjson"]
    // ... 自由函数
}

hicc::import_class! {
    #[cpp(class = "rapidjson::GenericValue<rapidjson::UTF8<char>, ...>",
          ctor = "Value()")]
    class Value {
        // ... Value 方法
    }
}

hicc::import_class! {
    #[cpp(class = "rapidjson::GenericDocument<rapidjson::UTF8<char>, ...>",
          ctor = "Document()")]
    class Document: Value {
        // ... Document 方法
    }
}

// ===== mod_schema =====
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParseErrorCode {
    kParseErrorNone = 0,
    kParseErrorDocumentEmpty = 1,
    // ...
}
```

### 接口报告（`init-interface-report.md`，节选）

```markdown
## Type Aliases

| C++ alias | Underlying C++ type | Rust type |
|-----------|---------------------|-----------|
| `Value`   | `rapidjson::GenericValue<rapidjson::UTF8<char>, ...>` | `Value` |
| `Document`| `rapidjson::GenericDocument<rapidjson::UTF8<char>, ...>` | `Document` |
| `Writer`  | `rapidjson::GenericWriter<rapidjson::StringBuffer, ...>` | `Writer` |
...

## Skipped declarations

| Name | Reason | Category |
|------|--------|----------|
| `~GenericDocument` | destructor | hicc_limitation |
| `operator[]` | operator_overload | hicc_limitation |
...
```

---

## 五、不支持特性说明

### 5.1 operator 重载（半支持，需用户参与）

**现状**：`operator[]`、`operator=`、`operator bool` 等被跳过提取，
但工具自动生成 `operator_shims.hpp` starter 和 `shim_ops.rs` 骨架。

**使用方式**：
1. 查看 `meta/operator_shims.hpp`（自动生成的具名 C++ 包装函数）
2. 在 `build.rs` 中添加 shim 目录的 include path
3. 在 `hicc::cpp!` 块中引入 `operator_shims.hpp`
4. `shim_ops.rs` 中的 `import_lib!` 绑定即可激活

详细工作流见 `examples/rapidjson/07-operator-shim/README.md`。

### 5.2 析构函数（跳过，hicc 限制）

`~GenericDocument()` 等析构函数统一跳过，标记 `hicc_limitation`。
对象生命周期管理由 C++ 侧负责，Rust 侧通过 hicc 自动处理释放。

### 5.3 `std::string` / `std::basic_ostream` 参数

含 `std::string` 返回或参数的方法被跳过，标记 `hicc_limitation`。
建议手写 C++ shim，将 `std::string` 转换为 `const char*`。

**示例 shim**：
```cpp
// shim: wrap GenericValue::GetString() return
static inline const char* value_get_string(const rapidjson::Value& val) {
    return val.GetString();
}
```

### 5.4 多重继承（仅取首个 public 基类）

RapidJSON 的 `SchemaValidator` 等少数类使用多重继承，当前工具只提取首个 public 基类。
其余基类在接口报告中可见，待 `future-plan.md §2` 实现后解锁。

### 5.5 高度模板化的内部类型（partial support）

部分内部类型（如 `GenericSchemaDocument`）的完整特化参数过于复杂，
可能出现 `tool_conservative` 跳过。
解决方案：在 entry.cpp 中添加更简短的 `using` 别名，触发 AliasRegistry 注册。

---

## 六、CI 对应关系

本文档的验证流程与项目 CI 完全对应：

| 本地步骤 | CI 对应位置 |
|---------|-----------|
| `cargo build --release` | `.github/workflows/validate-rapidjson.yml` step "Build" |
| `git clone rapidjson` | `.github/workflows/validate-rapidjson.yml` step "Prepare" |
| `cpp2rust-demo init` (六个 TU) | `scripts/validate-rapidjson.sh` Step 3 |
| `cpp2rust-demo merge` | `scripts/validate-rapidjson.sh` Step 4 |
| 文件存在性 + 内容检查 | `scripts/validate-rapidjson.sh` Step 5 |
| CI artifact 上传 | `.github/workflows/validate-rapidjson.yml` step "Upload artifact" |

在 GitHub Actions 上，每次 push 或 PR 都会自动运行此验证，
产物上传为 CI artifact `rapidjson-cpp2rust-output`（可在 Actions 页面下载查看）。

---

## 七、完整的自包含示例索引

项目 `examples/` 目录下包含 8 个基于 RapidJSON 等价自包含类型的示例，
**无需安装 RapidJSON** 即可完整运行：

| 示例目录 | 演示特性 | 对应 RapidJSON 组件 |
|---------|---------|-------------------|
| `rapidjson/01-enum/` | `enum` / `enum class` | `ParseErrorCode`, `Type` |
| `rapidjson/02-typedef-alias/` | `typedef`/`using` + AliasRegistry | `Document`, `Value` 别名机制 |
| `rapidjson/03-template-class/` | 模板特化类提取 | `GenericDocument`, `GenericValue` |
| `rapidjson/04-abstract-interface/` | 全纯虚类 + `@make_proxy` | 自定义 Allocator 接口 |
| `rapidjson/05-virtual-methods/` | 非纯虚方法 | `CrtAllocator` 类 |
| `rapidjson/06-inheritance/` | public 继承链 | `PrettyWriter: Writer` |
| `rapidjson/07-operator-shim/` | 运算符重载 shim | `GenericValue::operator[]` 等 |
| `rapidjson/08-multi-tu/` | 多翻译单元 + `--no-link` + `merge` | 完整 RapidJSON 多头文件场景 |

运行任意示例（以 `rapidjson/01-enum` 为例）：

```bash
# 在仓库根目录
cargo build --release
BINARY=./target/release/cpp2rust-demo

# --link rapidjson : 设置生成代码中 import_lib! 的 link_name（逻辑库名）
# --no-link        : 不向 build.rs 注入 cargo::rustc-link-lib（header-only 无需实际链接）
# 两个标志不矛盾：--link 决定"叫什么名字"，--no-link 决定"是否要链接"
${BINARY} init --feature rj01 --link rapidjson --no-link \
    -- clang -x c++ -fsyntax-only examples/rapidjson/01-enum/entry.cpp < /dev/null

${BINARY} merge --feature rj01

cat .cpp2rust/rj01/rust/src/merged_ffi.rs
```
