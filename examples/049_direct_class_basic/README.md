# 049_direct_class_basic

Direct 绑定模式示例：纯 C++ 类，**无** `extern "C"` shim 访问器。

## 与 006_class_basic 的区别

| 项 | 006_class_basic（Shim） | 049_direct_class_basic（Direct） |
|---|---|---|
| C++ shim 函数 | `counter_new` / `counter_get` / `counter_delete` 等 | 无 |
| Rust 工厂 | `Counter* counter_new()` → `*mut` 持有 | `std::unique_ptr<Counter> hicc::make_unique<Counter>()` → owned T |
| `destroy` 属性 | `#[cpp(class = "Counter", destroy = "counter_delete")]` | `#[cpp(class = "Counter")]`（hicc make_unique 默认 `delete`） |
| 方法绑定 | 通过 shim 自由函数间接调用 | 直接 `#[cpp(method = "...")]` 指向成员函数指针 |

## 判定逻辑

工具自动通过 [`extractor::direct_binding::classify`] 判定：
- C++ 头/源没有任何 `counter_*` 形式且首参为 `Counter*` 的自由函数
- → `BindingMode::Direct`
- → 生成 `make_unique` 工厂、跳过 `destroy` 属性、方法直绑

## 运行

```bash
cd rust_hicc && cargo run
```

预期输出：

```
Initial value: 0
After 3 increments: 3
After 1 decrement: 2
```
