# C++ 到 Rust Safe FFI 自动化工具 - 方案 v4

## 1. 背景

### 1.1 演进路线

| 版本 | 核心突破 | 遗留问题 |
|------|---------|---------|
| v1 | 头文件解析 → 自动生成 hicc 脚手架 | ❌ 模板实例化 |
| v2 | AST 编译捕获 → 解决模板实例化 | ❌ 运算符/友元/lambda/RTTI 无法生成 |
| v3 | 后处理降级 → 5 类特性生成可编译代码 + TODO 清单 | ⚠️ 代码格式不对齐、TODO 系统过重 |
| **v4** | **修正 v3 三处核心缺陷，全面对齐 hicc 生态** | — |

### 1.2 v3 的三处核心缺陷

通过对 `examples/` 手写参考输出的分析，发现 v3 计划与实际 hicc 编程模型存在偏差：

**缺陷 1：代码格式与 hicc 宏不对齐**

v3 方案的代码示例使用裸 `#[link]` + `unsafe extern "C"` 块，但实际所有 examples 均使用：

```
hicc::cpp!{ ... }          ← C++ 实现内联在 Rust 源文件中
hicc::import_class!{ ... } ← 导入类及其成员方法
hicc::import_lib!{ ... }   ← 导入全局 C-compatible 函数
```

**缺陷 2：Lambda/Closure 处理策略偏差**

v3 仅讨论了"简单 lambda → 闭包"，忽略了 `examples/039_lambda_basic` 揭示的两种核心模式：
- **无状态 lambda**：退化为函数指针 `fn(i32, i32) -> i32`（`IntBinaryOp` 模式）
- **有状态 lambda**：封装为 class wrapper + opaque pointer（`StateLambda`/`LambdaWrapper` trampoline 模式）

**缺陷 3：RTTI 策略 ABI 不稳定**

v3 提出用 mangled name 字符串做映射表，但 `examples/023_typeid_rtti` 展示了稳定的替代方案：整数枚举 + 虚函数 `getTypeName()` —— 完全不依赖编译器特定的名称修饰。

### 1.3 v4 目标

1. **格式对齐**：自动生成的 Rust 代码完全使用 `hicc::cpp!` / `import_class!` / `import_lib!` 宏格式
2. **Lambda 全覆盖**：同时处理无状态（fn pointer）和有状态（class wrapper）两种 lambda 模式
3. **RTTI 稳定化**：改用枚举 + 虚函数方案，放弃 mangled name 字符串映射
4. **轻量 TODO**：用内联注释替代独立 JSON 文件，降低工具复杂度
5. **std::bind 完全支持**：从 v3 的 ⚠️ 升级为 ✅（class wrapper 模式与 lambda 统一处理）

---

## 2. 技术方案

### 2.1 整体架构

```
cpp2rust-ffi tool (v4)
├── src/
│   ├── main.rs                        # CLI 入口 (init / merge)
│   ├── compiler/
│   │   ├── ast_compiler.rs            # libclang 编译 C++ 源文件
│   │   └── cursor_visitor.rs          # AST 遍历器
│   ├── extractor/
│   │   ├── class_extractor.rs         # 类/结构体/友元
│   │   ├── function_extractor.rs      # 函数（含 operator）
│   │   ├── template_extractor.rs      # 模板实例化
│   │   ├── vtable_extractor.rs        # 虚函数表
│   │   ├── lambda_extractor.rs        # Lambda/Closure（新增）
│   │   └── enum_extractor.rs          # 枚举
│   ├── postprocessor/
│   │   ├── operator_handler.rs        # 运算符重载 → named shim
│   │   ├── friend_handler.rs          # 友元函数 → import_lib!
│   │   ├── lambda_handler.rs          # Lambda → fn ptr / class wrapper
│   │   ├── rtti_handler.rs            # RTTI → enum + virtual fn
│   │   └── variadic_handler.rs        # 可变参数模板 → fixed arity
│   ├── generator/
│   │   ├── hicc_codegen.rs            # hicc 宏格式代码生成（v4 新增）
│   │   ├── class_generator.rs
│   │   ├── template_generator.rs
│   │   ├── vtable_generator.rs
│   │   └── project_generator.rs
│   └── todo_collector.rs              # 内联注释生成器（简化版）
└── Cargo.toml
```

