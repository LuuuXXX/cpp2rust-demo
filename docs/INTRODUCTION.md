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
C++ 项目          ① 拦截编译          ② 解析 AST          ③ 提取 IR           ④ 后处理          ⑤ 生成 Rust        ⑥ 合并整理
                  (LD_PRELOAD)       (libclang)          (extractor)         (postprocessor)   (generator)        (merger)

foo.cpp ─make─>  g++ 被劫持        .cpp2rust 文件      CppAst 结构体        FfiSpec IR         .rs 文件           merged src/
bar.cpp             │                  │                    │                   │                  │                  │
                    ▼                  ▼                    ▼                   ▼                  ▼                  ▼
            ┌─────────────┐  ┌───────────────┐  ┌──────────────────┐  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐
            │  .cpp2rust  │  │   CppAst       │  │    FfiSpec        │  │  FfiSpec     │  │ hicc 三段式   │  │  统一 crate  │
            │  预处理文件  │  │  结构化 AST    │  │  FFI 中间表示     │  │ (处理特殊情况)│  │ Rust FFI 代码│  │  src/ 整理   │
            └─────────────┘  └───────────────┘  └──────────────────┘  └──────────────┘  └──────────────┘  └──────────────┘
             中间产物 ①        中间产物 ②          中间产物 ③            中间产物 ③(更新)    中间产出           最终产出
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
| Linux/macOS LD_PRELOAD / DYLD_INSERT_LIBRARIES | 编译拦截（捕获阶段；Windows 尚不支持） |

### 映射示例

**简单函数**：`void hello()` → `fn hello()`

**类**：构造 → `counter_new()`，析构 → `counter_delete()`，`get() const` → `fn get(&self) -> i32`

**模板实例化**：`swap<int>` → `swap_int()`，`swap<double>` → `swap_double()`（模板声明本身不导出）

**运算符重载**：`operator+` → `number_add()`（C ABI 没有运算符符号，降级为命名函数）

**Lambda**：无状态 → 函数指针直接导出；有状态 → 包装成 class 再导出

### 类型映射的已知局限

工具在 `src/extractor/type_mapper.rs` 中对 C++ 基础类型做静态映射，以下情况存在已知精度或平台差异：

| C++ 类型 | 默认映射 | 说明 |
|----------|---------|------|
| `long` / `unsigned long` | `i64` / `u64` | 基于 LP64（Linux/macOS 64 位）。Windows MSVC（LLP64）中 `long` 为 32 位，工具在 Windows 下会自动改为 `i32`/`u32` |
| `long double` | `f64` | x86-64 Linux 的 `long double` 是 80 位扩展浮点（`f80`），映射为 `f64` **有精度损失**。带 `cpp2rust-todo[LONG_DOUBLE]` 标注，需手工处理 |
| `void *` | `*mut u8` | 通用指针映射；实际语义依上下文而定 |
| C 函数指针 `T (*)(...)` | `unsafe extern "C" fn(...)` | 嵌套函数指针（参数中含 `(*)`）不支持，回退为原始字符串 |

#### Windows（LLP64）平台差异补充说明

在 64 位 Linux/macOS 上，C++ 的数据模型是 **LP64**（`long` 和指针均为 64 位）；而 Windows 64 位（MSVC & MinGW）采用 **LLP64**（`long` 仍为 32 位，只有 `long long` 和指针为 64 位）。工具在编译时通过 `#[cfg(target_os = "windows")]` 分支自动切换以下映射：

| C++ 类型 | LP64（Linux/macOS） | LLP64（Windows 64 位） |
|----------|--------------------|-----------------------|
| `long` | `i64` | `i32` |
| `unsigned long` | `u64` | `u32` |
| `long double` | `f64`（精度损失，标注 `[LONG_DOUBLE]`） | `f64`（同上） |

> **注意**：捕获阶段（`LD_PRELOAD` hook）目前仅支持 Linux/macOS。Windows 上运行 `cpp2rust init` 时，若需处理含 `long` 的接口，应确认目标平台的数据模型与工具运行时一致，避免类型宽度不匹配。

---

## Part 3：降级特性详解

**降级特性**是指 C++ 中无法直接映射到 C ABI 的特性，工具会自动生成命名 shim 函数 + 内联 TODO 标记，让生成的代码仍能通过 `cargo check`，同时标注出需要手工完善的位置。

共 6 类降级特性，分别对应 TAG `[OP]`、`[VA]`、`[LM]`、`[CV]`、`[FP]`、`[VM]`。

---

### 4.1 `[OP]` 运算符重载（019_operator_overload）

**根本原因**：C ABI 没有 `operator+` 等符号名，FFI 边界只能传命名函数。

**降级策略**：为每个运算符生成命名 shim 函数（如 `number_add`），写入 `hicc::cpp!` 包装层，同时在 `import_lib!` 中声明对应的 Rust 绑定。追加内联 TODO，提示可实现 `std::ops` trait。

