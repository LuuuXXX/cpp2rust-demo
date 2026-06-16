# 014_inheritance_multiple - 多继承（hicc 直出，无 shim）

## C++ 特性

本示例展示**多继承**的地道 C++ 命名空间类：派生类
`Derived : public Base1, public Base2` 同时继承两个基类，组合二者的数据成员。
用 hicc 直出绑定，**无 opaque 指针、无 `extern "C"` 桥接**。

## C++ 代码（节选）

```cpp
namespace inheritance_multiple_ns {

class Base1 {
public:
    explicit Base1(int v);
    virtual ~Base1();
    int value1() const;
protected:
    int value1_;
};

class Base2 {
public:
    explicit Base2(int v);
    virtual ~Base2();
    int value2() const;
protected:
    int value2_;
};

class Derived : public Base1, public Base2 {   // 多继承
public:
    Derived(int v1, int v2, int dv);
    ~Derived() override;
    int derived_value() const;
    int compute() const;   // value1_ + value2_ + derived_value_
private:
    int derived_value_;
};

} // namespace inheritance_multiple_ns
```

配套 `standalone.sh` / `Makefile` 两种纯 C++ 构建方式。

## Rust FFI 代码（hicc 直出）

两个基类与派生类各自独立 `import_class!` 直接绑定真实命名空间类，构造派生
`make_unique` 工厂（详见 `rust_hicc/src/lib.rs`，与工具默认支架
`lib_scaffold.rs` 一致）。

## 关键点

| C++ 概念 | hicc 直出映射 |
|----------|--------------|
| 多个基类 | 每个基类独立 `import_class!` 绑定 |
| 多继承派生类 | 独立 `import_class!`，构造 `make_unique<Derived, int, int, int>` |
| 数据组合 | 派生类 `compute()` 复用两基类数据成员 |
| 析构 | hicc `Drop` 自动析构，替代 `*_delete` shim |

> 按 hicc 约束，派生类绑定块只声明自身方法，不重复绑定继承而来的
> `value1()`/`value2()`（多继承下两基类 `this` 偏移不同）。基类数据的复用通过派生
> 类自身的 `compute()`（返回三者之和）体现。

## 构建方法

```bash
cd cpp && ./standalone.sh        # 纯 C++ 独立验证（或 make run）
cd rust_hicc && cargo test       # 行为级 smoke 断言
```

## 运行结果

```
b1.value1=10
b2.value2=20
derived=12 compute=42
```

## 总结

1. **多继承**：派生类同时继承两个基类并组合其数据。
2. **直出绑定**：基类/派生类各自独立 `import_class!`。
3. **数据复用**：`compute()` 读取两基类成员，返回三者之和。
4. **hicc 约束**：派生类不重复绑定继承的基类方法（this 偏移差异）。