### 2.2 四阶段处理流程

```
1. 编译 (compiler/)
   └── libclang 编译 C++ 源文件，触发模板实例化

2. 提取 (extractor/)
   ├── 类/结构体/友元/运算符
   ├── 函数（含 operator 方法）
   ├── 模板实例化
   ├── 虚函数表
   ├── Lambda 表达式（有状态 / 无状态分类）
   └── 枚举

3. 后处理 (postprocessor/)
   ├── 运算符重载 → named shim 函数（在 import_lib! 中导出）
   ├── 友元函数   → 独立条目加入 import_lib!
   ├── Lambda     → 无状态：fn ptr；有状态：class wrapper + opaque ptr
   ├── RTTI       → 枚举 + 虚函数
   └── 可变参数   → 固定元数命名函数

4. 生成 (generator/)
   └── hicc 宏格式 Rust 代码（hicc::cpp! + import_class! + import_lib!）
```

### 2.3 输出目录结构

```
rust_hicc/
├── Cargo.toml
├── build.rs
└── src/
    ├── lib.rs          # 或 main.rs
    ├── foo.rs          # 每个 TU 一个 .rs 文件
    └── bar.rs
```

生成的每个 `.rs` 文件均以三段式 hicc 宏组织：

```rust
// 1. C++ 实现内联（hicc::cpp! 块）
// 2. 类方法绑定（hicc::import_class! 块，每个类一个）
// 3. 全局函数绑定（hicc::import_lib! 块）
```

---

## 3. 核心设计

### 3.1 数据结构

```rust
pub struct CompilationResult {
    pub classes:        Vec<ClassInfo>,
    pub functions:      Vec<FunctionInfo>,
    pub operators:      Vec<OperatorInfo>,
    pub friends:        Vec<FriendFnInfo>,
    pub lambdas:        Vec<LambdaInfo>,
    pub enums:          Vec<EnumInfo>,
    pub templates:      Vec<TemplateInstantiation>,
    pub vtables:        Vec<VtableInfo>,
}

/// Lambda 分类
pub enum LambdaKind {
    /// 无状态：退化为函数指针
    Stateless { fn_ptr_type: String },
    /// 有状态：封装为 class wrapper + opaque pointer
    Stateful  { wrapper_class: ClassInfo },
}

pub struct LambdaInfo {
    pub kind:           LambdaKind,
    pub capture_list:   Vec<CaptureItem>,
    pub source_loc:     SourceLocation,
    pub note:           Option<String>,  // 内联 TODO 注释内容
}

/// 内联 TODO 条目（简化版，不生成独立文件）
pub struct InlineTodo {
    pub tag:     TodoTag,
    pub message: String,
}

pub enum TodoTag {
    OperatorOverload,
    FriendFunction,
    Lambda,
    Rtti,
    VariadicTemplate,
}
```

### 3.2 代码格式：hicc 宏三段式

v4 的代码生成目标格式与 examples 完全一致：