#### C++ 项目代码

**operator_overload.h**（头文件中的 C ABI 声明部分）

```cpp
#ifdef __cplusplus
extern "C" {
#endif

struct Number;

struct Number* number_new(int value);
void number_delete(struct Number* self);

int number_getValue(struct Number* self);

struct Number* number_add(struct Number* self, struct Number* other);
struct Number* number_sub(struct Number* self, struct Number* other);
struct Number* number_mul(struct Number* self, struct Number* other);
struct Number* number_div(struct Number* self, struct Number* other);

int number_compare(struct Number* self, struct Number* other);

struct Number* number_negate(struct Number* self);
struct Number* number_increment(struct Number* self);
struct Number* number_decrement(struct Number* self);

void number_add_assign(struct Number* self, struct Number* other);
void number_sub_assign(struct Number* self, struct Number* other);

#ifdef __cplusplus
}

// C++ 类定义（含运算符重载）
class Number {
    int value;
public:
    Number(int v);
    ~Number();
    int getValue() const;
    Number operator+(const Number& other) const;
    Number operator-(const Number& other) const;
    Number operator*(const Number& other) const;
    Number operator/(const Number& other) const;
    int compare(const Number& other) const;
    Number operator-() const;
    Number& operator++();
    Number& operator--();
    Number& operator+=(const Number& other);
    Number& operator-=(const Number& other);
};

#endif
```

**operator_overload.cpp**（shim 包装 + 类实现）

```cpp
#include "operator_overload.h"

// shim 包装函数：把运算符重载暴露为命名 C 函数
struct Number* number_add(struct Number* self, struct Number* other) {
    return new Number(self->operator+(*other));
}
struct Number* number_sub(struct Number* self, struct Number* other) {
    return new Number(self->operator-(*other));
}
struct Number* number_mul(struct Number* self, struct Number* other) {
    return new Number(self->operator*(*other));
}
struct Number* number_div(struct Number* self, struct Number* other) {
    return new Number(self->operator/(*other));
}
int number_compare(struct Number* self, struct Number* other) {
    return self->compare(*other);
}
struct Number* number_negate(struct Number* self) {
    return new Number(self->operator-());
}
struct Number* number_increment(struct Number* self) {
    return &self->operator++();
}
void number_add_assign(struct Number* self, struct Number* other) {
    self->operator+=(*other);
}

// Number 类实现
Number::Number(int v) : value(v) {}
Number::~Number() {}
int Number::getValue() const { return value; }
Number Number::operator+(const Number& other) const { return Number(value + other.value); }
Number Number::operator-(const Number& other) const { return Number(value - other.value); }
Number Number::operator*(const Number& other) const { return Number(value * other.value); }
Number Number::operator/(const Number& other) const { return Number(value / other.value); }
int Number::compare(const Number& other) const { return value - other.value; }
Number Number::operator-() const { return Number(-value); }
Number& Number::operator++() { ++value; return *this; }
Number& Number::operator--() { --value; return *this; }
Number& Number::operator+=(const Number& other) { value += other.value; return *this; }
Number& Number::operator-=(const Number& other) { value -= other.value; return *this; }
```

#### 生成的 Rust FFI 代码（lib.rs）

```rust
// 段 1：C++ shim 层（运算符 → 命名函数）
hicc::cpp! {
    #include <iostream>
    #include "operator_overload.h"

    int number_get_value(const Number* self) {
        return self->getValue();
    }
    Number* number_add(const Number* a, const Number* b) {
        return new Number(*a + *b);
    }
    Number* number_sub(const Number* a, const Number* b) {
        return new Number(*a - *b);
    }
    Number* number_mul(const Number* a, const Number* b) {
        return new Number(*a * *b);
    }
    Number* number_div(const Number* a, const Number* b) {
        return new Number(*a / *b);
    }
    Number* number_negate(const Number* a) {
        return new Number(-*a);
    }
    int number_compare(const Number* a, const Number* b) {
        return a->compare(*b);
    }
}

// 段 2：类方法绑定
hicc::import_class! {
    #[cpp(class = "Number", destroy = "number_delete")]
    pub class Number {
        #[cpp(method = "int getValue() const")]
        fn get_value(&self) -> i32;
    }
}

// 段 3：全局函数 + 运算符 shim 绑定
hicc::import_lib! {
    #![link_name = "operator_overload"]

    class Number;

    #[cpp(func = "Number* number_new(int)")]
    fn number_new(value: i32) -> Number;

    #[cpp(func = "Number* number_add(const Number*, const Number*)")]
    fn number_add(a: *const Number, b: *const Number) -> *mut Number;
    // cpp2rust-todo[OP]: 可实现 impl std::ops::Add<Number> for Number

    #[cpp(func = "Number* number_sub(const Number*, const Number*)")]
    fn number_sub(a: *const Number, b: *const Number) -> *mut Number;
    // cpp2rust-todo[OP]: 可实现 impl std::ops::Sub<Number> for Number

    #[cpp(func = "Number* number_mul(const Number*, const Number*)")]
    fn number_mul(a: *const Number, b: *const Number) -> *mut Number;
    // cpp2rust-todo[OP]: 可实现 impl std::ops::Mul<Number> for Number

    #[cpp(func = "Number* number_div(const Number*, const Number*)")]
    fn number_div(a: *const Number, b: *const Number) -> *mut Number;
    // cpp2rust-todo[OP]: 可实现 impl std::ops::Div<Number> for Number

    #[cpp(func = "Number* number_negate(const Number*)")]
    fn number_negate(a: *const Number) -> *mut Number;
    // cpp2rust-todo[OP]: 可实现 impl std::ops::Neg for Number

    #[cpp(func = "int number_compare(const Number*, const Number*)")]
    fn number_compare(a: *const Number, b: *const Number) -> i32;
    // cpp2rust-todo[OP]: 可实现 impl std::cmp::Ord for Number
}
```

