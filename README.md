# cpp2rust-demo

**C++ → Rust Safe FFI 自动化脚手架生成工具**。给定任意 C++ 项目，执行一条命令即可生成基于 [hicc](https://crates.io/crates/hicc) 的 Rust FFI 绑定层（`hicc::cpp!` / `hicc::import_class!` / `hicc::import_lib!` 三段式）。

仓库同时包含 **48 个循序渐进的 C++ 特性示例**，每个示例都有对应的 C++ 源码和可运行的 Rust FFI 参考实现，覆盖从基础函数到模板、STL、虚继承等复杂场景。

---

## 目录结构

```
cpp2rust-demo/
├── hook/              # LD_PRELOAD 拦截器（hook.cpp + Makefile）
├── src/               # 工具源码（Rust）
├── examples/          # 48 个示例，每个含 cpp/ 和 rust_hicc/ 子目录
├── tests/             # 三层测试体系（L1/L2/L3）
├── docs/
│   ├── plans/v5/      # 完整方案文档（automated-cpp2rust-ffi-v5.md）
│   └── references/    # hicc、c2rust-demo 等参考文档
└── references/
    └── c2rust-demo/   # C 语言版参考实现（同架构）
```

---

## 工作原理

工具通过 **三阶段流水线** 将 C++ 代码转换为 Rust FFI 脚手架：

```
┌──────────────────────────────────┐
│  阶段 1：编译拦截（hook.cpp）      │
│  LD_PRELOAD 注入 → g++ -E -C     │
│  → .cpp2rust（宏展开后的 C++ 代码）│
└──────────────┬───────────────────┘
               ↓
┌──────────────────────────────────┐
│  阶段 2：AST 提取                 │
│  clang crate 解析 .cpp2rust      │
│  → CppAst（类/函数/枚举/模板）    │
│  → FfiSpec IR（extractor/）      │
└──────────────┬───────────────────┘
               ↓
┌──────────────────────────────────┐
│  阶段 3：代码生成（generator/）   │
│  FfiSpec → hicc 三段式 Rust 代码 │
│  → .cpp2rust/<feature>/rust/     │
│     lib.rs + <unit>.rs           │
└──────────────────────────────────┘
```

**关键理念**：C++ 模板的价值在于**实例化结果**，不在于模板声明。工具只处理实际被实例化的具体类型（如 `std::vector<int>`），跳过未实例化的模板。

---

## 快速开始

### 安装依赖

```bash
# 系统依赖（Ubuntu/Debian）
sudo apt-get install clang libclang-dev g++ libstdc++-14-dev

# 从 GitHub 安装（无需克隆仓库）
cargo install --git https://github.com/LuuuXXX/cpp2rust-demo

# 或从本地源码安装（开发者）
cargo install --path .
```

> **注意**：`hook/hook.cpp` 已内嵌进 binary，无需额外文件。首次执行 `init` 时工具
> 自动将 hook 源码解压到 `~/.local/share/cpp2rust-demo/hook/`（Linux）或
> `~/Library/Application Support/cpp2rust-demo/hook/`（macOS）并编译；后续调用在 hook 库为最新版时自动跳过重编译。

### Step 1 — `init`：捕获构建 + 生成 FFI 脚手架

在目标 C++ 项目根目录执行：

```bash
cd /path/to/my-cpp-project

# 单文件项目
cpp2rust-demo init -- g++ -shared -fPIC mylib.cpp -o libmylib.so

# Make 项目
cpp2rust-demo init -- make -j4

# 指定 feature 名称以区分不同构建目标（如不同平台或构建配置）
# C++ 的 target 对应 Rust 的 feature：每个 feature 保存一次构建命令的产物
cpp2rust-demo init --feature linux_x86 -- make -j4
cpp2rust-demo init --feature arm_embedded -- arm-none-eabi-g++ -shared -fPIC mylib.cpp -o libmylib.so
```

> **多 target 场景**：同一 C++ 项目针对多个平台或构建配置，分别执行 `init` 并指定不同 `--feature`，
> 输出会落在各自独立的目录下，互不干扰：
>
> ```
> .cpp2rust/linux_x86/      ← linux_x86 构建命令对应的 Rust FFI 绑定
> .cpp2rust/arm_embedded/   ← arm_embedded 构建命令对应的 Rust FFI 绑定
> ```
>
> 各 target `init` 完成后，可用 `merge` 的多 feature 模式将它们聚合为一个统一 crate（详见 Step 2b）。


`init` 自动完成：
1. 首次运行时将内嵌的 `hook.cpp` 解压到用户数据目录并编译为 `libhook.so`（后续调用自动跳过）
2. 通过 LD_PRELOAD 注入构建过程，捕获 `.cpp2rust` 预处理文件
3. 交互式选择参与转换的文件（非交互环境自动全选）
4. libclang 解析 AST，提取类/函数/枚举/模板实例化
5. 生成 `.cpp2rust/<feature>/rust/` 下的 hicc Rust 脚手架

输出示例：
```
=== cpp2rust-demo init ===
Project root : /path/to/my-cpp-project
Feature      : default
...
  mylib.cpp.cpp2rust → 2 class(es), 5 fn(s), 0 enum(s)  [142 ms]

⚠ Degraded features (require manual attention):
  [OP] × 2
  → Search for 'cpp2rust-todo' in generated files to find these locations.

✓ cpp2rust-demo init completed.

Output structure:
  .cpp2rust/default/
    ├── c/          (captured .cpp2rust files)
    ├── meta/       (build_cmd.txt, selected_files.json)
    └── rust/       (generated Rust project: Cargo.toml, src/lib.rs, src/**/*.rs)
```

### Step 2 — `merge`：备份并整理编译单元输出（可选）

`merge` 将 `init` 生成的 `src/` 目录原地备份，并将整理后的输出写回同一 feature 目录，完整保留 C++ 项目的目录结构：

```bash
cpp2rust-demo merge                        # 默认 feature="default"
cpp2rust-demo merge --feature linux_x86   # 指定单个 feature
```

执行后在 `.cpp2rust/<feature>/rust/` 下生成：

```
.cpp2rust/default/rust/
    ├── src.1/   ← init 输出的原始备份（rename from src）
    ├── src.2/   ← merge 输出（维持 C++ 目录结构）
    └── src      ← symlink → src.2
```

- 首次运行：`src/` 重命名为 `src.1/`，输出写入 `src.2/`，建立 `src → src.2` symlink
- 重复运行：`src.1/` 保持不变，仅更新 `src.2/` 并重建 symlink

### Step 2b — `merge`（多 feature 模式）：跨 feature 聚合（可选）

`--feature` 可重复传入多次。指定多个 feature 时，工具将各 feature 的编译单元**跨 feature 聚合**（去重 + 冲突检测），输出到以各 feature 名**下划线拼接**命名的新目录，source feature 目录保持不变：

```bash
# 将 linux_x86 和 arm_embedded 两个 feature 合并
cpp2rust-demo merge --feature linux_x86 --feature arm_embedded
# 输出到 .cpp2rust/linux_x86_arm_embedded/
```

执行后生成：

```
.cpp2rust/linux_x86_arm_embedded/
    ├── meta/
    │   └── merge-report.md   ← 列出来源 feature、合并单元数、冲突（如有）
    └── rust/
        ├── Cargo.toml         ← package.name = "linux_x86_arm_embedded"
        ├── build.rs
        └── src/
            ├── lib.rs
            └── ffi.rs         ← 所有 feature 的合并 FFI 代码（include 去重、方法去重）
```

合并逻辑：
- **include 去重**：相同 `#include` 行只保留一次
- **class 方法去重**：相同 class 的相同方法签名只保留一次；不同签名的同名方法记为冲突
- **函数绑定去重**：相同 `#[cpp(func = "...")]` 绑定去重；签名不一致时记为冲突
- **冲突报告**：冲突会打印在终端，并详细记录于 `merge-report.md`；工具不会因冲突中止，保留第一次出现的版本

### Step 3 — 手动完善降级特性

搜索生成代码中的 `cpp2rust-todo` 注释，按 TAG 说明手动完善：

| TAG | 原因 | 需手动操作 |
|-----|------|-----------|
| `[OP]` | 运算符重载（C ABI 无运算符符号） | 为生成的命名 shim（`{class}_add` 等）添加 Rust `std::ops::*` trait 实现 |
| `[VA]` | 可变参数模板（编译期展开，FFI 无法表达任意参数数） | 检查 wrapper 类展开的版本数量是否满足需求，按需手动添加新版本 |
| `[LM]` | 有状态 Lambda / std::function（捕获列表不透明） | 若需 Rust 闭包 → C++ 回调，手动编写 trampoline |

### CLI 参数参考

#### `cpp2rust-demo init`

| 参数 | 必填 | 说明 |
|------|------|------|
| `-- <BUILD_CMD>...` | ✅ | `--` 后面的所有参数作为构建命令传入 |
| `--feature <name>` | ❌ | 构建目标名称（对应 Rust feature）；不同平台或构建配置使用不同名称，构建命令的差异即是 target 的差异（默认 `default`） |

#### `cpp2rust-demo merge`

| 参数 | 必填 | 说明 |
|------|------|------|
| `--feature <name>` | ❌ | 要操作的 feature 名称（可重复）。不传时默认 `default`（单 feature 模式）；传入多次时启用**跨 feature 合并**（多 feature 模式），输出目录名为各 feature 名下划线拼接 |

#### 环境变量

| 变量 | 说明 |
|------|------|
| `CPP2RUST_CXX` | 覆盖默认 C++ 编译器（默认自动检测 g++/clang++/c++，支持带版本后缀如 g++-13） |
| `CPP2RUST_DEBUG` | 非空时输出 hook 调试日志到 stderr |

---

## 生成代码格式（三段式）

工具输出标准的 hicc 三段式 Rust FFI 代码：

```rust
// ─── 段 1：C++ 实现内联（含必要 shim）───────────────────
hicc::cpp! {
    #include "foo.h"

    // ctor/dtor/operator/placement-new 等必要 shim
    Foo* foo_new(int value) { return new Foo(value); }
    void foo_delete(Foo* self) { delete self; }
}

// ─── 段 2：类方法绑定（每个类独立块）──────────────────────
hicc::import_class! {
    #[cpp(class = "Foo")]
    class Foo {
        #[cpp(method = "int getValue() const")]
        fn getValue(&self) -> i32;

        #[cpp(method = "void setValue(int v)")]
        fn setValue(&mut self, v: i32);
    }
}

// ─── 段 3：全局/关联函数绑定 ──────────────────────────────
hicc::import_lib! {
    #![link_name = "foo"]

    class Foo;

    #[cpp(func = "Foo* foo_new(int value)")]
    fn foo_new(value: i32) -> *mut Foo;

    #[cpp(func = "void foo_delete(Foo* self)")]
    unsafe fn foo_delete(self_: *mut Foo);
}
```

**最小 shim 策略**：成员方法直接通过 `import_class!` 绑定（由 hicc 处理虚表 dispatch），只有以下场景才生成 C shim 函数：
- 构造函数 / 析构函数（C 无 `new`/`delete`）
- 静态成员变量 getter/setter
- 运算符重载（C ABI 无运算符符号）
- placement new
- STL 容器 wrapper 类的 ctor/dtor

---

## C++ 特性支持矩阵

> 图例：✅ 完全自动生成可编译代码　⚠️ 降级生成 + 内联 TODO（代码仍可 `cargo check`）

### 基础类型与函数（001–005）

| 示例 | C++ 特性 | 状态 | FFI 策略 |
|------|---------|------|---------|
| [001_hello_world](examples/001_hello_world) | extern "C" 函数 | ✅ | AST 直接提取 → `import_lib!` |
| [002_function_overload](examples/002_function_overload) | 函数重载 | ✅ | 各重载名称加类型后缀区分（`_i32`/`_f64`）→ `import_lib!` |
| [003_default_args](examples/003_default_args) | 默认参数 | ✅ | C++ 侧展开为多个固定参数重载，写入 `hicc::cpp!` |
| [004_inline_functions](examples/004_inline_functions) | inline 函数 | ✅ | 函数体从 `.cpp2rust` 提取，内联写入 `hicc::cpp!` |
| [005_variadic_functions](examples/005_variadic_functions) | C 可变参数（va_list） | ✅ | 检测已有固定参数 wrapper，直接 `extern "C"` 绑定 |

### 类与对象（006–012）

| 示例 | C++ 特性 | 状态 | FFI 策略 |
|------|---------|------|---------|
| [006_class_basic](examples/006_class_basic) | 基础类 | ✅ | opaque pointer + `import_class!` + ctor/dtor shim |
| [007_class_constructor](examples/007_class_constructor) | 构造/析构 | ✅ | `*_new()` / `*_delete()` 必要 shim |
| [008_class_copy](examples/008_class_copy) | 拷贝构造 | ✅ | `*_copy(const Foo*)` 必要 shim |
| [009_class_move](examples/009_class_move) | 移动构造 | ✅ | `*_move(Foo*)` shim（内部 `std::move()`） |
| [010_class_static](examples/010_class_static) | 静态成员 | ✅ | `{class}_get_{field}` / `{class}_set_{field}` 必要 shim |
| [011_class_const](examples/011_class_const) | const 成员函数 | ✅ | 直接 `import_class!`，映射为 `fn method(&self)` |
| [012_class_volatile](examples/012_class_volatile) | volatile 成员函数 | ✅ | 直接 `import_class!`，Rust 侧使用 `*mut T` 并加注释 |

### 面向对象（013–018）

| 示例 | C++ 特性 | 状态 | FFI 策略 |
|------|---------|------|---------|
| [013_inheritance_single](examples/013_inheritance_single) | 单继承 | ✅ | 基类方法在子类 `import_class!` 中一并提升，无 shim |
| [014_inheritance_multiple](examples/014_inheritance_multiple) | 多继承 | ✅ | 多条继承链展开，所有方法通过 `import_class!` 直接绑定 |
| [015_virtual_basic](examples/015_virtual_basic) | 虚函数 | ✅ | opaque pointer 调用，hicc 宏负责虚表 dispatch |
| [016_virtual_pure](examples/016_virtual_pure) | 纯虚/抽象类 | ✅ | 抽象类只生成前向声明；子类通过 `import_class!` 绑定 |
| [017_virtual_override](examples/017_virtual_override) | override | ✅ | override 语义透传，与普通虚函数相同 |
| [018_virtual_diamond](examples/018_virtual_diamond) | 菱形继承（virtual 继承） | ✅ | 为每个需要从 Rust 侧调用的继承方法生成命名 shim（`d_getAValue(D*)`），避免指针调整 |

### 运算符与类型（019–023）

| 示例 | C++ 特性 | 状态 | FFI 策略 |
|------|---------|------|---------|
| [019_operator_overload](examples/019_operator_overload) | 运算符重载 | ⚠️ `[OP]` | 生成命名 shim（`{class}_add` 等）+ `cpp2rust-todo[OP]`，Rust `ops::*` trait 需手动实现 |
| [020_friend_function](examples/020_friend_function) | 友元函数 | ✅ | 友元函数提取为普通函数写入 `import_lib!` |
| [021_explicit_ctor](examples/021_explicit_ctor) | explicit 构造函数 | ✅ | `explicit` 对 FFI 透明，与普通构造相同 |
| [022_mutable_member](examples/022_mutable_member) | mutable 成员 | ✅ | `mutable` 对 FFI 透明，直接 `import_class!` |
| [023_typeid_rtti](examples/023_typeid_rtti) | typeid/RTTI/dynamic_cast | ✅ | 注入整数枚举 + 虚函数 `getType()`，完全绕过 `typeid` |

### 模板实例化（024–028）

| 示例 | C++ 特性 | 状态 | FFI 策略 |
|------|---------|------|---------|
| [024_template_function](examples/024_template_function) | 函数模板 | ✅ | 忽略模板声明，为每个实例化版本生成命名 C 包装函数 |
| [025_template_class](examples/025_template_class) | 类模板 | ✅ | 只处理实际实例化的具体类型，按普通类生成 |
| [026_template_specialization](examples/026_template_specialization) | 模板偏特化 | ✅ | 偏特化视为实例化路径之一，收集通过该路径实例化的类型 |
| [027_template_instantiation](examples/027_template_instantiation) | 显式模板实例化 | ✅ | 显式实例化在 AST 中直接可见，按普通类处理 |
| [028_variadic_template](examples/028_variadic_template) | 可变参数模板 | ⚠️ `[VA]` | 生成 wrapper 类 + 按参数数量展开的静态方法；超出范围的组合需手动添加 |

### 智能指针与内存（029–033）

| 示例 | C++ 特性 | 状态 | FFI 策略 |
|------|---------|------|---------|
| [029_unique_ptr](examples/029_unique_ptr) | std::unique_ptr | ✅ | opaque pointer；`*_new()` 返回裸指针，调用方 `*_delete()` 释放 |
| [030_shared_ptr](examples/030_shared_ptr) | std::shared_ptr | ✅ | `*_clone()` shim 增加引用计数，`*_delete()` 减少；其余方法直接绑定 |
| [031_custom_deleter](examples/031_custom_deleter) | 自定义删除器 | ✅ | 删除器函数注入 `hicc::cpp!`，`*_delete()` shim 内部调用自定义删除器 |
| [032_placement_new](examples/032_placement_new) | placement new | ✅ | 生成 `*_placement_new(ptr, ...)` 必要 shim |
| [033_raii_pattern](examples/033_raii_pattern) | RAII 模式 | ✅ | 析构函数生成 `*_delete()` shim，Rust 侧可实现 `Drop` trait |

### STL 容器（034–038）

> **核心策略**：STL 容器模板方法签名复杂，先在 `hicc::cpp!` 中生成**薄 wrapper 类**（如 `IntVector` 封装 `std::vector<int>`），再对 wrapper 类做 `import_class!` 绑定。

| 示例 | C++ 特性 | 状态 | wrapper 类 |
|------|---------|------|-----------|
| [034_vector_basic](examples/034_vector_basic) | std::vector\<T\> | ✅ | `IntVector`（`VectorImpl<int>` 封装）|
| [035_map_basic](examples/035_map_basic) | std::map\<K,V\> | ✅ | `StringIntMap`（`MapImpl<string,int>` 封装）|
| [036_string_basic](examples/036_string_basic) | std::string | ✅ | string wrapper，`c_str()`/`length()` 等通过 `import_class!` 绑定 |
| [037_array_basic](examples/037_array_basic) | std::array\<T,N\> | ✅ | 数组 wrapper（N 在实例化时已知） |
| [038_tuple_basic](examples/038_tuple_basic) | std::tuple\<T...\> | ✅ | tuple wrapper，按位置 `get<N>()` 通过 `import_class!` 绑定 |

### 函数对象（039–042）

| 示例 | C++ 特性 | 状态 | FFI 策略 |
|------|---------|------|---------|
| [039_lambda_basic](examples/039_lambda_basic) | Lambda 表达式 | ⚠️ `[LM]` | 无状态 lambda → 函数指针；有状态 lambda → class wrapper（`LambdaWrapper`）+ `call()` 方法 |
| [040_std_function](examples/040_std_function) | std::function\<Sig\> | ⚠️ `[LM]` | 类型擦除容器，统一用 class wrapper + opaque pointer；`call()` 方法通过 `import_class!` 绑定 |
| [041_functional_bind](examples/041_functional_bind) | std::bind | ✅ | 产物本质是函数对象，同有状态 lambda 策略，`import_class!` 完全覆盖 |
| [042_exception_basic](examples/042_exception_basic) | C++ 异常处理 | ✅ | shim 层 `try/catch` 捕获异常，转为错误码 + 错误消息字符串返回；FFI 边界不跨越异常 |

### 其他高级特性（043–048）

| 示例 | C++ 特性 | 状态 | FFI 策略 |
|------|---------|------|---------|
| [043_namespace_nested](examples/043_namespace_nested) | 嵌套命名空间 | ✅ | 命名空间前缀扁平化到函数名（`foo::bar::Baz` → `foo_bar_baz`） |
| [044_enum_class](examples/044_enum_class) | 强类型枚举（enum class） | ✅ | 枚举值导出为 Rust `const`，建议手动实现 `enum` + `TryFrom<i32>` |
| [045_union_basic](examples/045_union_basic) | union | ✅ | opaque pointer + 按字段名 getter/setter shim；Rust 侧用 `#[repr(C)] union` |
| [046_constexpr_basic](examples/046_constexpr_basic) | constexpr 常量/函数 | ✅ | 编译期常量读取 AST `IntegerLiteral` 值，生成 Rust `const`；constexpr 函数按普通函数处理 |
| [047_noexcept_basic](examples/047_noexcept_basic) | noexcept | ✅ | `noexcept` 语义对 FFI 透明，直接处理 |
| [048_summary](examples/048_summary) | 综合 FFI 模式 | ✅ | 以上所有策略的组合应用 |

---

## 降级特性详解（4 项）

| TAG | 示例 | C++ 特性 | 无法完全自动的根本原因 | 自动降级策略 | 用户剩余工作 |
|-----|------|---------|---------------------|------------|------------|
| `[OP]` | 019 | 运算符重载 | C ABI 无运算符符号；FFI 边界只能传命名函数 | 为每个运算符生成命名 shim（`{class}_add/sub/...`），写入 `hicc::cpp!` + `import_lib!` | 可选：手动实现 `impl std::ops::Add<T> for T` 等 |
| `[VA]` | 028 | 可变参数模板 | `...Args` 是编译期展开，FFI 无法表达"任意数量参数" | 生成 wrapper 类，按参数数量和类型组合分别封装为静态方法（`sum_1`/`sum_2` 等） | 若需要新的参数数量/类型组合，在 `hicc::cpp!` 中手动添加对应方法和包装函数 |
| `[LM]` | 039 | 有状态 Lambda | 匿名闭包类型，FFI 无法表达捕获列表 | 无状态 lambda → 函数指针；有状态 lambda → class wrapper + opaque pointer | 若需 Rust 闭包 → C++ 回调，手动编写 trampoline |
| `[LM]` | 040 | std::function | 类型擦除容器，捕获状态不透明 | 统一使用 class wrapper + opaque pointer | 可选：手动实现 Rust 闭包 → `std::function` 适配层 |

---

## 测试体系

测试分五层，位于 `tests/` 目录：

| 层 | 文件 | 验证内容 | 当前状态 |
|----|------|---------|---------|
| **L1** 黄金文件测试 | `l1_golden_tests.rs` | 工具生成的 hicc 脚手架与 `rust_hicc/src/main.rs` 中对应块一致 | ✅ **49/49 通过** |
| **L2** 编译测试 | `l2_compile_tests.rs` | 仓库中现有的 `rust_hicc/` 能通过 `cargo build` | ✅ **48/48 通过** |
| **L3** 运行测试 | `l3_run_tests.rs` | `cargo run` 输出与各示例 README 中"运行结果"一致 | ✅ **48/48 通过** |
| **L4** E2E 测试 | `rapidjson_e2e_test.rs` | 对 rapidjson 开源项目执行完整 init + merge 转换，验证 hicc 三段式格式 | ✅ 通过 |
| **L5** 符号验证测试 | `l5_nm_symbol_tests.rs` | 用 `nm` 双向验证 C++ 导出符号均已链接进 Rust FFI 二进制 | ✅ 通过 |

### 测试命令

```bash
# 运行 L1 黄金文件测试（须单线程：clang 全局状态竞争）
cargo test --test l1_golden_tests -- --include-ignored --test-threads=1

# 运行 L2 编译测试
cargo test --test l2_compile_tests

# 运行 L3 运行测试
cargo test --test l3_run_tests -- --include-ignored --test-threads=1

# 运行 L4 rapidjson E2E 测试
cargo test --test rapidjson_e2e_test -- --include-ignored

# 运行 L5 nm 符号验证测试
cargo test --test l5_nm_symbol_tests -- --include-ignored

# 运行某个示例的全部测试
cargo test 006_class_basic

# 更新黄金文件（工具输出有意变更时使用）
cargo test --test l1_golden_tests update_all_goldens -- --include-ignored
```

---

## 局限性

| 场景 | 说明 |
|------|------|
| **仅支持 Linux** | 依赖 LD_PRELOAD 机制，Windows 暂不支持 |
| **命名空间类** | extern "C" 函数签名中含 `::` 类型或 `void*` opaque 指针时，会压制 `import_class!`/`import_lib!` 块（仅生成空 `cpp!`），需手动绑定 |
| **运算符重载** | 生成命名 shim + `[OP]` TODO，Rust 运算符 trait 需手动实现 |
| **有状态 Lambda / std::function** | 生成 class wrapper，若需 Rust 闭包回调需手动编写 trampoline |
| **可变参数模板** | 按调用点展开有限版本，超出范围的参数组合需手动添加 |
| **业务逻辑** | 工具只生成 FFI 绑定层（`lib.rs`），`fn main()` 和业务代码需手动编写 |
| **跨翻译单元模板** | 每个 `.cpp2rust` 独立解析，跨文件模板实例化可能遗漏（`merge` 阶段部分缓解） |

---

## 学习路径（示例索引）

每个示例目录下包含：
- `cpp/`：C++ 源码（工具的输入）
- `rust_hicc/src/main.rs`：包含 hicc FFI 脚手架 + 手写 `fn main()` 的完整可运行参考文件
- `README.md`：特性说明 + 运行结果

```
入门阶段：001_hello_world → 002_function_overload → 003_default_args
         → 004_inline_functions → 005_variadic_functions → 006_class_basic

类与对象：007_class_constructor → 008_class_copy → 009_class_move
         → 010_class_static → 011_class_const → 012_class_volatile

面向对象：013_inheritance_single → 014_inheritance_multiple
         → 015_virtual_basic → 016_virtual_pure → 017_virtual_override → 018_virtual_diamond

运算符/类型：019_operator_overload → 020_friend_function → 021_explicit_ctor
            → 022_mutable_member → 023_typeid_rtti

模板：024_template_function → 025_template_class → 026_template_specialization
     → 027_template_instantiation → 028_variadic_template

内存管理：029_unique_ptr → 030_shared_ptr → 031_custom_deleter
         → 032_placement_new → 033_raii_pattern

STL：034_vector_basic → 035_map_basic → 036_string_basic
    → 037_array_basic → 038_tuple_basic

函数对象：039_lambda_basic → 040_std_function → 041_functional_bind → 042_exception_basic

高级特性：043_namespace_nested → 044_enum_class → 045_union_basic
         → 046_constexpr_basic → 047_noexcept_basic → 048_summary
```

---

## 运行单个示例

```bash
cd examples/001_hello_world

# 编译 C++ 共享库
cd cpp && g++ -shared -fPIC hello_world.cpp -o libhello_world.so && cd ..

# 编译并运行 Rust FFI
cd rust_hicc && cargo run
```

---

## 依赖

- Linux（LD_PRELOAD 必需）
- C++ 编译器：g++ 或 clang++（C++11 或更高）
- Rust 工具链：rustc / cargo（1.82+）
- libclang（用于 AST 解析）：`libclang-dev`
- [`hicc`](https://crates.io/crates/hicc) `0.2` 和 [`hicc-build`](https://crates.io/crates/hicc-build) `0.2`

---

## 许可

MIT

