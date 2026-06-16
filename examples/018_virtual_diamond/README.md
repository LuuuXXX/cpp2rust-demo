# 018_virtual_diamond - 菱形虚继承（hicc 直出，无 shim）

## C++ 特性

本示例展示**菱形虚继承**：`A` 为顶点基类，`B`/`C` 各以 `virtual public A` 虚继承，
`D : public B, public C` 汇聚。虚继承保证 `D` 中 `A` 子对象**唯一**（避免重复）。
用 hicc 直出绑定，**无 opaque 指针、无 `extern "C"` 桥接**。

## C++ 代码（节选）

```cpp
namespace virtual_diamond_ns {

class A { /* a_value_ */ public: explicit A(int v); int a_value() const; };
class B : virtual public A { /* b_value_ */ public: B(int a, int b); int b_value() const; };
class C : virtual public A { /* c_value_ */ public: C(int a, int c); int c_value() const; };

class D : public B, public C {     // 菱形汇聚
public:
    D(int a, int b, int c, int d); // 虚继承：D 直接初始化唯一的 A 子对象
    int d_value() const;
    int compute() const;           // a_value_ + b_value_ + c_value_ + d_value_
private:
    int d_value_;
};

} // namespace virtual_diamond_ns
```

配套 `standalone.sh` / `Makefile` 两种纯 C++ 构建方式。

## Rust FFI 代码（hicc 直出）

四个类各自独立 `import_class!` 直接绑定真实命名空间类，构造派生 `make_unique`
工厂（详见 `rust_hicc/src/lib.rs`，与工具默认支架 `lib_scaffold.rs` 一致）。

## 关键点

| C++ 概念 | hicc 直出映射 |
|----------|--------------|
| 虚继承 `virtual public A` | 顶点基类 `A` 子对象唯一；各类独立绑定 |
| 菱形派生 `D : public B, C` | 独立 `import_class!`，`make_unique<D, int, int, int, int>` |
| 数据汇聚 | `D::compute()` 在 C++ 内部读取唯一 `A` 及 `B`/`C` 数据 |
| 虚析构 | hicc `Drop` 自动析构，替代 `*_delete` shim |

### hicc 约束：派生类不重复绑定跨虚基类的继承方法

跨虚基类调用继承而来的方法（例如在 `D` 绑定块再声明 `int a_value() const`）会因 hicc
成员函数指针**丢失/截断虚继承的 `this` 偏移量**而出错。因此各派生类只绑定自身方法；
对唯一 `A` 子对象与 `B`/`C` 数据的汇聚访问，统一经派生类自身的 `compute()`（返回四者
之和）体现。这与本仓库早期版本用 `const D*` 包装函数规避偏移截断的处置一致，但直出
写法通过「只绑自身方法」更彻底地回避了该问题。

## 构建方法

```bash
cd cpp && ./standalone.sh        # 纯 C++ 独立验证（或 make run）
cd rust_hicc && cargo test       # 行为级 smoke 断言
```

## 运行结果

```
a=1 b=2 c=3
d_value=4 compute=10
```

## 总结

1. **菱形虚继承**：`A` 子对象唯一，避免重复继承的二义性。
2. **直出绑定**：A/B/C/D 各自独立 `import_class!`。
3. **数据汇聚**：`compute()` 返回唯一 A 与 B/C/D 数据之和（10）。
4. **hicc 约束**：派生类不重复绑定跨虚基类的继承方法（this 偏移截断）。