**用户剩余工作（可选）**：手动实现 `impl std::ops::Add<Number> for Number` 等 Rust trait，使 Rust 侧可以用 `a + b` 语法代替 `number_add(&a, &b)`。

---

### 4.2 `[VA]` 可变参数模板（028_variadic_template）

**根本原因**：C++ 可变参数模板（`template<typename... Args>`）是编译期展开，FFI 运行期无法表达"任意数量参数"。

**降级策略**：在 `hicc::cpp!` 中生成 wrapper 类（`SumCalculator`），按参数数量和类型组合分别封装为静态方法（`calculate_1`/`calculate_2` 等），再生成 C 兼容命名包装函数（`sum_1`/`sum_2` 等）；`import_lib!` 绑定各包装函数，追加内联 TODO。

#### C++ 项目代码

**variadic_template.h**

```cpp
#ifdef __cplusplus
extern "C" {
#endif

// FFI 无法直接表达可变参数，导出固定参数版本
int sum_zero(void);
int sum_1(int a);
int sum_2(int a, int b);
int sum_3(int a, int b, int c);
int sum_4(int a, int b, int c, int d);
int sum_5(int a, int b, int c, int d, int e);

double sum_double_2(double a, double b);
double sum_double_3(double a, double b, double c);
double sum_double_4(double a, double b, double c, double d);

const char* sum_getFormat(int count);

#ifdef __cplusplus
}

// C++ wrapper 类（封装模板实例化结果）
class SumCalculator {
public:
    static int calculate_zero();
    static int calculate_1(int a);
    static int calculate_2(int a, int b);
    static int calculate_3(int a, int b, int c);
    static int calculate_4(int a, int b, int c, int d);
    static int calculate_5(int a, int b, int c, int d, int e);
    static double calculate_double_2(double a, double b);
    static double calculate_double_3(double a, double b, double c);
    static double calculate_double_4(double a, double b, double c, double d);
    static const char* get_format(int count);
};

#endif
```

**variadic_template.cpp**（C ABI 包装函数 + 类实现）

```cpp
#include "variadic_template.h"

// C ABI 包装函数委托给 SumCalculator 静态方法
int sum_zero(void)                    { return SumCalculator::calculate_zero(); }
int sum_1(int a)                      { return SumCalculator::calculate_1(a); }
int sum_2(int a, int b)               { return SumCalculator::calculate_2(a, b); }
int sum_3(int a, int b, int c)        { return SumCalculator::calculate_3(a, b, c); }
int sum_4(int a, int b, int c, int d) { return SumCalculator::calculate_4(a, b, c, d); }
int sum_5(int a, int b, int c, int d, int e) { return SumCalculator::calculate_5(a, b, c, d, e); }
double sum_double_2(double a, double b) { return SumCalculator::calculate_double_2(a, b); }
double sum_double_3(double a, double b, double c) { return SumCalculator::calculate_double_3(a, b, c); }
double sum_double_4(double a, double b, double c, double d) { return SumCalculator::calculate_double_4(a, b, c, d); }
const char* sum_getFormat(int count) { return SumCalculator::get_format(count); }

// SumCalculator 类实现（对应模板实例化结果）
int SumCalculator::calculate_zero() { return 0; }
int SumCalculator::calculate_1(int a) { return a; }
int SumCalculator::calculate_2(int a, int b) { return a + b; }
int SumCalculator::calculate_3(int a, int b, int c) { return a + b + c; }
int SumCalculator::calculate_4(int a, int b, int c, int d) { return a + b + c + d; }
int SumCalculator::calculate_5(int a, int b, int c, int d, int e) { return a + b + c + d + e; }
double SumCalculator::calculate_double_2(double a, double b) { return a + b; }
double SumCalculator::calculate_double_3(double a, double b, double c) { return a + b + c; }
double SumCalculator::calculate_double_4(double a, double b, double c, double d) { return a + b + c + d; }
const char* SumCalculator::get_format(int count) {
    switch (count) {
        case 0: return "sum()";
        case 1: return "sum(%d)";
        case 2: return "sum(%d, %d)";
        case 3: return "sum(%d, %d, %d)";
        default: return "unknown";
    }
}
```

