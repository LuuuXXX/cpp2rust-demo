# 020_friend_function - 友元函数（hicc 直出，无 shim）

## C++ 特性

本示例展示**友元函数**的地道 C++ 命名空间类 `MyClass`：自由函数 `getSum`/`getProduct`/
`compare` 被声明为 `MyClass` 的友元，因而可以访问其私有成员 `value_`。友元在类体内**内联
定义**，经 ADL（参数依赖查找）在命名空间内调用。用 hicc 直出绑定，**无 opaque 指针、无
`extern "C"` 桥接、无 `*_new`/`*_delete` shim**。

## C++ 代码（节选）

```cpp
namespace friend_function_ns {

class MyClass {
public:
    explicit MyClass(int v);
    int getValue() const;
    void setValue(int v);

    // 非成员友元：可访问私有成员 value_
    friend int getSum(const MyClass& a, const MyClass& b) {
        return a.value_ + b.value_;
    }
    friend int getProduct(const MyClass& a, const MyClass& b) { /* ... */ }
    friend int compare(const MyClass& a, const MyClass& b)    { /* ... */ }
private:
    int value_;
};

} // namespace friend_function_ns
```

配套 `standalone.sh` / `Makefile` 两种纯 C++ 构建方式。

## Rust FFI 代码（hicc 直出 + 友元命名包装）

hicc 直出只绑定类的公有成员方法 `getValue()`/`setValue()` 与构造工厂：默认支架
（`lib_scaffold.rs`）即由 `init` 生成。友元是非成员自由函数，不进 `import_class!`；
在手写 `lib.rs` 中以 `hicc::cpp!` 命名包装函数补全——每个友元包成一个具名 C++ 函数
（经 ADL 调用真实友元），再用 `#[cpp(func = ...)]` 绑定为 `MyClass` 的关联方法：

```rust
hicc::cpp! {
    #include "friend_function.h"
    using friend_function_ns::MyClass;
    int myclass_friend_sum(const MyClass* self, const MyClass& other) {
        return getSum(*self, other);   // 经 ADL 调用真实友元
    }
    // ... product / compare 同理
}

// import_class! 内：
#[cpp(func = "int myclass_friend_sum(const friend_function_ns::MyClass*, const friend_function_ns::MyClass&)")]
pub fn friend_sum(&self, other: &MyClass) -> i32;
```

## 关键点

| C++ 概念 | hicc 直出映射 |
|----------|--------------|
| 公有成员方法 `getValue`/`setValue` | 直接绑定（`const` → `&self`，非 `const` → `&mut self`） |
| 构造函数 `MyClass(int)` | `hicc::make_unique` 工厂 → 关联函数 `MyClass::new` |
| 友元自由函数 `getSum` 等 | 跳过自动绑定 → `hicc::cpp!` 命名包装 + `#[cpp(func)]` 关联方法 |
| 访问私有成员 | 由 C++ 侧友元完成；Rust 侧只调用包装函数 |
| ADL 查找 | 包装函数以 `MyClass` 实参触发命名空间内友元查找 |

## 构建方法

```bash
cd cpp && ./standalone.sh        # 纯 C++ 独立验证（或 make run）
cd rust_hicc && cargo test       # 行为级 smoke 断言
```

## 运行结果

```
a.get_value()=10
Friend getSum: 10 + 3 = 13
getSum(a,b)=13
Friend getProduct: 10 * 3 = 30
getProduct(a,b)=30
compare(a,b)=1
compare(c,b) after set_value=0
--- end main ---
Friend compare: a > b
Friend compare: a == b
```

## 总结

1. **友元跳过**：hicc 直出不自动绑定非成员自由函数（含友元）。
2. **命名包装**：以 `hicc::cpp!` 具名函数补全友元，绑定为关联方法。
3. **私有访问**：访问私有成员的逻辑留在 C++ 侧友元，Rust 仅做调用。
4. **去 shim**：无 `*_new`/`*_delete`、无 opaque 指针、无 `extern "C"` 桥接。
