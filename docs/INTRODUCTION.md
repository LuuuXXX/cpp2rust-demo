# cpp2rust-demo

> C++ → Rust Safe FFI 自动化脚手架生成工具

---

## Part 1：方案介绍

### 核心思路

给定一个任意 C++ 项目，执行一条命令，工具自动完成：

1. **编译拦截**（LD_PRELOAD hook）：捕获实际被编译的 C++ 文件及其预处理内容
2. **AST 解析**：用 libclang 解析宏展开后的代码，提取类/函数/枚举/模板实例化
3. **代码生成**：输出 hicc 宏格式的 Rust FFI 脚手架（`hicc::cpp!` / `hicc::import_class!` / `hicc::import_lib!` 三段式）

工具**不**负责生成 `fn main()` 和完整的语义等价翻译，只生成 FFI 绑定层。

### 数据流

```
C++ 项目          ① 拦截编译          ② 解析 AST          ③ 提取 IR           ④ 后处理          ⑤ 生成 Rust
                  (LD_PRELOAD)       (libclang)          (extractor)         (postprocessor)   (generator)

foo.cpp ─make─>  g++ 被劫持        .cpp2rust 文件      CppAst 结构体        FfiSpec IR         .rs 文件
bar.cpp             │                  │                    │                   │                  │
                    ▼                  ▼                    ▼                   ▼                  ▼
            ┌─────────────┐  ┌───────────────┐  ┌──────────────────┐  ┌──────────────┐  ┌──────────────┐
            │  .cpp2rust  │  │   CppAst       │  │    FfiSpec        │  │  FfiSpec     │  │ hicc 三段式   │
            │  预处理文件  │  │  结构化 AST    │  │  FFI 中间表示     │  │ (处理特殊情况)│  │ Rust FFI 代码│
            └─────────────┘  └───────────────┘  └──────────────────┘  └──────────────┘  └──────────────┘
             中间产物 ①        中间产物 ②          中间产物 ③            中间产物 ③(更新)    最终产出
```

### 中间产物说明

#### ① `.cpp2rust` 预处理文件

`g++ -E -C` 的输出——宏已展开、头文件已内联的纯 C++ 源码。

**为什么需要**：原始 `.cpp` 文件里有 `#include <vector>`、`#define MAX 100` 等预处理指令，libclang 无法直接理解宏展开后的类型。预处理一步把所有宏、条件编译、头文件全部展开，后续解析才能准确。

**怎么产生的**：`hook.cpp` 编译成 `libhook.so`，通过 LD_PRELOAD 劫持 `g++`/`clang++` 调用。正常编译不受影响，hook 额外 fork 子进程跑 `g++ -E -C -P`，将预处理结果存到 `.cpp2rust` 文件。同时记录编译选项（`.cpp2rust.opts`）和链接目标（`targets.list`）。

#### ② `CppAst` 结构化 AST

用 libclang 解析 `.cpp2rust` 文件后得到的结构化抽象语法树。

**为什么需要**：libclang 原始 AST 太底层（每个括号分号都是节点）。`CppAst` 提炼成三个维度：有哪些类、有哪些函数、有哪些枚举。

**关键过滤**：通过 `is_in_system_header()` 按源码位置过滤掉 99% 的 STL 展开内容，只保留用户代码。

#### ③ `FfiSpec` FFI 中间表示

从 `CppAst` 转换出的、面向 FFI 生成的规格说明书。**这是核心决策层**，决定了什么进 FFI、什么不进。

关键转换逻辑：

| CppAst（C++ 视角） | FfiSpec（FFI 视角） | 转换逻辑 |
|---|---|---|
| `Counter()` 构造函数 | `counter_new() -> *mut Counter` | 构造 → 工厂函数，返回 opaque pointer |
| `~Counter()` 析构函数 | `unsafe fn counter_delete(*mut Counter)` | 析构 → 销毁函数 |
| `int get() const` | `fn get(&self) -> i32` | const → `&self`，`int` → `i32` |
| `void increment()` | `fn increment(&mut self)` | 非 const → `&mut self` |
| `int value`（私有） | 不导出 | 私有成员不进入 FfiSpec |
| `static void reset()` | `fn reset()`（import_lib! 中） | 静态方法等价于全局函数 |