#### 生成的 Rust FFI 代码（lib.rs）

```rust
// 段 1：C++ 头文件内联（wrapper 类已在 .h 中定义）
hicc::cpp! {
    #include <iostream>
    #include <cstdarg>
    #include "variadic_template.h"
}

// 段 2：无 import_class!（SumCalculator 所有方法均为静态，暴露为全局函数）

// 段 3：固定参数版本函数绑定
hicc::import_lib! {
    #![link_name = "variadic_template"]

    #[cpp(func = "int sum_zero()")]
    fn sum_zero() -> i32;

    #[cpp(func = "int sum_1(int)")]
    fn sum_1(a: i32) -> i32;

    #[cpp(func = "int sum_2(int, int)")]
    fn sum_2(a: i32, b: i32) -> i32;

    #[cpp(func = "int sum_3(int, int, int)")]
    fn sum_3(a: i32, b: i32, c: i32) -> i32;

    #[cpp(func = "int sum_4(int, int, int, int)")]
    fn sum_4(a: i32, b: i32, c: i32, d: i32) -> i32;

    #[cpp(func = "int sum_5(int, int, int, int, int)")]
    fn sum_5(a: i32, b: i32, c: i32, d: i32, e: i32) -> i32;
    // cpp2rust-todo[VA]: 可变参数模板，已按调用点展开 5 个版本
    //                    新增参数组合时需手动在 hicc::cpp! 中添加对应版本

    #[cpp(func = "double sum_double_2(double, double)")]
    fn sum_double_2(a: f64, b: f64) -> f64;

    #[cpp(func = "double sum_double_3(double, double, double)")]
    fn sum_double_3(a: f64, b: f64, c: f64) -> f64;

    #[cpp(func = "double sum_double_4(double, double, double, double)")]
    fn sum_double_4(a: f64, b: f64, c: f64, d: f64) -> f64;

    #[cpp(func = "const char* sum_getFormat(int)")]
    unsafe fn sum_get_format(count: i32) -> *const i8;
}
```

**用户剩余工作**：若需要新的参数数量或类型组合（如 `sum_6` / `sum_double_5`），手动在 `hicc::cpp!` 中为 `SumCalculator` 添加对应静态方法和 C 包装函数，并在 `import_lib!` 中声明。

---

### 4.3 `[LM]` 有状态 Lambda（039_lambda_basic）

**根本原因**：有状态 lambda（含捕获列表）是匿名闭包类型，FFI 无法表达捕获列表，无法自动推断 `operator()` 的真实签名。

**降级策略（双策略）**：
- **无状态 lambda**（空捕获 `[]`）→ 退化为普通函数，直接在 `import_lib!` 中声明为函数绑定，无需 shim。
- **有状态 lambda**（含捕获）→ `hicc::cpp!` 中生成 class wrapper（如 `LambdaWrapper`/`StateLambda`），通过 ctor/call/dtor shim 暴露；Rust 侧通过 `import_class!` 调用 `invoke()`/`add()` 等方法。

#### C++ 项目代码

**lambda_basic.h**（关键部分）

```cpp
#ifdef __cplusplus
extern "C" {
#endif

typedef int (*IntBinaryOp)(int, int);

// 无状态 lambda 退化为普通函数——直接导出
int add_impl(int a, int b);
int multiply_impl(int a, int b);
int max_impl(int a, int b);

// 有状态 lambda wrapper
struct LambdaWrapper;
struct LambdaWrapper* lambda_wrapper_new(int (*fn)(int, int));
void lambda_wrapper_delete(struct LambdaWrapper* self);
struct LambdaWrapper* make_add_lambda(void);
struct LambdaWrapper* make_multiply_lambda(void);

// 捕获外部状态的 lambda
struct StateLambda;
struct StateLambda* state_lambda_new(int initial_value);
void state_lambda_delete(struct StateLambda* self);
int state_lambda_get_value(const struct StateLambda* self);

#ifdef __cplusplus
}

// C++ class wrapper 定义（封装有状态 lambda）
#include <functional>

struct LambdaWrapper {
    LambdaWrapperImpl* impl;
    explicit LambdaWrapper(int (*fn)(int, int));
    ~LambdaWrapper();
    int invoke(int a, int b) { return impl->fn(a, b); }
};

struct StateLambda {
    StateLambdaImpl* impl;
    explicit StateLambda(int initial_value);
    ~StateLambda();
    int get_value() const { return impl->value; }
    int add(int delta) { return impl->adder(delta); }
    // StateLambdaImpl::adder 是捕获 this 的 lambda：
    // adder = [this](int delta) { return value += delta; }
};

#endif
```