```rust
hicc::cpp! {
    #include "foo.h"

    // shim 函数（由自动化工具生成）
    Foo* foo_new(int value) { return new Foo(value); }
    void foo_delete(Foo* self) { delete self; }
    int foo_getValue(Foo* self) { return self->getValue(); }

    // 运算符 shim（由 operator_handler 生成）
    Foo* foo_add(Foo* self, Foo* other) {
        return new Foo(self->operator+(*other));
    }
}

hicc::import_class! {
    #[cpp(class = "Foo")]
    class Foo {
        #[cpp(method = "int getValue() const")]
        fn getValue(&self) -> i32;
    }
}

hicc::import_lib! {
    #![link_name = "foo"]

    class Foo;

    #[cpp(func = "Foo* foo_new(int value)")]
    fn foo_new(value: i32) -> *mut Foo;

    #[cpp(func = "void foo_delete(Foo* self)")]
    unsafe fn foo_delete(self_: *mut Foo);

    #[cpp(func = "int foo_getValue(Foo* self)")]
    fn foo_getValue(self_: *mut Foo) -> i32;

    // 运算符 shim（自动生成，保持可编译）
    #[cpp(func = "Foo* foo_add(Foo* self, Foo* other)")]
    fn foo_add(self_: *mut Foo, other: *mut Foo) -> *mut Foo;
    // cpp2rust-todo[OP]: 可手动实现 std::ops::Add<Foo> for Foo
}
```

> **关键原则**：即使存在需要手动优化的特性，自动生成的代码也必须能通过 `cargo check`。TODO 以 `// cpp2rust-todo[TAG]: ...` 注释形式内联在代码中，不生成独立 JSON/Markdown 文件。

---

## 4. 后处理器设计（v4 修订版）

### 4.1 后处理器接口

```rust
pub trait PostProcessor {
    fn process(&self, result: &mut CompilationResult) -> Vec<InlineTodo>;
    fn priority(&self) -> u32;
}
```

### 4.2 运算符重载处理（对齐 examples/019）

**策略**：生成命名 shim 函数，放入 `hicc::cpp!{}` 块和 `import_lib!` 块；同时在 shim 条目下追加内联注释提示用户可手动实现 Rust trait。

| 运算符类型 | shim 命名规则 | 示例 |
|-----------|-------------|------|
| 算术 `+` `-` `*` `/` | `{class}_{add\|sub\|mul\|div}` | `foo_add(self, other)` |
| 复合赋值 `+=` `-=` | `{class}_{add\|sub}_assign` | `foo_add_assign(self, other)` |
| 一元 `-` | `{class}_negate` | `foo_negate(self)` |
| 前置 `++` `--` | `{class}_{inc\|dec}` | `foo_inc(self)` |
| 比较 `<=>` | `{class}_compare` | `foo_compare(self, other)` |
| 下标 `[]` | `{class}_index` | `foo_index(self, idx)` |

```rust
// 自动生成示例（对标 examples/019_operator_overload/rust_hicc/src/main.rs）
hicc::import_lib! {
    // ...
    #[cpp(func = "Number* number_add(Number* self, Number* other)")]
    fn number_add(self_: *mut Number, other: *mut Number) -> *mut Number;
    // cpp2rust-todo[OP]: 可实现 impl std::ops::Add<Number> for Number

    #[cpp(func = "void number_add_assign(Number* self, Number* other)")]
    fn number_add_assign(self_: *mut Number, other: *mut Number);
    // cpp2rust-todo[OP]: 可实现 impl std::ops::AddAssign<Number> for Number
}
```

### 4.3 友元函数处理（对齐 examples/020）

**策略**：友元函数在 C++ 侧本质上是普通函数（只是能访问私有成员），直接作为独立条目加入 `import_lib!`，与普通全局函数无语法差异。

```rust
// 自动生成示例（对标 examples/020_friend_function/rust_hicc/src/main.rs）
hicc::import_lib! {
    // ...
    #[cpp(func = "int friend_function_getSum(const MyClass* a, const MyClass* b)")]
    fn friend_function_getSum(a: *mut MyClass, b: *mut MyClass) -> i32;
    // cpp2rust-todo[FR]: 此函数是友元函数，可访问 MyClass 私有成员
}
```

> **注意**：友元函数的 shim 直接放入 `import_lib!`，无需单独的 `import_class!` 条目。

