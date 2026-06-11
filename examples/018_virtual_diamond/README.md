# 018_virtual_diamond - 菱形继承

## C++ 特性

本示例展示 C++ 菱形继承的 FFI 映射。

## C++ 代码

### virtual_diamond.h

```cpp
//     A
//    / \
//   B   C
//    \ /
//     D

class A { int a_value; };

class B : virtual public A { int b_value; };
class C : virtual public A { int c_value; };

class D : public B, public C { int d_value; };
```

## 菱形继承问题

### 非虚拟继承的问题

```cpp
class D : public B, public C { };
// D 有两个 A 子对象！
// 通过 B 访问的 A 和通过 C 访问的 A 是不同的
```

### 虚拟继承的解决

```cpp
class B : virtual public A { };  // virtual 关键字
class C : virtual public A { };

class D : public B, public C { };
// D 只有一个 A 子对象
// B 和 C 共享同一个 A
```

### 内存布局

**虚拟继承的内存布局（编译器相关）：**

```
非虚拟继承：
D:
  B:
    A: a_value
    b_value
  C:
    A: a_value  <-- 另一个 A 子对象
    c_value
  d_value

虚拟继承：
D:
  B:
    A (共享): a_value  <-- 同一个 A
    b_value
  C:
    c_value
  d_value
  [vptr 或偏移信息]
```

## FFI 挑战

### 指针调整

菱形继承中，指针转换需要调整：

```cpp
D* d = new D;
B* b = d;      // B 在 offset 0
C* c = d;      // C 可能在不同 offset
A* a_via_b = d; // 需要调整
A* a_via_c = d; // 需要调整
```

### FFI 中的虚拟继承

```c
// 在 C FFI 中，虚拟继承需要显式建模
struct D {
    struct B base_b;  // B 包含 A
    struct C base_c;  // C 也包含 A，但它们是同一个 A
    int d_value;
};
```

注意：在纯 C FFI 中无法自动保证 A 的唯一性，需要手动处理。

## Rust FFI 代码

```rust
hicc::cpp! {
    #include <iostream>

    #include "virtual_diamond.h"
    int d_get_a_value(D* self) {
        return self->getAValue();
    }

}

hicc::import_class! {
    #[cpp(class = "D", destroy = "d_delete")]
    pub class D {
        #[cpp(method = "int getBValue() const")]
        fn get_b_value(&self) -> i32;

        #[cpp(method = "int getCValue() const")]
        fn get_c_value(&self) -> i32;

        #[cpp(method = "int getDValue() const")]
        fn get_d_value(&self) -> i32;

        #[cpp(method = "void compute() const")]
        fn compute(&self);
    }
}

hicc::import_lib! {
    #![link_name = "virtual_diamond"]

    class D;

    #[cpp(func = "D* d_new(int, int, int, int)")]
    fn d_new(a: i32, b: i32, c: i32, d: i32) -> D;

    #[cpp(func = "int d_get_a_value(D*)")]
    fn d_get_a_value(self_: *mut D) -> i32;
}
```
## 关键点

### 虚拟继承的复杂性

| 问题 | 影响 |
|------|------|
| 多个中间类 | B 和 C 都需要知道 A 的偏移 |
| 运行时调度 | 可能需要 vptr 或其他机制 |
| 指针转换 | D* 到 A* 需要正确调整 |

### 实际建议

1. **避免菱形继承**：在 FFI 边界优先使用组合
2. **显式建模**：在 C 结构中明确表示继承关系
3. **工厂函数**：隐藏复杂的构造逻辑

### 虚拟继承的实现

编译器通常使用以下技术之一：
1. **vptr 指向虚基类表**
2. **偏移量直接编码在对象中**
3. **部分构造/析构**

## 运行结果

```
=== Diamond Inheritance FFI with hicc ===

Diamond inheritance structure:
       A
      / \
     B   C
      \ /
       D

Virtual inheritance ensures only ONE A subobject in D

Values:
  A value (via B): 1
  B value: 2
  C value: 3
  D value: 4

D::compute: a=1 b=2 c=3 d=4
Sum: 10

Rust FFI: Diamond inheritance works correctly with hicc!
```

## 总结

1. **菱形继承**：D 继承 B 和 C，B 和 C 都继承 A
2. **虚拟继承**：确保只有一个 A 子对象
3. **FFI 挑战**：指针转换需要正确调整
4. **最佳实践**：在 FFI 边界避免复杂的继承层次
