# 017_virtual_override - override 说明符

## C++ 特性

本示例展示 C++ `override` 说明符的 FFI 映射。

## C++ 代码

### virtual_override.h

```cpp
class Base {
public:
    virtual double area() = 0;
    virtual const char* getName() = 0;
    virtual ~Base() {}
};

class Derived : public Base {
    double value;
public:
    Derived(double v) : value(v) {}
    double area() override {  // override 明确表示覆盖
        return value * 2;
    }
    const char* getName() override {
        return "Derived";
    }
};
```

## override 说明符

### C++11 override

```cpp
class Derived : public Base {
    double area() override {  // 如果不覆盖基类虚函数，编译错误
        return value * 2;
    }
};
```

`override` 的作用：
1. **编译时检查**：确保确实覆盖了基类方法
2. **文档作用**：明确代码意图
3. **防止错误**：拼写错误或签名不匹配会导致编译错误

### 没有 override 的风险

```cpp
class Derived : public Base {
    double area(double x) override {  // 错误！这是重载，不是覆盖
        return value * x;
    }
};
// 编译错误：没有虚函数可覆盖
```

## FFI 映射

### override 不影响 FFI

`override` 是编译时说明符，不生成额外代码：

```cpp
// 无论是否有 override，vtable 条目相同
class Base { virtual double area(); };
class Derived : public Base { double area() override; };

// vtable 布局相同
```

### FFI 中的 override

```c
// C FFI 中看不到 override
struct Derived {
    void** vtable;  // 与 Base 相同的布局
    double value;
};
```

### Rust FFI

```rust
// override 对 Rust FFI 透明
#[cpp(func = "double base_area(struct Base*)")]
unsafe fn base_area(self_: *mut Base) -> f64;
```

## 关键点

### override vs virtual

| 说明符 | 作用 | FFI 影响 |
|--------|------|----------|
| virtual | 声明虚函数 | 创建 vtable 条目 |
| override | 确保覆盖基类 | 编译时检查，无 FFI 影响 |

### 为什么使用 override

1. **早期错误检测**：拼写错误立即报错
2. **维护性**：基类改变时，派生类会报错
3. **可读性**：明确意图

### FFI 注意事项

```cpp
// C++ 中 override 是可选的
class Derived : public Base {
    double area() override { ... }  // 显式
    double area() { ... }          // 隐式（同样有效）
};

// FFI 只看到结果：vtable 中是 Derived::area
```


## Rust FFI 代码

```rust
hicc::cpp! {
    #include <iostream>
    #include <cstring>
    #include <string>

    #include "virtual_override.h"
}

hicc::import_class! {
    #[cpp(class = "Base", destroy = "base_delete")]
    pub class Base {
        #[cpp(method = "double area() const")]
        fn area(&self) -> f64;

        #[cpp(method = "const char* getName() const")]
        fn get_name(&self) -> *const i8;
    }
}

hicc::import_class! {
    #[cpp(class = "Derived", destroy = "derived_delete")]
    pub class Derived {
        #[cpp(method = "const char* getName() const")]
        fn get_name(&self) -> *const i8;

        #[cpp(method = "double area() const")]
        fn area(&self) -> f64;

        #[cpp(method = "double getValue() const")]
        fn get_value(&self) -> f64;
    }
}

hicc::import_lib! {
    #![link_name = "virtual_override"]

    class Base;
    class Derived;

    #[cpp(func = "Base* base_create(int)")]
    fn base_create(type_: i32) -> Base;

    #[cpp(func = "Derived* derived_new(double)")]
    fn derived_new(value: f64) -> Derived;
}
```

## 运行结果

```
=== Virtual Override FFI with hicc ===

The 'override' keyword explicitly marks method overriding in C++

Creating Base
Creating Derived (as Base*)
--- Calling through Base pointer ---
Name: Base
Area: 0.0000

--- Calling through Derived (as Base*) ---
Name: Derived
Area: 1764.0000

override ensures Derived::area() is called not Base::area()
This is polymorphism: same interface, different implementations

Rust FFI: override keyword works correctly through hicc!
```

## 冒烟测试

本示例包含集成冒烟测试（`rust_hicc/tests/smoke.rs`），验证生成的 Rust FFI 绑定可编译、
可链接 C++ 实现，且基本行为正确。

### 测试用例

| 测试函数 | 验证内容 |
|---------|---------|
| `smoke_derived_get_value` | `derived_new(3.14)` 后 `get_value()` 返回 3.14 |
| `smoke_derived_area` | `derived_new(2.0)` 后 `area()` 可正常调用（override 路径） |
| `smoke_base_create` | `base_create(0)` 返回 Base 实例，`area()` 与 `get_name()` 均可调用 |

### 运行方式

```bash
cd examples/017_virtual_override/rust_hicc
cargo test --test smoke
```

### 各平台支持

| 平台 | 状态 | 备注 |
|------|------|------|
| Linux (Ubuntu) | ✅ | CI `l-smoke` job 已覆盖 |
| macOS | ✅ | 支持 |
| Windows MinGW | ✅ | 支持 |

## 总结

1. **override**：确保派生类方法覆盖基类虚函数
2. **编译时检查**：签名不匹配导致编译错误
3. **FFI 影响**：无，override 不生成额外代码
4. **vtable**：无论是否使用 override，vtable 布局相同
