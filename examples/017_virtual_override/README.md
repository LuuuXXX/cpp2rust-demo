# 017_virtual_override - 显式 override 覆写虚函数（hicc 直出，无 shim）

## C++ 特性

本示例展示用 **`override` 关键字显式覆写虚函数**的地道 C++ 命名空间类：基类 `Base`
声明虚函数 `area()`，派生类 `Derived : public Base` 以 `override` 覆写。用 hicc 直出
绑定，**无 opaque 指针、无 `extern "C"` 桥接**。

## C++ 代码（节选）

```cpp
namespace virtual_override_ns {

class Base {
public:
    Base();
    virtual ~Base();
    virtual double area() const;       // 虚函数，默认 0
};

class Derived : public Base {
public:
    explicit Derived(double v);
    ~Derived() override;
    double area() const override;      // 显式 override 覆写：value_ * value_
    double value() const;
private:
    double value_;
};

} // namespace virtual_override_ns
```

配套 `standalone.sh` / `Makefile` 两种纯 C++ 构建方式。

## Rust FFI 代码（hicc 直出）

基类与派生类各自独立 `import_class!` 直接绑定真实命名空间类，构造派生
`make_unique` 工厂（详见 `rust_hicc/src/lib.rs`，与工具默认支架 `lib_scaffold.rs`
一致）。

## 关键点

| C++ 概念 | hicc 直出映射 |
|----------|--------------|
| 虚函数 `area()` | 基类绑定 `area() -> f64` |
| `override` 覆写 | 派生类各自绑定自身 `area()`，运行期分派由 C++ 负责 |
| 构造函数 | `make_unique<Derived, double>(double&&)` 工厂 |
| 虚析构 | hicc `Drop` 自动析构，替代 `*_delete` shim |

> `override` 是编译期检查关键字（确保确实覆写了基类虚函数），运行期行为与普通虚函数
> 覆写一致：`Base::area()` 返回 0，`Derived::area()` 返回 `value_²`。

## 构建方法

```bash
cd cpp && ./standalone.sh        # 纯 C++ 独立验证（或 make run）
cd rust_hicc && cargo test       # 行为级 smoke 断言
```

## 运行结果

```
base.area=0
derived.area=36 value=6
```

## 总结

1. **显式覆写**：`override` 关键字在编译期校验覆写正确性。
2. **直出绑定**：基类/派生类各自独立 `import_class!`。
3. **覆写生效**：`Derived::area()` 返回 `value_²`，区别于基类的 0。
4. **虚析构**：hicc `Drop` 自动析构，替代 `*_delete` shim。