#### ③ 更新 后处理（Postprocessor）

对 `FfiSpec` 二次加工，处理不能直接映射到 C ABI 的特性：

| 特性 | 问题 | 策略 |
|------|------|------|
| 运算符重载 | C ABI 没有 `operator+` 符号名 | 降级为命名函数 `number_add()` |
| 有状态 Lambda | 匿名闭包，捕获列表不可见 | 无状态→函数指针；有状态→class wrapper |
| `std::function` | 类型擦除，内部捕获不透明 | class wrapper + opaque pointer |

降级处标记 `cpp2rust-todo[TAG]`，开发者可直接定位待手动完善的位置。

### 最终产出：hicc 三段式

以 `Counter` 类为例：

```rust
// 段 1：C++ 侧包装实现（shim 函数）
hicc::cpp! {
    class Counter {
        int value = 0;
    public:
        Counter() : value(0) {}
        ~Counter() {}
        int get() const { return value; }
        void increment() { value++; }
    };
    Counter* counter_new() { return new Counter(); }
    void counter_delete(Counter* self) { delete self; }
}

// 段 2：类方法的 Rust 声明
hicc::import_class! {
    #[cpp(class = "Counter")]
    pub class Counter {
        #[cpp(method = "int get() const")]
        fn get(&self) -> i32;
        #[cpp(method = "void increment()")]
        fn increment(&mut self);
    }
}

// 段 3：全局函数 + 工厂/析构函数
hicc::import_lib! {
    #![link_name = "class_basic"]
    class Counter;  // opaque pointer 前向声明
    #[cpp(func = "Counter* counter_new()")]
    fn counter_new() -> *mut Counter;
    #[cpp(func = "void counter_delete(Counter* self)")]
    unsafe fn counter_delete(self_: *mut Counter);
}
```

| 段 | 作用 |
|----|------|
| `hicc::cpp!` | C++ 侧 shim 实现（构造/析构包装、运算符→命名函数等） |
| `hicc::import_class!` | 类方法的 Rust 声明（`&self`/`&mut self`） |
| `hicc::import_lib!` | 全局函数、工厂函数、析构函数的 Rust 声明 |

Rust 侧通过 opaque pointer（`*mut Counter`）持有 C++ 对象，不需要知道内存布局，所有操作通过 C ABI 委托回 C++ 侧。

---

## Part 2：使用方法

### 一条命令

```bash
cpp2rust-demo init -- make -j4
```

背后发生的事：

1. 编译 `hook.cpp` → `libhook.so`
2. `LD_PRELOAD=libhook.so make -j4`（C++ 项目正常编译）
3. hook 劫持每次 g++ 调用 → 生成 `.cpp2rust` 预处理文件
4. 交互式选择要转换哪些文件
5. 解析 AST → 提取 FfiSpec → 后处理 → 生成 Rust 项目

### 其他命令

```bash
# 合并多个 feature 的编译单元输出
cpp2rust-demo merge --feature <name>
```

### 输出目录

```
你的项目/
├── .cpp2rust/<feature>/
│   ├── c/                      # 预处理文件（.cpp2rust + .opts + targets.list）
│   ├── meta/                   # build_cmd.txt、selected_files.json
│   └── rust/                   # 生成的 Rust FFI 项目
│       ├── Cargo.toml
│       └── src/
│           ├── lib.rs          # 汇总所有编译单元
│           └── foo.rs          # 每个 .cpp 对应一个 .rs
└── src/                        # 原始 C++ 项目（工具不修改）
```

开发者拿到生成的 Rust 项目后，只需在 `fn main()` 中编写业务逻辑。

### 依赖