### 4.4 Lambda 处理（v4 完整修订，对齐 examples/039）

v4 将 lambda 分为两类，分别采用不同策略：

#### 4.4.1 无状态 Lambda → 函数指针

无状态 lambda（`[]` 空捕获列表）退化为 C 函数指针，可直接通过 FFI 传递。

```
C++ 输入:
  typedef int (*IntBinaryOp)(int, int);
  int apply_operation(int a, int b, IntBinaryOp op);

自动生成:
  hicc::cpp! {
      // 函数指针类型直接透传
      int apply_operation(int a, int b, int (*op)(int, int)) { ... }
  }
  hicc::import_lib! {
      type IntBinaryOp = extern "C" fn(i32, i32) -> i32;

      #[cpp(func = "int apply_operation(int, int, IntBinaryOp)")]
      fn apply_operation(a: i32, b: i32, op: IntBinaryOp) -> i32;
  }
```

#### 4.4.2 有状态 Lambda → Class Wrapper + Opaque Pointer

有状态 lambda（捕获外部变量）在 C++ 中本质是匿名函数对象，v4 的策略是将其检测为 `std::function<...>` 成员的宿主类，生成 opaque pointer 封装（即 `StateLambda` / `LambdaWrapper` 模式）。

```
C++ 输入:
  class StateLambda {  // 包含 std::function<int(int)> 成员
      StateLambdaImpl* impl;
  public:
      explicit StateLambda(int initial_value);
      ~StateLambda();
  };
  int state_lambda_apply(StateLambda* self, int delta);

自动生成:
  hicc::cpp! {
      // 直接 include 头文件，impl 细节不透出
      struct StateLambda* state_lambda_new(int initial_value) { ... }
      void state_lambda_delete(struct StateLambda* self) { ... }
      int state_lambda_apply(struct StateLambda* self, int delta) { ... }
  }
  hicc::import_lib! {
      class StateLambda;

      #[cpp(func = "StateLambda* state_lambda_new(int)")]
      fn state_lambda_new(initial: i32) -> *mut StateLambda;

      #[cpp(func = "void state_lambda_delete(StateLambda*)")]
      unsafe fn state_lambda_delete(self_: *mut StateLambda);

      #[cpp(func = "int state_lambda_apply(StateLambda*, int)")]
      fn state_lambda_apply(self_: *mut StateLambda, delta: i32) -> i32;
      // cpp2rust-todo[LM]: StateLambda 包含 std::function 成员，内部状态不透明
  }
```

> **v4 新增检测规则**：当 `CXXRecordDecl` 含有 `std::function<...>` 类型字段时，自动识别为 Stateful Lambda 宿主，直接生成 opaque pointer 封装，跳过内部字段生成。

#### 4.4.3 `std::bind` 升级为 ✅

`std::bind` 与有状态 lambda 的底层机制相同（均为函数对象），使用 4.4.2 的 class wrapper 策略完全覆盖（参见 `examples/041_functional_bind`）。v4 中 `std::bind` 从 v3 的 ⚠️ 升级为 ✅。

### 4.5 typeid/RTTI 处理（v4 修订，对齐 examples/023）

**v3 问题**：提出用 mangled name 字符串 `"N7DerivedE"` 做映射表，依赖 ABI，不同平台/编译器结果不同。

**v4 策略**：检测到 `typeid`/`dynamic_cast` 使用场景时，生成**整数枚举 + 虚函数**方案：

