# 016_virtual_pure - 纯虚接口与具体实现（hicc 直出，无 shim）

## C++ 特性

本示例展示**纯虚函数（抽象基类）与多态实现**：抽象基类 `AbstractShape` 声明纯虚
函数 `area() const = 0`，不可实例化；`Circle`/`Rectangle` 各自实现接口。用 hicc
直出绑定，**无 opaque 指针、无 `extern "C"` 桥接**。

## C++ 代码（节选）

```cpp
namespace virtual_pure_ns {

class AbstractShape {                // 抽象基类（纯虚）
public:
    virtual ~AbstractShape() = default;
    virtual double area() const = 0; // 纯虚函数
};

class Circle : public AbstractShape {
public:
    explicit Circle(double r);
    double area() const override;
    double radius() const;
private:
    double radius_;
};

class Rectangle : public AbstractShape {
public:
    Rectangle(double w, double h);
    double area() const override;
private:
    double width_, height_;
};

} // namespace virtual_pure_ns
```

配套 `standalone.sh` / `Makefile` 两种纯 C++ 构建方式。

## Rust FFI 代码（hicc 直出）

具体类 `Circle`/`Rectangle` 各自独立 `import_class!` 直接绑定真实命名空间类，构造
派生 `make_unique` 工厂（详见 `rust_hicc/src/lib.rs`，与工具默认支架
`lib_scaffold.rs` 一致）。

## 关键点

| C++ 概念 | hicc 直出映射 |
|----------|--------------|
| 抽象基类（纯虚） | 无公有构造，工具跳过，不生成绑定（不可实例化） |
| 具体实现类 | 各自 `import_class!` 绑定，实现 `area()` |
| 构造函数 | `make_unique<T, double...>(double&&...)` 工厂 |
| 虚析构 | hicc `Drop` 自动析构，替代 `*_delete` shim |

> 抽象基类 `AbstractShape` 因无公有构造函数而被工具自动跳过——这正符合「抽象类不可
> 实例化」的语义；多态实现由具体类 `Circle`/`Rectangle` 的 `area()` 覆写体现。

## 构建方法

```bash
cd cpp && ./standalone.sh        # 纯 C++ 独立验证（或 make run）
cd rust_hicc && cargo test       # 行为级 smoke 断言
```

## 运行结果

```
circle.area=12.566370614359172 radius=2
rect.area=12
```

## 总结

1. **纯虚接口**：`AbstractShape::area() = 0` 定义抽象契约。
2. **跳过抽象类**：无公有构造的抽象基类不生成绑定（与语义一致）。
3. **具体实现**：`Circle`/`Rectangle` 直出绑定并实现 `area()`。
4. **虚析构**：hicc `Drop` 自动析构，替代 `*_delete` shim。
