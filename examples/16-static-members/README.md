# 示例 16：静态成员

## 特性概述

本示例展示 C++ 的**静态成员**，包括静态成员变量（类级别状态）、静态成员函数（类级别操作）、静态常量以及单例模式。静态成员在 Rust FFI 中通过 `import_lib!` 中的类方法声明来处理。

## C++ 特性说明

| 特性 | 说明 |
|------|------|
| 静态成员变量 | `static int total_count` 类所有实例共享 |
| 静态成员函数 | `static int get_total_count()` 无需实例调用 |
| 静态常量 | `static const int MAX_COUNT = 1000` |
| 单例模式 | 通过静态成员实现唯一实例 |
| 类外定义 | `int Counter::total_count = 0` |

### 代码结构

```cpp
class Counter {
    int count;
    static int total_count;           // 静态成员变量
    static const int MAX_COUNT = 1000; // 静态常量

public:
    static int get_total_count();     // 静态成员函数
    static int get_max_count();
    static void reset_total();
    void increment();
    int get() const;
};

// 单例
class Singleton {
    static Singleton* instance;
    static Singleton& get_instance();
};
```

## AST JSON 结构要点

运行以下命令生成 `ast.json`：

```bash
clang++ -Xclang -ast-dump=json -fsyntax-only main.cpp > ast.json
```

关键 AST 节点与属性：

| AST 节点 / 属性 | 含义 |
|-----------------|------|
| `CXXMethodDecl.isStatic: true` | 静态成员函数 |
| `VarDecl.isStaticMember: true` | 静态成员变量 |
| `VarDecl.isConst: true` | 静态常量 |

AST 片段示例：

```json
{
  "kind": "CXXMethodDecl",
  "name": "get_total_count",
  "isStatic": true,
  "type": { "qualType": "int ()" }
}
```

## hicc 处理方式

### 静态成员函数 → `import_lib!`

静态成员函数通过 `import_lib!` 映射，不需要类实例：

```rust
hicc::import_class! {
    #[cpp(class = "Counter")]
    class Counter {
        // 非静态方法
        #[cpp(method = "void increment()")]
        fn increment(&mut self);

        #[cpp(method = "int get() const")]
        fn get(&self) -> i32;
    }
}

hicc::import_lib! {
    #![link_name = "example"]

    class Counter;

    // 构造函数
    #[cpp(func = "std::unique_ptr<Counter> std::make_unique<Counter, int>(int&&)")]
    fn counter_new(initial: i32) -> Counter;

    // 静态成员函数 → 通过类限定名声明
    #[cpp(func = "int Counter::get_total_count()")]
    fn counter_get_total() -> i32;

    #[cpp(func = "int Counter::get_max_count()")]
    fn counter_get_max() -> i32;

    #[cpp(func = "void Counter::reset_total()")]
    fn counter_reset_total();
}
```

在 Rust 侧可以将静态方法组织为关联函数：

```rust
impl Counter {
    pub fn total_count() -> i32 { counter_get_total() }
    pub fn max_count() -> i32 { counter_get_max() }
    pub fn reset_total() { counter_reset_total() }
}
```

### 静态成员变量的访问

静态成员变量通过 `[cpp(data = ...)]` 声明：

```rust
hicc::import_lib! {
    #![link_name = "example"]

    // 读写静态成员变量
    #[cpp(data = "Counter::total_count")]
    fn counter_total_count() -> &'static i32;

    #[cpp(data = "Counter::total_count")]
    fn counter_total_count_mut() -> &'static mut i32;
}
```

### 单例模式

```rust
hicc::import_class! {
    #[cpp(class = "Singleton")]
    class Singleton {
        #[cpp(method = "void do_something()")]
        fn do_something(&mut self);
    }
}

hicc::import_lib! {
    #![link_name = "example"]

    class Singleton;

    // 静态工厂方法返回引用（生命周期静态）
    #[cpp(func = "Singleton& Singleton::get_instance()")]
    fn singleton_get_instance() -> ClassRefMut<'static, Singleton>;
}

fn main() {
    let instance = singleton_get_instance();
    instance.do_something();
}
```

### 静态常量访问

静态编译期常量可直接通过 `import_lib!` 的 `data` 属性读取：

```rust
hicc::import_lib! {
    #![link_name = "example"]

    #[cpp(data = "Counter::MAX_COUNT")]
    fn counter_max_count() -> &'static i32;
}
```

## cpp2rust-demo 处理策略

cpp2rust-demo 通过 AST 中的 `isStatic: true` 标志识别静态成员函数，并将其归入 `import_lib!` 而非 `import_class!` 的方法列表，因为静态方法不需要类实例（无 `this` 指针）。

## 注意事项

1. **静态函数调用语法**：C++ 静态函数使用 `Counter::get_total_count()` 语法，Rust 侧需在 `#[cpp(func = ...)]` 中包含类名前缀
2. **单例生命周期**：`'static` 生命周期表示引用在程序运行期间始终有效
3. **线程安全**：静态成员变量在多线程环境下需要同步（`std::mutex` 等），Rust 侧需要相应的 `Mutex` 包装
4. **静态变量初始化顺序**：C++ 静态变量的初始化顺序未定义（跨翻译单元），需注意全局对象间的依赖