```
C++ 输入:
  // 使用了 typeid 或 dynamic_cast 的类体系
  class Shape { virtual ~Shape() = default; };
  class Circle : public Shape { ... };

自动生成:
  hicc::cpp! {
      // 工具在 C++ 侧注入类型枚举
      enum ShapeTypeTag {
          SHAPE_TAG_UNKNOWN  = -1,
          SHAPE_TAG_CIRCLE   = 0,
          SHAPE_TAG_RECTANGLE = 1,
      };

      // 为基类注入 getTypeTag() 虚函数（如原本不存在）
      int shape_getTypeTag(Shape* self) { return self->getTypeTag(); }
      const char* shape_getTypeName(Shape* self) { return self->getTypeName(); }
  }
  hicc::import_lib! {
      class Shape;

      #[cpp(func = "int shape_getTypeTag(Shape*)")]
      fn shape_getTypeTag(self_: *mut Shape) -> i32;

      #[cpp(func = "const char* shape_getTypeName(Shape*)")]
      fn shape_getTypeName(self_: *mut Shape) -> *const i8;
      // cpp2rust-todo[RTTI]: 枚举由工具生成，新增子类时需手动添加枚举值
  }
```

**检测条件**：当 AST 中存在 `CXXTypeidExpr` 或 `CXXDynamicCastExpr` 节点时触发 RTTI 后处理。

### 4.6 可变参数模板处理（对齐 examples/028）

**策略**：遍历 AST 中所有针对该模板的 `CallExpr` 调用点，收集实际调用的参数个数和类型组合，为每种组合生成一个固定元数版本。

| 调用点 | 生成函数 |
|--------|---------|
| `sum(1, 2)` | `fn sum_2(a: i32, b: i32) -> i32` |
| `sum(1, 2, 3)` | `fn sum_3(a: i32, b: i32, c: i32) -> i32` |
| `sum(1.5, 2.5)` | `fn sum_double_2(a: f64, b: f64) -> f64` |

```rust
hicc::import_lib! {
    #[cpp(func = "int sum_2(int, int)")]
    fn sum_2(a: i32, b: i32) -> i32;

    #[cpp(func = "int sum_3(int, int, int)")]
    fn sum_3(a: i32, b: i32, c: i32) -> i32;

    #[cpp(func = "double sum_double_2(double, double)")]
    fn sum_double_2(a: f64, b: f64) -> f64;
    // cpp2rust-todo[VA]: 可变参数模板，已展开检测到的 3 个调用点
}
```

---

## 5. 轻量 TODO 系统

### 5.1 设计原则

v3 的 `.todo.json` / `.todo.md` 双文件系统增加了工具实现复杂度，且 examples 证明所有特性均可生成**立即可编译**的降级代码。v4 改为：

- **内联注释**：`// cpp2rust-todo[TAG]: <message>` 追加在相关条目后
- **控制台摘要**：`init` 命令完成后在 stderr 打印 TODO 统计
- **无独立文件**：不生成 `.todo.json` 或 `.todo.md`

```
TAG 枚举:
  OP   = 运算符重载（可选实现 std::ops trait）
  FR   = 友元函数（已提取为独立函数）
  LM   = Lambda/std::function（有状态，class wrapper 封装）
  RTTI = typeid/RTTI（枚举注入，新增子类时需手动维护）
  VA   = 可变参数模板（固定元数展开）
```

### 5.2 控制台摘要格式

```
$ cpp2rust-ffi init -i ./cpp -o ./rust_hicc

[cpp2rust-ffi] Generated 3 files in rust_hicc/src/

  cpp2rust-todo summary:
    [OP]   5  operator shims generated (consider implementing std::ops traits)
    [FR]   3  friend functions extracted to import_lib!
    [LM]   2  stateful lambda wrappers (class wrapper pattern)
    [RTTI] 1  type tag enum injected (update enum when adding subclasses)
    [VA]   1  variadic template (3 call-site arities detected)

  All generated code passes `cargo check`. Grep for `cpp2rust-todo` to review.
```

---

## 6. 特性覆盖详情（48 个示例）

### 6.1 总览