| 依赖 | 用途 |
|------|------|
| g++ / clang++ | 编译原始 C++ 项目 |
| libclang-dev | C++ AST 解析 |
| Rust 工具链 | 构建工具本身和生成项目 |
| `hicc` 0.2 | FFI 框架 |
| `hicc-build` 0.2 | hicc 构建时依赖 |
| Linux LD_PRELOAD | 编译拦截（暂不支持 Windows） |

### 映射示例

**简单函数**：`void hello()` → `fn hello()`

**类**：构造 → `counter_new()`，析构 → `counter_delete()`，`get() const` → `fn get(&self) -> i32`

**模板实例化**：`swap<int>` → `swap_int()`，`swap<double>` → `swap_double()`（模板声明本身不导出）

**运算符重载**：`operator+` → `number_add()`（C ABI 没有运算符符号，降级为命名函数）

**Lambda**：无状态 → 函数指针直接导出；有状态 → 包装成 class 再导出

---

## Part 3：约束与限制

### 核心约束

**一切 C++ 特性最终都要降级成 C ABI 能表达的东西。**

Rust 和 C++ 之间只能走 C ABI。C ABI 只有：函数调用、指针、基本类型。没有类、没有泛型、没有运算符。所有"不能直接映射"的特性，都要通过 shim 层包装成 C ABI 能表达的等价形式。

### 会导出 vs 不会导出

**会导出**：

| C++ 特性 | FFI 表示 |
|----------|---------|
| 全局函数 | 直接映射为 Rust `fn` |
| 类的 public 普通方法 | `import_class!` + `&self`/`&mut self` |
| 构造/析构函数 | 工厂/销毁函数（shim 层 `new`/`delete` 包装） |
| 枚举 | Rust enum |
| 静态成员函数 | 独立 `fn` |
| 模板实例化结果 | 每个实例一个函数 |
| 单/多继承、虚函数 | opaque pointer + 方法分发 |

**不会导出**：

| C++ 特性 | 原因 | 处理方式 |
|----------|------|---------|
| 模板声明本身 | C ABI 没有泛型 | 只导出实例化结果 |
| 运算符重载 | C ABI 没有运算符符号名 | 降级为命名函数 + TODO 标记 |
| 有状态 Lambda | 匿名闭包，捕获列表不可见 | class wrapper + TODO 标记 |
| `std::function` | 类型擦除，内部不透明 | class wrapper + TODO 标记 |
| 系统头文件内容 | 展开后数万行无关代码 | 按源码位置过滤 |
| 私有成员 | 外部不可访问 | 不进入 FfiSpec |
| 宏定义 | 预处理阶段已展开 | 展开后按实际代码处理 |

### 测试体系

测试分五层，逐层递进：

| 层 | 验证什么 | 方法 |
|----|---------|------|
| **L1** | 工具生成的代码和手写参考是否一致 | 从 `rust_hicc/src/main.rs` 提取手写的 `hicc::cpp!` / `import_class!` / `import_lib!` 三段代码作为"黄金标准"，与工具自动生成的 `lib.rs` 逐段文本比对（忽略 `fn main()` 和注释差异） |
| **L2** | 生成的 Rust 项目能否编译通过 | `cargo build` |
| **L3** | 运行结果是否正确 | `cargo run` 输出与预期比对 |
| **L4** | 对真实开源项目（rapidjson）的 E2E 转换 | 完整执行 init + merge，验证生成格式正确 |
| **L5** | C++ 导出符号全部链接进 Rust 二进制 | `nm` 双向符号集比对 |

当前 L1 测试通过率：**49 / 49（100%）**。

### 当前局限性

| 项目 | 状态 |
|------|------|
| 模板跨翻译单元合并 | 当前每个文件独立解析；`merge` 阶段通过去重部分缓解 |
| Windows 支持 | 当前仅 Linux LD_PRELOAD，Windows 暂不支持 |
| 运算符重载 | 生成命名 shim + `[OP]` TODO，Rust 运算符 trait 需手动实现 |
| 有状态 Lambda / std::function | 生成 class wrapper，Rust 闭包回调需手动编写 trampoline |
| 可变参数模板 | 按调用点展开有限版本，超出范围的参数组合需手动添加 |
