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

## 总结

1. **override**：确保派生类方法覆盖基类虚函数
2. **编译时检查**：签名不匹配导致编译错误
3. **FFI 影响**：无，override 不生成额外代码
4. **vtable**：无论是否使用 override，vtable 布局相同
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
