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

## 运行结果

```
=== 023_typeid_rtti - typeid 与 RTTI ===


Using typeid to determine runtime type:
Circle: type=-1, name=Circle, area=0.00
Rectangle: type=-1, area=0.00
Triangle: type=-1, area=0.00
Deleting Shape
Deleting Shape
Deleting Shape

Rust FFI: typeid 变成类型枚举或字符串比较
RTTI 信息在 FFI 边界丢失，需在 C++ 侧导出类型信息
```

## 总结

- C++ RTTI 信息无法直接传递到 Rust
- 解决方案：在 C++ 侧导出类型枚举
- 使用工厂函数创建对象，返回时携带类型信息