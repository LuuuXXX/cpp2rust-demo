# 特性示例：右值引用方法（ref-qualifier）

## 背景

C++ 成员函数可以通过 **ref-qualifier** 区分调用时 `this` 的值类别：

| C++ 声明 | `this` 类型 | Rust 映射 |
|---------|------------|---------|
| `T foo() const &` / `const` 方法 | const lvalue | `fn foo(&self)` |
| `T foo() &` / 普通方法（不加 `const`） | mutable lvalue | `fn foo(&mut self)` |
| `T foo() &&` | rvalue（消费） | `fn foo(self)` |

`&&` 限定方法在语义上表示"消耗对象后返回结果"，cpp2rust-demo 将其
映射为 Rust 的 `self`（按值接收，消耗所有权）。

## 源码文件

- `rvalue_builder.hpp`：`Builder` 类，展示三种 ref-qualifier 变体
- `entry.cpp`：翻译单元入口

## 运行步骤

```bash
cpp2rust-demo init --feature feat03 --link builder \
    -- clang -x c++ -fsyntax-only examples/features/03-rvalue-ref/entry.cpp

cpp2rust-demo merge --feature feat03
cat .cpp2rust/feat03/rust/src/merged_ffi.rs
```

## 预期生成结果

```rust
hicc::import_class! {
    #[cpp(class = "Builder", ctor = "Builder(int)")]
    class Builder {
        // const lvalue → &self
        #[cpp(method = "int get() const")]
        fn get(&self) -> i32;

        // mutable lvalue → &mut self
        #[cpp(method = "void set(int)")]
        fn set(&mut self, v: i32);

        // rvalue-ref (&&) → self（消费对象）
        #[cpp(method = "int build() &&")]
        fn build(self) -> i32;

        // 普通 mutable 方法 → &mut self
        #[cpp(method = "void reset()")]
        fn reset(&mut self);
    }
}
```

## 关键结论

| C++ 方法 | Rust self 类型 |
|---------|--------------|
| `int get() const` | `&self` |
| `void set(int v)` | `&mut self` |
| `int build() &&` | `self`（消耗） |
| `void reset()` | `&mut self` |

> `&&` 限定方法在 Rust 侧对应 `self`，调用后对象所有权转移，不可再使用。
