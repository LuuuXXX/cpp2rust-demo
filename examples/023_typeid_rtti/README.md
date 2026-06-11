# 023_typeid_rtti - typeid 与 RTTI

## C++ 特性

本示例展示 C++ RTTI（运行时类型信息）的使用，通过 `typeid` 在运行时确定对象实际类型。

## C++ 代码

### typeid_rtti.cpp

```cpp
// 使用 typeid 确定实际类型
const std::type_info& ti = typeid(*self);

if (ti == typeid(Circle)) {
    return SHAPE_TYPE_CIRCLE;
} else if (ti == typeid(Rectangle)) {
    return SHAPE_TYPE_RECTANGLE;
}
```

## Rust FFI 方案

### main.rs

```rust
// FFI 策略：在 C++ 侧导出类型枚举
#[cpp(func = "int shape_getType(struct Shape*)")]
unsafe fn shape_getType(self_: *mut Shape) -> i32;

// 类型检查变成枚举值比较
match shape_getType(shape) {
    SHAPE_TYPE_CIRCLE => { /* Circle 逻辑 */ }
    SHAPE_TYPE_RECTANGLE => { /* Rectangle 逻辑 */ }
    _ => { /* Unknown */ }
}
```

## FFI 对比分析

| 方面 | C++ | Rust FFI |
|------|-----|----------|
| 运行时类型检测 | `typeid(*obj)` | 导出类型枚举函数 |
| 类型名称 | `ti.name()` | 编译器特定，无法可靠传递 |
| 类型比较 | `ti == typeid(Circle)` | 枚举值比较 |


## Rust FFI 代码

```rust
hicc::cpp! {
    #include <iostream>
    #include <typeinfo>
    #include <cmath>

    #include "typeid_rtti.h"
}

hicc::import_class! {
    #[cpp(class = "Shape", destroy = "shape_delete")]
    pub class Shape {
        #[cpp(method = "int getType() const")]
        fn get_type(&self) -> i32;

        #[cpp(method = "const char* getTypeName() const")]
        fn get_type_name(&self) -> *const i8;

        #[cpp(method = "double area() const")]
        fn area(&self) -> f64;
    }
}

hicc::import_lib! {
    #![link_name = "typeid_rtti"]

    class Shape;

    #[cpp(func = "Shape* shape_new_circle(double)")]
    fn shape_new_circle(radius: f64) -> Shape;

    #[cpp(func = "Shape* shape_new_rectangle(double, double)")]
    fn shape_new_rectangle(width: f64, height: f64) -> Shape;

    #[cpp(func = "Shape* shape_new_triangle(double, double)")]
    fn shape_new_triangle(base: f64, height: f64) -> Shape;
}
```

## 运行结果

```
=== 023_typeid_rtti - typeid 与 RTTI ===


Using typeid to determine runtime type:
Circle: type=0, name=Circle, area=78.54
Rectangle: type=1, area=24.00
Triangle: type=2, area=6.00
Deleting Circle
Deleting Rectangle
Deleting Triangle

Rust FFI: typeid 变成类型枚举或字符串比较
RTTI 信息在 FFI 边界丢失，需在 C++ 侧导出类型信息
```

## 冒烟测试

本示例包含集成冒烟测试（`rust_hicc/tests/smoke.rs`），验证生成的 Rust FFI 绑定可编译、
可链接 C++ 实现，且基本行为正确。

### 测试用例

| 测试函数 | 验证内容 |
|---------|---------|
| `smoke_circle_type_and_area` | `shape_new_circle(5.0)` 后 `get_type()` = 0（`SHAPE_TYPE_CIRCLE`），面积约 78.5 |
| `smoke_rectangle_type_and_area` | `shape_new_rectangle(4.0, 6.0)` 后 `get_type()` = 1，面积 = 24.0 |
| `smoke_triangle_type_and_area` | `shape_new_triangle(3.0, 4.0)` 后 `get_type()` = 2，面积 = 6.0 |
| `smoke_get_type_name` | `get_type_name()` 返回 `"Circle"` 字符串 |

### 运行方式

```bash
cd examples/023_typeid_rtti/rust_hicc
cargo test --test smoke
```

### 各平台支持

| 平台 | 状态 | 备注 |
|------|------|------|
| Linux (Ubuntu) | ✅ | CI `l-smoke` job 已覆盖 |
| macOS | ✅ | 支持 |
| Windows MinGW | ✅ | 支持 |

## 总结

- C++ RTTI 信息无法直接传递到 Rust
- 解决方案：在 C++ 侧导出类型枚举
- 使用工厂函数创建对象，返回时携带类型信息