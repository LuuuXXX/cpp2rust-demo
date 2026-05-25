# hicc-autogen

## 简介

`hicc-autogen` 是 hicc 生态中的**代码生成核心库**，负责将 Rust 源文件中的 hicc DSL（`hicc::import_lib!`、`hicc::import_class!`、`hicc::cpp!`）解析并转换为对应的 C++ 适配代码。它是 `hicc-build` 和 `hicc-macros` 的共同依赖。

## 模块结构

```text
hicc-autogen/src/
├── lib.rs           # 公开接口，重新导出所有子模块
├── attr.rs          # C++ 属性解析（#[cpp(func=...)], #[cpp(method=...)] 等）
├── class.rs         # class/interface 声明的 AST 表示
├── class_visitor.rs # class 块内部元素的访问者模式
├── cpp.rs           # hicc::cpp!{} 宏的 AST 表示与解析
├── function.rs      # 函数声明的 AST 表示与类型转换
├── import_class.rs  # hicc::import_class!{} 宏的完整解析与导出
├── import_lib.rs    # hicc::import_lib!{} 宏的完整解析与导出
├── utils.rs         # 公共工具函数（类型映射等）
├── visitor.rs       # 通用 Rust 语法访问者
└── export/          # C++ 代码生成（导出为字符串）
```

## 核心数据结构

### `ImportLib`（`import_lib.rs`）

对应 `hicc::import_lib!{}` 宏块的 AST 表示：

```rust
pub struct ImportLib {
    pub attrs: Vec<syn::Attribute>,  // #![link_name = "..."] 等内部属性
    pub funcs: Vec<ImportFn>,        // 函数声明列表
    pub items: Vec<syn::Item>,       // 原始 Rust item（透传）
    pub cpps: Vec<Cpp>,              // 内嵌的 hicc::cpp! 块
    pub decls: Vec<ClassDecl>,       // class 别名声明（class Foo = ...）
    pub hicc: syn::Path,             // hicc crate 路径
}
```

### `ImportClass`（`import_class.rs`）

对应 `hicc::import_class!{}` 宏块：

```rust
pub struct ImportClass {
    pub classes: Vec<Class>,         // 类定义列表
    pub decls: Vec<ClassDecl>,       // class 别名声明
}
```

### `Class`（`class.rs`）

单个类定义：

```rust
pub struct Class {
    pub attrs: Vec<syn::Attribute>,  // #[cpp(class = "...")], #[interface] 等
    pub ident: syn::Ident,           // Rust 类型名
    pub generics: Option<Generics>,  // 泛型参数（<T>）
    pub intf: Option<syn::Ident>,    // 基类（: Base）
    pub methods: Vec<ClassMethod>,   // 方法列表
    pub cpps: Vec<Cpp>,              // 内嵌 hicc::cpp! 块
}
```

### `Cpp`（`cpp.rs`）

`hicc::cpp!{}` 宏块，内容为 C++ 代码的 token stream：

```rust
pub struct Cpp {
    pub tokens: proc_macro2::TokenStream,  // C++ 代码内容
}
```

## 属性系统（`attr.rs`）

支持以下 `#[cpp(...)]` 属性：

| 属性 | 用途 | 示例 |
|------|------|------|
| `func = "..."` | 声明 C++ 全局函数或模板函数 | `#[cpp(func = "int add(int, int)")]` |
| `method = "..."` | 声明 C++ 成员函数 | `#[cpp(method = "void foo() const")]` |
| `class = "..."` | 声明对应的 C++ 类类型 | `#[cpp(class = "std::string")]` |
| `class = "...", ctor = "..."` | 声明类和构造函数 | `#[cpp(class = "Foo", ctor = "Foo()")]` |
| `class = "...", destroy = "..."` | 声明私有析构的类 | `#[cpp(class = "Bar", destroy = "Bar::Delete")]` |
| `field = "..."` | 声明 C++ 成员变量访问 | `#[cpp(field = "int value")]` |
| `data = "..."` | 声明 C++ 全局/静态变量访问 | `#[cpp(data = "Counter::count")]` |

特殊标记：

| 属性 | 用途 |
|------|------|
| `#[interface]` | 声明纯虚接口类（生成 Rust trait） |
| `#[interface(name = "Foo")]` | 与 `@make_proxy` 配合实现接口 |
| `#[method(class = Foo, name = bar)]` | 将全局工厂函数绑定为类的关联函数 |

## C++ 代码导出（`export/`）

`ExportLib` 和 `ExportClasses` 将解析后的 AST 转换为 C++ 代码字符串：

```rust
// hicc-build 中的调用方式
let lib = syn::parse2::<ImportLib>(tokens)?;
let export = ExportLib::try_from(lib)?;
let cpp_code: String = export.export()?;
```

生成的 C++ 代码包含：
1. 来自 `hicc::cpp!{}` 的内联 C++ 代码
2. 函数包装器（处理默认参数、返回值省略、异常捕获等）
3. 类成员函数的适配代码
4. `@make_proxy` 代理类生成
5. `@dynamic_cast` 转换辅助

## 函数类型映射（`function.rs`）

`hicc-autogen` 自动处理以下 C++ → Rust 类型适配：

| C++ 签名特性 | 生成的适配逻辑 |
|-------------|--------------|
| 缺省参数 | Rust 函数参数可少于 C++ 参数 |
| 忽略返回值 | Rust 函数可声明为 `()` 返回 |
| `T&&` 参数 | 等同于按值传递 |
| `std::function<R(A)>` | 通过 `hicc::Function<fn(A)->R>` 传递 |
| `va_list` | 生成变长参数函数指针 |
| 异常包装 | `hicc::Exception<T>` 包装返回值 |
| 引用返回 | 自动适配为 `ClassRef<'_, T>` |

## 使用方式

`hicc-autogen` 通常不直接使用，而是通过 `hicc-build` 或 `hicc-macros` 间接使用：

```toml
# Cargo.toml (build-dependencies)
[build-dependencies]
hicc-build = "0.2"  # 内部依赖 hicc-autogen
```

如需直接使用（例如自定义代码生成器）：

```rust
use hicc_autogen::{ImportLib, ImportClass, Cpp, ExportLib, ExportClasses};

let lib = syn::parse2::<ImportLib>(tokens)?;
let cpp_code = ExportLib::try_from(lib)?.export()?;
```

## 注意事项

1. **解析错误处理**：解析错误通过 `syn::Error` 传递，`hicc-build` 会将错误位置（文件:行:列）打印后 panic
2. **`in_hicc` 属性**：内部使用，表示宏在 hicc crate 内部，此时 `hicc` 路径为 `crate` 而非 `::hicc`
3. **Speculative 解析**：使用 `syn` 的 `Speculative` trait 进行尝试性解析（先 fork，成功后 advance），避免消耗 token stream