**lambda_basic.cpp**（无状态函数 + 有状态 wrapper 实现）

```cpp
#include "lambda_basic.h"
#include <functional>
#include <algorithm>

// 无状态 lambda 的等价普通函数（可直接做函数指针）
int add_impl(int a, int b)      { return a + b; }
int multiply_impl(int a, int b) { return a * b; }
int max_impl(int a, int b)      { return std::max(a, b); }

// make_*_lambda：把无状态函数包装进 LambdaWrapper（工厂函数）
struct LambdaWrapper* make_add_lambda(void)      { return new LambdaWrapper(add_impl); }
struct LambdaWrapper* make_multiply_lambda(void) { return new LambdaWrapper(multiply_impl); }

// StateLambda：内部持有捕获 this 的有状态 lambda
struct StateLambda* state_lambda_new(int initial_value) {
    return new StateLambda(initial_value);
}
void state_lambda_delete(struct StateLambda* self) { delete self; }
int state_lambda_get_value(const struct StateLambda* self) {
    return self ? self->impl->value : 0;
}
```

#### 生成的 Rust FFI 代码（lib.rs）

```rust
// 段 1：C++ 头文件内联
hicc::cpp! {
    #include <stddef.h>
    #include <iostream>
    #include <functional>
    #include <algorithm>
    #include "lambda_basic.h"

    typedef int (*IntBinaryOp)(int, int);
}

// 段 2：有状态 lambda wrapper 的类方法绑定
hicc::import_class! {
    #[cpp(class = "LambdaWrapper", destroy = "lambda_wrapper_delete")]
    pub class LambdaWrapper {
        // cpp2rust-todo[LM]: 有状态 lambda，内部捕获状态不透明，已封装为 class wrapper
        #[cpp(method = "int invoke(int a, int b)")]
        fn invoke(&mut self, a: i32, b: i32) -> i32;
    }
}

hicc::import_class! {
    #[cpp(class = "StateLambda", destroy = "state_lambda_delete")]
    pub class StateLambda {
        // cpp2rust-todo[LM]: 有状态 lambda，捕获外部 int 状态
        #[cpp(method = "int get_value() const")]
        fn get_value(&self) -> i32;

        #[cpp(method = "int add(int delta)")]
        fn add(&mut self, delta: i32) -> i32;
    }
}

// 段 3：工厂函数 + 无状态 lambda 对应的直接函数绑定
hicc::import_lib! {
    #![link_name = "lambda_basic"]

    class LambdaWrapper;
    class StateLambda;

    // 无状态 lambda 退化为普通函数——直接绑定
    #[cpp(func = "int add_impl(int, int)")]
    fn add_impl(a: i32, b: i32) -> i32;

    #[cpp(func = "int multiply_impl(int, int)")]
    fn multiply_impl(a: i32, b: i32) -> i32;

    #[cpp(func = "int max_impl(int, int)")]
    fn max_impl(a: i32, b: i32) -> i32;

    // 有状态 lambda 工厂函数
    #[cpp(func = "LambdaWrapper* make_add_lambda()")]
    fn make_add_lambda() -> *mut LambdaWrapper;

    #[cpp(func = "LambdaWrapper* make_multiply_lambda()")]
    fn make_multiply_lambda() -> *mut LambdaWrapper;

    #[cpp(func = "StateLambda* state_lambda_new(int)")]
    fn state_lambda_new(initial_value: i32) -> StateLambda;
}
```

**用户剩余工作**：若需要从 Rust 侧传递 Rust 闭包给 C++ 回调，需手动编写 trampoline 函数（`extern "C" fn`）。

---

### 4.4 `[LM]` std::function（040_std_function）

**根本原因**：`std::function<Sig>` 是类型擦除容器，签名可推断但内部捕获状态不透明，与有状态 lambda 同属一类问题。

**降级策略**：统一使用 class wrapper + opaque pointer 策略。C++ 侧把 `std::function` 封装在 class（如 `CallbackWrapper`/`Processor`）中，通过工厂函数和方法暴露给 FFI；Rust 侧通过 `import_class!` 调用 `invoke()`/`process()` 等方法。

#### C++ 项目代码

**std_function.h**（关键部分）