| 类别 | 示例数 | ✅ 完全自动 | ⚠️ 降级生成（内联 TODO） | ❌ 不支持 |
|------|--------|------------|------------------------|---------|
| 基础类型与函数 | 5 | 5 | 0 | 0 |
| 类与对象 | 7 | 7 | 0 | 0 |
| 面向对象特性 | 6 | 6 | 0 | 0 |
| 运算符与类型 | 5 | 3 | 2 | 0 |
| 模板 | 5 | 4 | 1 | 0 |
| 智能指针与内存 | 5 | 5 | 0 | 0 |
| STL 容器 | 5 | 5 | 0 | 0 |
| 函数对象 | 4 | **3** | **1** | 0 |
| 其他高级特性 | 6 | 6 | 0 | 0 |
| **总计** | **48** | **44** | **4** | **0** |

> **v4 改进**：相比 v3（43 直接 + 5 后处理），v4 将 `std::bind`（041）从 ⚠️ 升级为 ✅，实现 44 直接 + 4 降级。

### 6.2 后处理特性详情（4 个 ⚠️）

| 特性 | 示例 | v3 处理 | v4 改进 | 内联 TODO tag |
|------|------|---------|---------|--------------|
| 运算符重载 | 019 | 降级为 FFI 函数 | ✅ 同策略，但格式对齐 hicc 宏 | `[OP]` |
| 友元函数 | 020 | 提取为独立函数 | ✅ 同策略，直接入 `import_lib!` | `[FR]` |
| typeid/RTTI | 023 | ❌ mangled name 字符串映射 | ✅ 整数枚举 + 虚函数注入 | `[RTTI]` |
| 有状态 Lambda | 039 | ⚠️ 仅部分处理 | ✅ fn ptr / class wrapper 双策略 | `[LM]` |

### 6.3 详细特性表（48 个示例）

#### 基础类型与函数 (1-5)

| 示例 | 特性 | AST 节点 | 支持 | 处理方式 |
|------|------|----------|------|---------|
| 001_hello_world | extern "C" 函数 | `FunctionDecl` | ✅ | 直接生成 import_lib! |
| 002_function_overload | 函数重载 | `FunctionDecl` (多个同名) | ✅ | 每个重载生成独立条目 |
| 003_default_args | 默认参数 | `ParmVarDecl` (带默认值) | ✅ | shim 包装默认值 |
| 004_inline_functions | 内联函数 | `FunctionDecl` + `inline` | ✅ | 内联到 hicc::cpp! 块 |
| 005_variadic_functions | 可变参数函数 | `FunctionDecl` (va_list) | ✅ | C variadic 直接映射 |

#### 类与对象 (6-12)

| 示例 | 特性 | AST 节点 | 支持 | 处理方式 |
|------|------|----------|------|---------|
| 006_class_basic | 基础类 | `CXXRecordDecl` | ✅ | opaque ptr + import_class! |
| 007_class_constructor | 构造/析构 | `CXXConstructorDecl` | ✅ | `*_new()` / `*_delete()` shim |
| 008_class_copy | 拷贝构造 | `CXXConstructorDecl` (copy) | ✅ | `*_copy()` shim |
| 009_class_move | 移动构造 | `CXXConstructorDecl` (move) | ✅ | `*_move()` shim |
| 010_class_static | 静态成员 | `VarDecl` (static) | ✅ | 静态访问 shim |
| 011_class_const | const 成员函数 | `CXXMethodDecl` (const) | ✅ | `&self` 绑定 |
| 012_class_volatile | volatile 成员函数 | `CXXMethodDecl` (volatile) | ✅ | 透传 volatile 语义 |

#### 面向对象特性 (13-18)

