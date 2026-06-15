# 012_class_volatile - volatile 数据成员（hicc 直出，无 shim）

## C++ 特性

本示例展示带 **volatile 数据成员**的地道 C++ 命名空间类，用 hicc 直出方式绑定：
寄存器以 `volatile` 修饰，禁止编译器缓存/优化掉读取；访问器以普通 const/非 const
成员函数暴露，映射为 `&self` / `&mut self`，默认构造派生 `hicc::make_unique` 工厂，
**无 opaque 指针、无 `extern "C"` 桥接**。

> 说明：hicc 不支持 `volatile`-this 限定的成员函数（方法指针类型不匹配），因此地道
> 写法是保留 volatile **数据成员**、以非 volatile-this 的访问器读取，而非 `volatile`
> 限定的成员函数。

## C++ 代码（节选）

```cpp
namespace class_volatile_ns {

class HardwareDevice {
public:
    HardwareDevice();
    ~HardwareDevice();

    uint32_t read_status() const;   // 读取 volatile 寄存器 → &self
    uint32_t read_data() const;

    void init();                    // 配置 → &mut self
    void reset();

private:
    volatile uint32_t status_reg_;  // volatile 数据成员
    volatile uint32_t data_reg_;
    uint32_t config_reg_;
};

} // namespace class_volatile_ns
```

配套 `standalone.sh` / `Makefile` 两种纯 C++ 构建方式。

## Rust FFI 代码（hicc 直出）

```rust
hicc::import_class! {
    #[cpp(class = "class_volatile_ns::HardwareDevice")]
    pub class HardwareDevice {
        #[cpp(method = "uint32_t read_status() const")]
        pub fn read_status(&self) -> u32;

        #[cpp(method = "uint32_t read_data() const")]
        pub fn read_data(&self) -> u32;

        #[cpp(method = "void init()")]
        pub fn init(&mut self);

        #[cpp(method = "void reset()")]
        pub fn reset(&mut self);

        pub fn new() -> Self { hardware_device_new() }
    }
}

hicc::import_lib! {
    #![link_name = "class_volatile"]

    #[cpp(func = "std::unique_ptr<class_volatile_ns::HardwareDevice> hicc::make_unique<class_volatile_ns::HardwareDevice>()")]
    pub fn hardware_device_new() -> HardwareDevice;
}
```

## 关键点

| C++ 概念 | hicc 直出映射 |
|----------|--------------|
| volatile 数据成员 | 保留 `volatile` 修饰，访问器以普通方法读取 |
| const 访问器 | `&self`（`read_status`/`read_data`） |
| 非 const 配置 | `&mut self`（`init`/`reset`） |
| `volatile`-this 成员函数 | hicc 不支持，采用非 volatile-this 访问器替代 |

> 工具默认产物（`lib_scaffold.rs`）即包含全部访问器与构造工厂，本示例 `lib.rs`
> 与支架一致，无需手写补全。

## 构建方法

```bash
cd cpp && ./standalone.sh        # 纯 C++ 独立验证（或 make run）
cd rust_hicc && cargo test       # 行为级 smoke 断言
```

## 总结

1. **volatile 数据**：寄存器保留 `volatile` 语义，编译器不缓存读取。
2. **访问器映射**：const → `&self`，非 const → `&mut self`。
3. **Drop 析构**：hicc 自动析构，替代 `hardware_device_delete` shim。
4. **hicc 约束**：volatile-this 方法不被支持，地道写法用 volatile 数据成员 + 普通访问器。