```cpp
#ifdef __cplusplus
extern "C" {
#endif

// std::function<int(int)> 封装为 CallbackWrapper opaque pointer
struct CallbackWrapper;
struct CallbackWrapper* callback_wrapper_new(int (*fn)(int));
struct CallbackWrapper* callback_wrapper_new_double(void); // 预置 x*2 回调
void callback_wrapper_delete(struct CallbackWrapper* self);

// std::function<int(int)> 封装为 Processor
struct Processor;
struct Processor* processor_new(void);
void processor_set_double(struct Processor* p);
void processor_delete(struct Processor* self);

// 多回调链
struct MultiCallback;
struct MultiCallback* multi_callback_new(void);
void multi_callback_add_double(struct MultiCallback* mc);
void multi_callback_add_triple(struct MultiCallback* mc);
void multi_callback_delete(struct MultiCallback* self);

#ifdef __cplusplus
}

// C++ class wrapper（持有 std::function）
#include <functional>
#include <vector>

struct CallbackWrapper {
    CallbackWrapperImpl* impl;
    explicit CallbackWrapper(int (*fn)(int));
    ~CallbackWrapper();
    int invoke(int value) { return impl->invoke(value); }
};

struct Processor {
    ProcessorImpl* impl;
    Processor();
    ~Processor();
    int process(int value) { return impl->process(value); }
};

struct MultiCallback {
    MultiCallbackImpl* impl;
    MultiCallback();
    ~MultiCallback();
    void invoke_all(int value) { impl->invoke_all(value); }
};

#endif
```

**std_function.cpp**（工厂函数实现）

```cpp
#include "std_function.h"
#include <functional>
#include <vector>

// CallbackWrapper 持有 std::function<int(int)>
CallbackWrapperImpl::CallbackWrapperImpl(int (*fn)(int)) : callback(fn) {}
int CallbackWrapperImpl::invoke(int value) {
    return callback ? callback(value) : value;
}

// callback_wrapper_new_double：用 lambda 初始化 std::function
struct CallbackWrapper* callback_wrapper_new_double(void) {
    return new CallbackWrapper([](int x) -> int { return x * 2; });
    // cpp2rust-todo[LM]: std::function，已封装为 class wrapper
}

// Processor 持有 std::function<int(int)>，支持运行时替换回调
void processor_set_double(struct Processor* p) {
    p->impl->set_callback([](int x) -> int { return x * 2; });
}

// MultiCallback 持有 std::vector<std::function<int(int)>>
void multi_callback_add_double(struct MultiCallback* mc) {
    mc->impl->add([](int x) -> int { return x * 2; });
}
void multi_callback_add_triple(struct MultiCallback* mc) {
    mc->impl->add([](int x) -> int { return x * 3; });
}
```

#### 生成的 Rust FFI 代码（lib.rs）

```rust
// 段 1：C++ 头文件内联
hicc::cpp! {
    #include <stddef.h>
    #include <iostream>
    #include <functional>
    #include <vector>
    #include <thread>
    #include <chrono>
    #include "std_function.h"
}

use hicc::AbiClass;

// 段 2：std::function wrapper 类的方法绑定
hicc::import_class! {
    // cpp2rust-todo[LM]: std::function，已封装为 class wrapper
    #[cpp(class = "CallbackWrapper", destroy = "callback_wrapper_delete")]
    pub class CallbackWrapper {
        #[cpp(method = "int invoke(int value)")]
        fn invoke(&mut self, value: i32) -> i32;
    }
}

hicc::import_class! {
    #[cpp(class = "Processor", destroy = "processor_delete")]
    pub class Processor {
        #[cpp(method = "int process(int value)")]
        fn process(&mut self, value: i32) -> i32;
    }
}

hicc::import_class! {
    #[cpp(class = "MultiCallback", destroy = "multi_callback_delete")]
    pub class MultiCallback {
        #[cpp(method = "void invoke_all(int value)")]
        fn invoke_all(&mut self, value: i32);
    }
}

// 段 3：工厂函数绑定
hicc::import_lib! {
    #![link_name = "std_function"]

    class CallbackWrapper;
    class Processor;
    class MultiCallback;

    // 预置回调的工厂函数（内部用 lambda 初始化 std::function）
    #[cpp(func = "CallbackWrapper* callback_wrapper_new_double()")]
    fn callback_wrapper_new_double() -> CallbackWrapper;

    #[cpp(func = "Processor* processor_new()")]
    fn processor_new() -> Processor;

    #[cpp(func = "MultiCallback* multi_callback_new()")]
    fn multi_callback_new() -> MultiCallback;

    // 运行时修改回调（C++ 侧用 lambda 赋值 std::function）
    #[cpp(func = "void processor_set_double(Processor* p)")]
    unsafe fn processor_set_double(p: *mut Processor);

    #[cpp(func = "void multi_callback_add_double(MultiCallback* mc)")]
    unsafe fn multi_callback_add_double(mc: *mut MultiCallback);

    #[cpp(func = "void multi_callback_add_triple(MultiCallback* mc)")]
    unsafe fn multi_callback_add_triple(mc: *mut MultiCallback);
}
```

**用户剩余工作（可选）**：手动实现 Rust 闭包 → C++ `std::function` 适配层：在 C++ 侧暴露 `callback_wrapper_new(int (*fn)(int))` 工厂函数，Rust 侧用 `extern "C" fn` 作为函数指针传入。

---

### 4.5 `[CV]` — C 可变参数函数（005_variadic_functions）

**根本原因**：Rust 的 FFI 接口要求每个参数都有精确的静态类型，而 C 的 `...` 只在运行时通过 `va_list` 访问参数，无法在编译期表达参数数量与类型。

