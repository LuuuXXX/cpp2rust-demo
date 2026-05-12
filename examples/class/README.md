# 类示例：包含成员方法的 C++ 类

该示例演示如何从 C++ 类声明/实现生成 Rust FFI。  
本示例已扩展为同时覆盖：**构造函数**、**virtual/纯虚方法**、**public 继承**、**静态方法**，
以反映 cpp2rust-demo 当前实际支持的全部类特性。

## 源码文件

- `widget.hpp`：类声明（抽象基类 `Shape` + 继承自 `Shape` 的具体类 `Widget`）
- `widget.cpp`：类实现

## 运行步骤

在仓库根目录执行：

```bash
# 第 1 步：生成分组 FFI（用独立 feature，避免和其他示例混合）
cpp2rust-demo init --feature widget --link widget -- clang -x c++ -fsyntax-only examples/class/widget.cpp

# 第 2 步：合并
cpp2rust-demo merge --feature widget

# 第 3 步：查看结果
cat .cpp2rust/widget/rust/src/merged_ffi.rs
```

> 说明：交互终端下，`init` 会提示选择要纳入转换的中间件文件（`Space` 勾选，`Enter` 确认）；非交互环境会自动全选。

## 预期生成结果

### 抽象基类 `Shape`（全纯虚 → `#[interface]`）

```rust
// method/mtd_widget.rs
hicc::import_class! {
    #[interface]
    class Shape {
        #[cpp(method = "double area() const")]
        fn area(&self) -> f64;

        #[cpp(method = "double perimeter() const")]
        fn perimeter(&self) -> f64;

        #[cpp(method = "const char * name() const")]
        fn name(&self) -> *const i8;
    }
}
```

### 具体类 `Widget`（继承 `Shape`，有构造函数 + 方法）

```rust
hicc::import_class! {
    #[cpp(class = "Widget", ctor = "Widget(int)")]
    class Widget: Shape {
        #[cpp(method = "double area() const")]
        fn area(&self) -> f64;

        #[cpp(method = "double perimeter() const")]
        fn perimeter(&self) -> f64;

        #[cpp(method = "const char * name() const")]
        fn name(&self) -> *const i8;

        #[cpp(method = "void update(double, double)")]
        fn update(&mut self, x: f64, y: f64);

        #[cpp(method = "int getId() const")]
        fn get_id(&self) -> i32;

        #[cpp(method = "bool isVisible() const")]
        fn is_visible(&self) -> bool;

        #[cpp(method = "void setVisible(bool)")]
        fn set_visible(&mut self, v: bool);
    }
}
```

### 静态方法与前置声明进入 `import_lib!`

```rust
// free/fn_widget.rs
hicc::import_lib! {
    #![link_name = "widget"]

    class Widget;

    #[cpp(func = "int Widget::instanceCount()")]
    fn widget_instance_count() -> i32;
}
```

### `@make_proxy`（为 `Shape` 接口生成 Rust 实现桩）

```rust
// free/fn_widget.rs（续）
hicc::import_lib! {
    #![link_name = "widget"]
    // ...
    #[cpp(func = "Shape @make_proxy<Shape>()")]
    #[interface(name = "Shape")]
    fn new_shape_proxy(intf: hicc::Interface<Shape>) -> Shape;
}
```

## hicc 中各类绑定的映射规则

| C++ 构造 | 生成位置 | hicc 语法 |
|----------|---------|----------|
| 全纯虚类 | `method/` | `#[interface] class Foo { ... }` |
| 普通/virtual 方法 | `method/` | `#[cpp(method = "...")]` |
| `const` 方法 | `method/` | `fn foo(&self)` |
| 非 `const` 方法 | `method/` | `fn foo(&mut self)` |
| 构造函数（主） | `method/` | `#[cpp(class = "Foo", ctor = "...")]` |
| 额外构造函数 | `free/` | `import_lib!` + `class Foo;` forward decl |
| `static` 方法 | `free/` | `#[cpp(func = "T Foo::bar(...)")]` |
| 公有基类 | `method/` | `class Foo: Base` 继承语法 |
| `@make_proxy` | `free/` | `#[interface(name = "...")] fn new_foo_proxy(...)` |
| 析构函数 | — | 跳过（hicc 限制，见下方能力表） |
| `operator` 重载 | `free/shim_ops.rs` | 生成 C++ shim starter，需手写实现 |

## 当前能力全览

| 能力项 | 状态 | 说明 |
|---------|--------|------|
| public 实例方法（非 virtual） | ✅ 已支持 | `#[cpp(method = "...")]` |
| `const` 方法 | ✅ 已支持 | 映射为 `&self` |
| `static` 方法 | ✅ 已支持 | 进入 `import_lib!` |
| 构造函数 | ✅ 已支持 | 主构造函数 `ctor="..."`；其余为工厂函数 |
| 非纯 `virtual` 方法 | ✅ 已支持 | hicc 通过 vtable 透明调用，Rust 端无感知 |
| 全纯虚类（抽象接口） | ✅ 已支持 | 生成 `#[interface]` trait + `@make_proxy` 绑定 |
| 混合类（纯虚 + 普通方法） | ✅ 已支持 | 普通方法正常提取；纯虚方法生成 companion interface |
| public 继承 | ✅ 已支持 | `class Foo: Base` 语法，支持 upcasting |
| `typedef`/`using` 别名解锁模板 | ✅ 已支持 | 见 `rapidjson-02` 和 `rapidjson-03` 示例 |
| 运算符重载 | ⚠️ 半自动 | 生成 `operator_shims.hpp` starter；需手写 C++ 实现 |
| 析构函数 | ❌ 跳过（hicc 限制） | hicc 不支持显式析构，交由对象生命周期管理 |
| private/protected 成员 | ✅ 自动跳过 | 不进入输出 |
| 模板类（无别名） | ⚠️ 跳过（ToolConservative） | 添加 `typedef`/`using` 别名后可解锁 |
| 多重继承 | ❌ 暂不支持 | 仅处理首个 public 基类 |