| 示例 | 特性 | AST 节点 | 支持 | 处理方式 |
|------|------|----------|------|---------|
| 013_inheritance_single | 单继承 | `CXXBaseSpecifier` | ✅ | 基类方法提升到派生类 import_class! |
| 014_inheritance_multiple | 多继承 | `CXXBaseSpecifier` (多个) | ✅ | 多继承链展开 |
| 015_virtual_basic | 虚函数 | `CXXMethodDecl` (virtual) | ✅ | vtable shim |
| 016_virtual_pure | 纯虚/抽象类 | `CXXMethodDecl` (= 0) | ✅ | Rust trait 接口 |
| 017_virtual_override | override | `CXXMethodDecl` (override) | ✅ | override 透传 |
| 018_virtual_diamond | 菱形继承 | `CXXBaseSpecifier` (virtual) | ✅ | virtual 继承展开 |

#### 运算符与类型 (19-23)

| 示例 | 特性 | AST 节点 | 支持 | 处理方式 |
|------|------|----------|------|---------|
| 019_operator_overload | 运算符重载 | `CXXMethodDecl` (operator) | ⚠️ | named shim + `[OP]` TODO |
| 020_friend_function | 友元函数 | `FriendDecl` | ⚠️ | 直接入 import_lib! + `[FR]` TODO |
| 021_explicit_ctor | explicit 构造 | `CXXConstructorDecl` (explicit) | ✅ | explicit 标记保留 |
| 022_mutable_member | mutable 成员 | `FieldDecl` (mutable) | ✅ | `&mut self` 访问函数 |
| 023_typeid_rtti | typeid/RTTI | `CXXTypeidExpr` | ⚠️ | 枚举注入 + `[RTTI]` TODO |

#### 模板 (24-28)

| 示例 | 特性 | AST 节点 | 支持 | 处理方式 |
|------|------|----------|------|---------|
| 024_template_function | 函数模板 | `FunctionTemplateDecl` | ✅ | 实例化时生成 |
| 025_template_class | 类模板 | `ClassTemplateDecl` | ✅ | 实例化时生成 |
| 026_template_specialization | 模板偏特化 | `ClassTemplatePartialSpecialization` | ✅ | 只处理实例化 |
| 027_template_instantiation | 显式实例化 | `ClassTemplateSpecialization` | ✅ | 捕获实例化 |
| 028_variadic_template | 可变参数模板 | `VariadicTemplate` | ⚠️ | 固定元数展开 + `[VA]` TODO |

#### 智能指针与内存 (29-33)

| 示例 | 特性 | AST 节点 | 支持 | 处理方式 |
|------|------|----------|------|---------|
| 029_unique_ptr | std::unique_ptr | `CXXNewExpr` | ✅ | hicc-smart-ptr 封装 |
| 030_shared_ptr | std::shared_ptr | `CXXNewExpr` | ✅ | hicc-smart-ptr 封装 |
| 031_custom_deleter | 自定义删除器 | `FunctionDecl` | ✅ | 删除器函数注入 |
| 032_placement_new | Placement new | `CXXNewExpr` | ✅ | placement new shim |
| 033_raii_pattern | RAII 模式 | 构造/析构 | ✅ | Drop trait 模式 |

#### STL 容器 (34-38)

| 示例 | 特性 | AST 节点 | 支持 | 处理方式 |
|------|------|----------|------|---------|
| 034_vector_basic | std::vector | `ClassTemplateSpecialization` | ✅ | hicc-std 封装 |
| 035_map_basic | std::map | `ClassTemplateSpecialization` | ✅ | hicc-std 封装 |
| 036_string_basic | std::string | `ClassTemplateSpecialization` | ✅ | hicc-std 封装 |
| 037_array_basic | std::array | `ClassTemplateSpecialization` | ✅ | hicc-std 封装 |
| 038_tuple_basic | std::tuple | `ClassTemplateSpecialization` | ✅ | hicc-std 封装 |

#### 函数对象 (39-42)

| 示例 | 特性 | AST 节点 | 支持 | 处理方式 |
|------|------|----------|------|---------|
| 039_lambda_basic | Lambda | `LambdaExpr` | ⚠️ | fn ptr / class wrapper 双策略 + `[LM]` TODO |
| 040_std_function | std::function | `ClassTemplateSpecialization` | ✅ | hicc-std 封装 |
| 041_functional_bind | std::bind | `CallExpr` | ✅ | class wrapper 模式（同 Lambda 有状态） |
| 042_exception_basic | 异常处理 | `CXXThrowExpr` | ✅ | 异常框架透传 |