```c
// 头文件中的 C 可变参数函数 —— 工具跳过此函数
int sum(int count, ...);            // ← is_variadic=true，整体跳过

// 手动提供的固定参数 wrapper —— 工具正常绑定
int sum_3(int a, int b, int c);
int sum_5(int a, int b, int c, int d, int e);
```

生成结果：`sum` 不出现；`sum_3` / `sum_5` 正常进入 `import_lib!`。

**用户操作**：若现有 wrapper 数量不足（例如需要 `sum_4`），在头文件和实现文件中手动添加，重新运行 `cpp2rust-demo init`。

---

### 4.6 `[FP]` — 函数指针参数（039_lambda_basic / 040_std_function）

**C 函数指针**（`int (*op)(int, int)` 形式）自动映射为 `unsafe extern "C" fn(i32, i32) -> i32`，函数标记 `is_unsafe = true`，并在 `#[cpp(func = "...")]` 前自动插入：

```rust
// cpp2rust-todo[FP]: 含函数指针参数，需确保回调符合 extern "C" 调用约定
#[cpp(func = "int apply_operation(int, int, int (*)(int, int))")]
unsafe fn apply_operation(a: i32, b: i32, op: unsafe extern "C" fn(i32, i32) -> i32) -> i32;
```

**C++ 成员函数指针**（`int (Cls::*)() const` 形式）仍无法映射为合法 Rust FFI 类型，含此类参数的函数整体跳过。

生成结果：`apply_operation`、`lambda_wrapper_new` 等含 C 函数指针参数的函数现在出现在 `import_lib!` 中；`add_impl`、`make_add_lambda` 等普通函数不受影响。

**用户操作**：确认传入的回调函数符合 `extern "C"` 调用约定（无 Rust 闭包捕获、无 panic）；若需 Rust 闭包 → C++ 回调，手动编写 trampoline。

---

### 4.7 `[VM]` — volatile 成员函数（012_class_volatile）

**根本原因**：hicc 通过方法指针类型（`R (T::*)() volatile`）绑定成员方法，而 Rust 的 `fn` 签名中没有 `volatile this` 的概念，类型不匹配导致编译失败。工具因此将 volatile 方法从 `import_class!` 中整体移除。

```cpp
class HardwareDevice {
public:
    void init();                             // 普通方法 —— 进入 import_class!
    uint32_t readStatus() volatile; // volatile 方法 —— 从 import_class! 移除 [VM]
};

// extern "C" shim（接收 volatile T* 作为第一参数）—— 仍进入 import_lib!
uint32_t hardware_device_read_status(volatile HardwareDevice* self);
```

生成结果：`HardwareDevice` 的 `import_class!` 中只有 `init`；`readStatus` 被跳过；但 `hardware_device_read_status(volatile HardwareDevice*)` 作为自由函数进入 `import_lib!`（标注 `unsafe`）。

**用户操作**：优先在 `extern "C"` 头文件中提供 `volatile T*` 参数的 C shim，工具即可自动生成 `import_lib!` 绑定。若头文件中无对应 shim，手动在 `hicc::cpp!` 中添加并声明到 `import_lib!`。

---

### 4.8 降级特性对比总览

| TAG | 特性 | 示例 | 降级前（C++ 侧） | 降级后（FFI 侧） | Rust 侧剩余工作 |
|-----|------|------|----------------|----------------|----------------|
| `[OP]` | 运算符重载 | 019 | `Number::operator+` | `number_add(a, b)` 命名 shim | 可选：实现 `std::ops::Add` trait |
| `[VA]` | 可变参数模板 | 028 | `sum<Args...>(args...)` | `sum_1`/`sum_2`/`sum_3`... 分版本 | 需新参数组合时手动添加版本 |
| `[LM]` | 有状态 Lambda | 039 | `[&x](int a){ return a+x; }` | class wrapper + `invoke()` 方法 | 若需传 Rust 闭包，编写 trampoline |
| `[LM]` | std::function | 040 | `std::function<int(int)>` | class wrapper + `invoke()` 方法 | 可选：实现 Rust 闭包 → C++ 适配层 |
| `[CV]` | C 可变参数函数 | 005 | `int sum(int count, ...)` | 整体跳过；固定参数 wrapper 正常绑定 | 手动提供各参数组合的固定 wrapper |
| `[FP]` | 函数指针参数 | 039, 040 | `int (*op)(int, int)` 参数 | 自动映射为 `unsafe extern "C" fn`，加 `[FP]` 注释 | 确认回调符合 `extern "C"` 调用约定 |
| `[VM]` | volatile 成员函数 | 012 | `uint32_t readStatus() volatile` | 方法从 `import_class!` 整体移除 | 在头文件中提供 `volatile T*` C shim |

所有降级处均标注 `// cpp2rust-todo[TAG]: ...` 注释，可通过 `grep -r "cpp2rust-todo"` 快速定位待手动完善的位置。

