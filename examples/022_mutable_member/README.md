# 022_mutable_member - mutable 成员

## C++ 特性

本示例展示 C++ 中 `mutable` 关键字的作用：允许在 const 成员函数中修改特定成员。

## C++ 代码

### mutable_member.h

```cpp
struct DataFetcher {
    mutable int cache_count;  // mutable 成员
    const char* getName() const;  // const 方法可以修改 cache_count
};
```

## FFI 对比分析

| 方面 | C++ | Rust FFI |
|------|-----|----------|
| mutable 作用 | 允许 const 方法修改该成员 | 无影响 - FFI 调用只是 C 函数 |
| 函数签名 | `const char* getName() const` | `const char* datafetcher_getName(struct DataFetcher*)` |
| 实现差异 | 内部可修改 mutable 成员 | 相同实现，但 Rust 不区分 |

## 运行结果

```
=== 022_mutable_member - mutable 成员 ===

Calling getName() 3 times (const method with mutable cache):
  Call 1: name = 0, cache_count = 0
  Call 2: name = 1, cache_count = 0
  Call 3: name = 2, cache_count = 0

Refreshing...
Cache count after refresh: 1

Rust FFI: mutable 关键字在 FFI 中无影响
mutable 只影响 C++ 编译器允许在 const 方法中修改该成员
```

## 总结

- `mutable` 是 C++ 编译器内部优化机制
- 在 FFI 中无影响 - 传递的是指针，函数实现相同
- Rust FFI 调用时无需特别处理