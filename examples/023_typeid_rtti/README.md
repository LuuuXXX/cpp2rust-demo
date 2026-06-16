# 023_typeid_rtti - RTTI / typeid（hicc 直出，无 shim）

## C++ 特性

本示例展示 **RTTI（运行期类型识别）/ `typeid`** 的地道 C++ 命名空间多态体系：抽象基类
`Shape`（含纯虚 `area()`）与具体实现 `Circle`/`Rectangle`/`Triangle`。经**基类引用**对对象
施加 `typeid(...)`，可在运行期取回其**动态类型**。用 hicc 直出绑定，**无 opaque 指针、
无 `extern "C"` 桥接、无 `*_new`/`*_delete` shim**。

## C++ 代码（节选）

```cpp
namespace typeid_rtti_ns {

class Shape {
public:
    virtual ~Shape() = default;
    virtual double area() const = 0;   // 纯虚 → 多态，启用 RTTI
};

class Circle : public Shape {
public:
    explicit Circle(double r);
    double area() const override;
    double radius() const;
private:
    double radius_;
};
// Rectangle / Triangle 同理

} // namespace typeid_rtti_ns
```

配套 `standalone.sh` / `Makefile` 两种纯 C++ 构建方式。

## Rust FFI 代码（hicc 直出 + typeid 命名包装）

抽象基类 `Shape` 无公有构造、不可实例化，被工具跳过；具体类各自直接绑定（默认支架见
`lib_scaffold.rs`）。RTTI 不在直出范围内，故在手写 `lib.rs` 中以 `hicc::cpp!` 命名包装
对每个对象经**基类引用**调用 `typeid(...).name()`，再绑定为关联方法 `runtime_type_name`：

```rust
hicc::cpp! {
    #include "typeid_rtti.h"
    #include <typeinfo>
    using typeid_rtti_ns::Shape;
    using typeid_rtti_ns::Circle;
    const char* circle_runtime_type(const Circle* self) {
        const Shape& s = *self;      // 上行为基类引用
        return typeid(s).name();     // RTTI 取回动态类型名
    }
}

// import_class! 内：
#[cpp(func = "const char* circle_runtime_type(const typeid_rtti_ns::Circle*)")]
pub fn runtime_type_name(&self) -> *const i8;
```

## 关键点

| C++ 概念 | hicc 直出映射 |
|----------|--------------|
| 抽象基类 `Shape`（纯虚） | 无公有构造 → 工具跳过，不生成绑定 |
| 具体类 `Circle`/`Rectangle`/`Triangle` | 各自 `import_class!` 直接绑定 + `make_unique` 工厂 |
| `typeid(基类引用).name()` | 跳过自动绑定 → `hicc::cpp!` 命名包装 + `#[cpp(func)]` `runtime_type_name` |
| 动态类型识别 | 包装内经 `const Shape&` 取回派生类的动态类型 |
| `const char*` 返回 | Rust `*const i8`，测试侧用 `CStr` 转字符串（断言 `contains` 类名以跨平台稳定） |

## 构建方法

```bash
cd cpp && ./standalone.sh        # 纯 C++ 独立验证（或 make run）
cd rust_hicc && cargo test       # 行为级 smoke 断言
```

## 总结

1. **多态 + RTTI**：含虚函数的类启用 RTTI，`typeid` 可取动态类型。
2. **抽象基类跳过**：无公有构造的抽象基类不被直出绑定。
3. **typeid 命名包装**：以 `hicc::cpp!` 经基类引用调用 `typeid().name()`。
4. **去 shim**：无 `*_new`/`*_delete`、无 opaque 指针、无 `extern "C"` 桥接。