---

## Part 3.5：Phase 6 — merge 阶段技术细节

### `merge_in_place` 的原子性 rename 机制

`init` 命令将每个翻译单元生成到 `.cpp2rust/<feature>/rust/src/<相对路径>.rs`，输出结构是扁平目录。`merge_in_place` 在此基础上执行以下原子性重组：

```
首次运行：
  src/  (init 输出)  →  src.1/  (永久备份，只创建一次)
  src.2/ (merge 输出) →  src/  (原子 rename)

重复运行：
  src.1/ 已存在  →  直接用 src.1/ 作为输入源（不再覆盖）
  src/  →  src.2/  →  src/  (原子 rename 覆盖旧 merge 结果)
```

核心设计原则：
- **幂等性**：重复运行不破坏 `src.1/` 备份，始终保留最初的 init 原始输出
- **原子性**：`std::fs::rename` 在同一文件系统内是原子操作，中途失败不会留下不完整状态
- **可追溯性**：`src.1/` 保存的是 init 阶段未经整理的单元文件，方便比对 merge 前后差异

### 跨翻译单元 `cpp_lines` 去重策略

`init` 阶段每个翻译单元的 `hicc::cpp!` 块都会独立生成一份 `#include` 行。多翻译单元合并时若不去重，同一个头文件的 `#include` 会出现数十次，导致 C++ 编译报重定义错误。

`merge_units`（`merger/mod.rs`）的去重逻辑：

```
merge_units(paths) →
  for each unit .rs file:
    parse_unit_rs → ParsedUnit { cpp_lines, class_blocks, lib_block, ... }
    for each cpp_line:
      if not in seen_cpp_lines set:
        append to merged_cpp_lines
        insert into seen_cpp_lines
```

去重以**行为单位**（`String` 精确匹配），不进行语义分析。这意味着：
- `#include "foo.h"` 和 `  #include "foo.h"` 视为不同行（实际上 codegen 输出格式固定，不会有缩进差异）
- 模板类体（`template<class T> class Stack { ... }`）采用**块级去重**：将整个类体规范化为字符串后加入 `template_bodies` set，跨翻译单元重复的模板定义只保留一份

### 模板特化分组逻辑

当同一个模板类在多个翻译单元有不同实例化（如 `Stack<int>` 和 `Stack<float>`），`merge_units` 通过 `template_base` 字段将它们归组：

- `block_parser.rs` 在解析 `import_class!` 块时，若类名含 `<`（如 `Stack<int>`），自动提取 `template_base = "Stack"`
- `merge_units` 按 `template_base` 分组，同一基类的所有特化 `import_class!` 块在最终输出中相邻排列
- `lib_block` 中的前向声明（`class Stack<int>;`）按特化实例分别生成，不合并

### 冲突检测与报告生成

`merge_units` 在合并类方法时检测**方法签名冲突**：若同一类的同名方法在不同翻译单元有不同的 `#[cpp(method = "...")]` 属性（C++ 签名不同），则：

1. 记录到 `MergedSpec::conflicts` 列表
2. 在 `api-manifest.md` 中以 `⚠️ CONFLICT` 标记该方法
3. 取**最后出现**的版本写入合并结果（保守策略）

`merge` 命令完成后在 `.cpp2rust/<feature>/meta/` 生成两份报告：
- `api-manifest.md`：汇总所有导出接口，格式为 Markdown 表格，按类/函数分节，含 C++ 签名和 Rust 对应声明
- `merge-report.md`：合并统计（翻译单元数、去重前后 cpp 行数、类/函数绑定数、冲突数）

---

## Part 4：约束与限制

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
| **L4** | 对真实开源项目的 E2E 转换 | 完整执行 init + merge，验证生成格式正确；覆盖 rapidjson（10 子系统）+ tinyxml2 / pugixml / sqlite3 / nlohmann-json / fmtlib |
| **L5** | C++ 导出符号全部链接进 Rust 二进制 | `nm` 双向符号集比对 |

当前 L1 测试通过率：**49 / 49（100%）**。

### 当前局限性

| 项目 | 状态 |
|------|------|
| 模板跨翻译单元合并 | 当前每个文件独立解析；`merge` 阶段通过去重部分缓解 |
| Windows 编译拦截 | 捕获阶段依赖 `LD_PRELOAD`（Linux）/ `DYLD_INSERT_LIBRARIES`（macOS），Windows 尚无等价机制；生成的 Rust 项目（L2/L3/L5）已在 Windows MSVC & MinGW 通过 CI |
| 运算符重载 | 生成命名 shim + `[OP]` TODO，Rust 运算符 trait 需手动实现 |
| 有状态 Lambda / std::function | 生成 class wrapper，Rust 闭包回调需手动编写 trampoline |
| 可变参数模板 | 按调用点展开有限版本，超出范围的参数组合需手动添加 |
