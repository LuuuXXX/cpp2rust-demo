# 015_virtual_basic - 虚函数与覆写（hicc 直出，无 shim）

## C++ 特性

本示例展示**虚函数及派生类覆写**的地道 C++ 命名空间类：基类 `Shape` 声明虚函数
`area()`，派生类 `Circle : public Shape` 以 `override` 覆写。用 hicc 直出绑定，
**无 opaque 指针、无 `extern "C"` 桥接**。

## C++ 代码（节选）

```cpp
namespace virtual_basic_ns {

class Shape {
public:
    Shape();
    virtual ~Shape();
    virtual double area() const;   // 虚函数，默认 0
};

class Circle : public Shape {
public:
    explicit Circle(double r);
    ~Circle() override;
    double area() const override;  // 覆写：π·r²
    double radius() const;
private:
    double radius_;
};

} // namespace virtual_basic_ns
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
| 派生类覆写 | 派生类各自绑定自身 `area()`，运行期分派由 C++ 负责 |
| 构造函数 | `make_unique<Circle, double>(double&&)` 工厂 |
| 析构（虚） | hicc `Drop` 自动析构，替代 `*_delete` shim |

> 虚函数的运行期分派由 C++ 负责：`Shape` 实例 `area()` 返回 0，`Circle` 实例返回
> π·r²，体现覆写生效。

## 构建方法

```bash
cd cpp && ./standalone.sh        # 纯 C++ 独立验证（或 make run）
cd rust_hicc && cargo test       # 行为级 smoke 断言
```

## 总结

1. **虚函数**：基类声明 `virtual area()`，派生类 `override` 覆写。
2. **直出绑定**：基类/派生类各自独立 `import_class!`。
3. **覆写生效**：`Circle::area()` 返回 π·r²，区别于基类的 0。
4. **虚析构**：hicc `Drop` 自动析构，替代 `*_delete` shim。