#### 其他高级特性 (43-48)

| 示例 | 特性 | AST 节点 | 支持 | 处理方式 |
|------|------|----------|------|---------|
| 043_namespace_nested | 嵌套命名空间 | `NamespaceDecl` | ✅ | Rust mod 嵌套 |
| 044_enum_class | 强类型枚举 | `EnumDecl` (scoped) | ✅ | Rust enum |
| 045_union_basic | union | `RecordDecl` (union) | ✅ | Rust union |
| 046_constexpr_basic | constexpr | `Expr` (constexpr) | ✅ | const 常量 |
| 047_noexcept_basic | noexcept | `NoexceptSpec` | ✅ | 透传 noexcept |
| 048_summary | FFI 模式总结 | — | ✅ | 综合示例 |

**图例**：✅ 完全自动  ⚠️ 降级生成（含内联 TODO）

---

## 7. 实现计划

| 阶段 | 内容 | 优先级 | 覆盖 |
|------|------|--------|------|
| Phase 1 | AST 编译引擎（libclang 编译 .cpp，遍历 AST） | P0 | 所有特性基础 |
| Phase 2 | hicc 宏格式代码生成器（`hicc_codegen.rs`）| P0 | 格式对齐 |
| Phase 3 | 基础提取器（类/函数/枚举/虚函数表） | P0 | 001-018, 021-022, 024-027, 029-038, 042-048 |
| Phase 4 | 运算符重载后处理 | P1 | 019 |
| Phase 5 | 友元函数后处理 | P1 | 020 |
| Phase 6 | Lambda 后处理（fn ptr + class wrapper 双策略） | P1 | 039, 040, 041 |
| Phase 7 | RTTI 后处理（枚举注入） | P2 | 023 |
| Phase 8 | 可变参数模板后处理（固定元数展开） | P2 | 028 |
| Phase 9 | 控制台 TODO 摘要输出 | P1 | 全部 |
| Phase 10 | 集成测试：48 个示例全部通过 `cargo check` | P1 | 全部 |

---

## 8. 技术依赖

```toml
[dependencies]
clang = "2"            # libclang 绑定（v4 使用较新版本）
clap = "4"             # CLI
anyhow = "1"           # 错误处理
serde = { version = "1", features = ["derive"] }
serde_json = "1"       # AST JSON 解析

[build-dependencies]
cc = "1"               # C++ 编译器调用
```

```bash
# 系统依赖
apt-get install clang libclang-dev
apt-get install libstdc++-dev
```

---

## 9. v3 → v4 对比总结

| 维度 | v3 | v4 |
|------|----|----|
| 代码格式 | ❌ 裸 `extern "C"` 块 | ✅ `hicc::cpp!` + `import_class!` + `import_lib!` |
| Lambda 策略 | ⚠️ 仅简单 lambda | ✅ fn ptr / class wrapper 双策略覆盖全部场景 |
| RTTI 策略 | ❌ mangled name 字符串（ABI 不稳定） | ✅ 整数枚举 + 虚函数（跨平台稳定） |
| `std::bind` | ⚠️ 部分支持 | ✅ 完全支持（class wrapper 统一） |
| TODO 系统 | 独立 `.todo.json` + `.todo.md` 双文件 | 内联注释 `// cpp2rust-todo[TAG]:` |
| 可编译保证 | ⚠️ 部分特性可能悬空 | ✅ 所有生成代码通过 `cargo check` |
| 直接生成数 | 43/48 | **44/48** |
| 两阶段流程 | init（FFI + TODO 文件）→ merge | init（FFI + 内联注释）→ 可选精化 |
