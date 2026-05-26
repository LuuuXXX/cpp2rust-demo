# 014_inheritance_multiple - 多继承

## C++ 特性

本示例展示 C++ 多继承的 FFI 映射。

## C++ 代码

### inheritance_multiple.h

```cpp
// Base1 基类
struct Base1 {
    int value1;
};

// Base2 基类
struct Base2 {
    int value2;
};

// Derived 派生类 (多继承：继承自 Base1 和 Base2)
struct Derived {
    struct Base1 base1;  // 第一个基类
    struct Base2 base2;  // 第二个基类
    int derived_value;
};
```

### C++ 多继承

```cpp
class Base1 {
public:
    int value1;
    int getValue1();
};

class Base2 {
public:
    int value2;
    int getValue2();
};

class Derived : public Base1, public Base2 {
public:
    int derived_value;
    int getDerivedValue();
    void compute();
};
```

## 多继承与 FFI

### 多个基类成员

在 FFI 中，多继承通过多个基类成员实现：

```cpp
// C++ 多继承
class Derived : public Base1, public Base2 { };

// FFI：每个基类作为结构体成员
struct Derived {
    struct Base1 base1;  // 第一个基类
    struct Base2 base2;  // 第二个基类
    int derived_value;
};
```

### 挑战

1. **多个基类指针**：多继承中，Derived* 可以转换为 Base1* 或 Base2*
2. **不同偏移**：Base1 在 offset 0，Base2 在 `sizeof(Base1)` 偏移
3. **方法调度**：需要知道从哪个基类开始计算方法偏移

### FFI 映射

| C++ 场景 | FFI 挑战 |
|----------|----------|
| `Derived*` 到 `Base1*` | 直接使用指针（Base1 在 offset 0） |
| `Derived*` 到 `Base2*` | 需要计算 `Base2` 的偏移 |
| 调用 `Base2` 方法 | 函数需要处理正确的偏移 |

## Rust FFI 代码

```rust
// 多继承：多个基类成员
#[cpp(func = "int derived_getValue1(struct Derived*)")]
unsafe fn derived_getValue1(self_: *mut Derived) -> i32;

#[cpp(func = "int derived_getValue2(struct Derived*)")]
unsafe fn derived_getValue2(self_: *mut Derived) -> i32;
```

## 关键点

### 内存布局

```
struct Derived {
    struct Base1 base1;  // offset 0
    struct Base2 base2;  // offset sizeof(Base1)
    int derived_value;   // offset after base2
}
```

### 方法转发

```cpp
// 调用 Base2 的方法需要正确的指针
int derived_getValue2(struct Derived* self) {
    // self 实际上被当作 Base2* 使用
    // 需要调整指针或通过 base2 成员访问
    return self->base2.value2;
}
```

### 菱形继承问题（更复杂）

如果存在菱形继承（Derived 同时继承自两个都继承自 Base 的类），
情况会更复杂，需要虚拟继承。

## 总结

1. **多继承**：每个基类作为单独的成员
2. **FFI 挑战**：多个基类指针需要正确处理偏移
3. **方法转发**：每个基类方法都需要单独的转发函数
## 运行结果

```
Base1 value: 10
Base2 value: 10
Derived value: 30
Computing: 10 + 20 + 30 = 60

Rust FFI: Multiple inheritance with hicc pattern
```
