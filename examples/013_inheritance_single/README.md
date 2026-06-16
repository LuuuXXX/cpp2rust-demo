# 013_inheritance_single - 单继承（hicc 直出，无 shim）

## C++ 特性

本示例展示**单继承**的地道 C++ 命名空间类，用 hicc 直出方式绑定：基类 `Animal`
与派生类 `Dog : public Animal` 各自以 `import_class!` 直接绑定真实命名空间类，
**无 opaque 指针、无 `extern "C"` 桥接**。派生类复用基类的 `name_` 数据成员，
并以 `override` 覆写 `speak()` 虚函数。

## C++ 代码（节选）

```cpp
namespace inheritance_single_ns {

class Animal {
public:
    explicit Animal(std::string name);
    virtual ~Animal();
    const std::string& name() const;
    virtual std::string speak() const;     // 虚函数
protected:
    std::string name_;
};

class Dog : public Animal {                // 单继承 public Animal
public:
    explicit Dog(std::string name);
    ~Dog() override;
    std::string bark() const;
    std::string speak() const override;    // 覆写
};

} // namespace inheritance_single_ns
```

配套 `standalone.sh` / `Makefile` 两种纯 C++ 构建方式。

## Rust FFI 代码（hicc 直出）

```rust
hicc::cpp! {
    #include "inheritance_single.h"
    #include <hicc/std/string.hpp>
}

hicc::import_class! {
    class string = hicc_std::string;

    #[cpp(class = "inheritance_single_ns::Animal")]
    pub class Animal {
        #[cpp(method = "const std::string& name() const")]
        pub fn name(&self) -> &string;
        #[cpp(method = "std::string speak() const")]
        pub fn speak(&self) -> string;
        pub fn new(name: string) -> Self { animal_new(name) }
    }
}

hicc::import_class! {
    #[cpp(class = "inheritance_single_ns::Dog")]
    pub class Dog {
        #[cpp(method = "std::string bark() const")]
        pub fn bark(&self) -> string;
        #[cpp(method = "std::string speak() const")]
        pub fn speak(&self) -> string;
        pub fn new(name: string) -> Self { dog_new(name) }
    }
}

hicc::import_lib! {
    #![link_name = "inheritance_single"]
    #[cpp(func = "std::unique_ptr<inheritance_single_ns::Animal> hicc::make_unique<inheritance_single_ns::Animal, std::string>(std::string&&)")]
    pub fn animal_new(name: hicc_std::string) -> Animal;
    #[cpp(func = "std::unique_ptr<inheritance_single_ns::Dog> hicc::make_unique<inheritance_single_ns::Dog, std::string>(std::string&&)")]
    pub fn dog_new(name: hicc_std::string) -> Dog;
}
```

## 关键点

| C++ 概念 | hicc 直出映射 |
|----------|--------------|
| 基类 / 派生类 | 各自独立 `import_class!` 直接绑定真实命名空间类 |
| `std::string` 成员 | `hicc_std::string`（`class string = hicc_std::string;` 别名，模块内声明一次） |
| 构造函数 | `hicc::make_unique<T, std::string>(std::string&&)` 工厂 |
| 虚函数覆写 | 各类绑定自身 `speak()`，运行期分派由 C++ 负责 |
| 析构 | hicc `Drop` 自动析构，替代 `*_delete` shim |

### hicc 约束：派生类不重复绑定继承的引用返回方法

hicc 对**派生类**绑定块中声明的「继承自基类、且返回引用」的方法（如在 `Dog`
块再声明 `const std::string& name() const`）会产生错误的 `this` 偏移，运行期触发
SIGSEGV。因此继承而来的访问器仅在基类 `Animal` 块声明；派生类 `Dog` 只绑定自身
方法。派生类对基类数据的复用通过其自身方法（`bark()`/`speak()` 的输出含构造名）
体现。

## 构建方法

```bash
cd cpp && ./standalone.sh        # 纯 C++ 独立验证（或 make run）
cd rust_hicc && cargo test       # 行为级 smoke 断言
```

## 运行结果

```
Generic: Generic makes a sound
dog speak: Buddy barks: Woof! Woof!
dog bark : Buddy barks: Woof! Woof!
```

## 总结

1. **单继承**：基类/派生类各自直出绑定真实命名空间类。
2. **数据复用**：派生类复用基类 `name_`，经自身方法体现。
3. **虚函数**：`speak()` 覆写，C++ 负责运行期分派。
4. **hicc 约束**：派生类不重复绑定继承的引用返回方法（this 偏移问题）。
