# 特性示例：静态类数据成员

## 背景

C++ 类的 `static` 数据成员是类作用域内的全局变量。cpp2rust-demo 将它们提取为
`#[cpp(data = "ClassName::member")]` 形式的绑定，并放入 `import_lib!` 块。

- **可变 static** → `fn member() -> &'static mut T`
- **const static** → `fn member() -> &'static T`

与普通全局变量的区别：静态成员使用全限定名 `ClassName::member`，
而非普通全局的 `namespace::var`。

## 源码文件

- `static_members.hpp`：`Counter` 类，含静态可变成员 `instance_count` 和 const 成员 `max_count`
- `entry.cpp`：翻译单元入口

## 运行步骤

```bash
cpp2rust-demo init --feature feat06 --link counter \
    -- clang -x c++ -fsyntax-only examples/features/06-static-members/entry.cpp

cpp2rust-demo merge --feature feat06
cat .cpp2rust/feat06/rust/src/lib.rs
```

## 预期生成结果

```rust
hicc::import_lib! {
    #![link_name = "counter"]

    // 可变静态成员 → &'static mut T
    #[cpp(data = "Counter::instance_count")]
    fn instance_count() -> &'static mut i32;

    // const 静态成员 → &'static T（只读）
    #[cpp(data = "Counter::max_count")]
    fn max_count() -> &'static i32;
}

hicc::import_class! {
    #[cpp(class = "Counter", ctor = "Counter()")]
    class Counter {
        #[cpp(method = "void increment()")]
        fn increment(&mut self);

        #[cpp(method = "int get() const")]
        fn get(&self) -> i32;
    }
}
```

## 接口报告

`meta/init-interface-report.md` 的 `## Static Data Members` 部分会列出所有提取到的静态成员。

## 关键结论

| C++ 成员 | Rust 绑定 | 位置 |
|---------|---------|------|
| `static int instance_count` | `fn instance_count() -> &'static mut i32` | `import_lib!` (free/) |
| `static const int max_count` | `fn max_count() -> &'static i32` | `import_lib!` (free/) |

> 静态成员使用全限定名（`ClassName::member`），与普通全局变量（`namespace::var`）不同。
