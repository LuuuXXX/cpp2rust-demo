# 019_operator_overload - 运算符重载（hicc 直出，无 shim）

## C++ 特性

本示例展示**运算符重载**的地道 C++ 命名空间类 `Number`：算术（`+ - * /`）、一元负号、
前置自增/自减（`++`/`--`）、复合赋值（`+= -=`）以及普通比较方法 `compare()`。用 hicc
直出绑定，**无 opaque 指针、无 `extern "C"` 桥接**。

## C++ 代码（节选）

```cpp
namespace operator_overload_ns {

class Number {
public:
    explicit Number(int v);
    int value() const;
    Number operator+(const Number& other) const;   // 算术
    Number operator-(const Number& other) const;
    Number operator*(const Number& other) const;
    Number operator/(const Number& other) const;
    int compare(const Number& other) const;          // 普通方法
    Number operator-() const;                         // 一元
    Number& operator++();                             // 前置 ++
    Number& operator--();
    Number& operator+=(const Number& other);          // 复合赋值
    Number& operator-=(const Number& other);
private:
    int value_;
};

} // namespace operator_overload_ns
```

配套 `standalone.sh` / `Makefile` 两种纯 C++ 构建方式。

## Rust FFI 代码（hicc 直出 + 运算符命名包装）

hicc 直出**跳过 `operator` 重载**：默认支架（`lib_scaffold.rs`）仅含普通方法
`value()`/`compare()` 与构造工厂。运算符在手写 `lib.rs` 中以 `hicc::cpp!` 命名包装
函数补全——每个运算符包成一个具名 C++ 函数（返回 `std::unique_ptr<Number>` 或就地
修改 `*self`），再用 `#[cpp(func = ...)]` 绑定为 `Number` 的关联方法：

```rust
hicc::cpp! {
    #include "operator_overload.h"
    #include <memory>
    using operator_overload_ns::Number;
    std::unique_ptr<Number> number_op_add(const Number* self, const Number& other) {
        return std::make_unique<Number>(*self + other);
    }
    void number_op_inc(Number* self) { ++(*self); }
    // ... sub/mul/div/neg/dec/+=/-= 同理
}

// import_class! 内：
#[cpp(func = "std::unique_ptr<operator_overload_ns::Number> number_op_add(const operator_overload_ns::Number*, const operator_overload_ns::Number&)")]
pub fn op_add(&self, other: &Number) -> Number;
#[cpp(func = "void number_op_inc(operator_overload_ns::Number*)")]
pub fn increment(&mut self);
```

## 关键点

| C++ 概念 | hicc 直出映射 |
|----------|--------------|
| `operator+ - * /` | 跳过自动绑定 → `hicc::cpp!` 命名包装 + `#[cpp(func)]` 关联方法 `op_add` 等 |
| 一元 `operator-` | 包装 `number_op_neg` → `op_neg(&self) -> Number` |
| `++`/`--` | 就地修改 `*self` → `increment`/`decrement`（`&mut self`） |
| `+=`/`-=` | 就地修改 → `add_assign`/`sub_assign`（`&mut self`） |
| 普通方法 `compare` | 直接绑定（`const Number&` → `&Number`） |
| 返回新对象 | 包装函数返回 `std::unique_ptr<Number>`，hicc 映射为按值 `Number` |

## 构建方法

```bash
cd cpp && ./standalone.sh        # 纯 C++ 独立验证（或 make run）
cd rust_hicc && cargo test       # 行为级 smoke 断言
```

## 总结

1. **运算符跳过**：hicc 直出不自动绑定 `operator` 重载。
2. **命名包装**：以 `hicc::cpp!` 具名函数补全运算符，绑定为关联方法。
3. **返回新对象**：包装函数返回 `unique_ptr<Number>`，映射为按值 `Number`。
4. **就地修改**：`++`/`--`/`+=`/`-=` 以 `&mut self` 包装实现。
