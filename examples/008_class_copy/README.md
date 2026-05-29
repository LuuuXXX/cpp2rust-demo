# 008_class_copy - 拷贝构造函数

## C++ 特性

本示例展示 C++ 拷贝构造函数的 FFI 模式，通过工厂函数实现深拷贝。

## C++ 代码

### class_copy.h

```cpp
struct Buffer* buffer_newCopy(const struct Buffer* other);
```

### class_copy.cpp

```cpp
struct Buffer* buffer_newCopy(const struct Buffer* other) {
    int* new_data = new int[other->size];
    if (other->data) {
        memcpy(new_data, other->data, other->size * sizeof(int));
    }
    return new Buffer{new_data, other->size};
}
```

## 拷贝构造函数与 FFI

### 深拷贝 vs 浅拷贝

| 类型 | 行为 |
|------|------|
| 浅拷贝 | 复制指针，不复制数据 |
| 深拷贝 | 复制指针，分配新内存，复制数据 |

本例实现深拷贝：修改原对象不影响拷贝对象。

### FFI 映射

| C++ 概念 | FFI 函数 |
|----------|----------|
| 拷贝构造函数 | `Class* class_copy(const Class*)` |
| const 引用参数 | `const Class*` |

## Rust FFI 代码

```rust
#[cpp(func = "struct Buffer* buffer_newCopy(const struct Buffer*)")]
unsafe fn buffer_newCopy(other: *const Buffer) -> *mut Buffer;
```

## 关键点

### const 在 FFI 的意义

1. **承诺不修改**：调用者保证不修改数据
2. **编译器检查**：编译器帮助检测意外修改
3. **Rust 映射**：`const` 映射为 `*const T`

### 内存管理

拷贝构造后，两个 Buffer 独立存在：
```rust
let buf2 = buffer_newCopy(buf1);  // 新的独立内存
buffer_delete(buf1);               // buf1 释放，不影响 buf2
buffer_delete(buf2);              // buf2 独立释放
```

## 运行结果

```
buf1 size: 5
buf1 values: 10 20 30 40 50
buf2 created by copy
buf2 size: 5
buf2 values: 10 20 30 40 50
After modifying buf1[0] = 999:
buf1[0] = 999
buf2[0] = 10 (unchanged)

Rust FFI: Copy constructor pattern works!
```

## 总结

1. **拷贝构造需要深拷贝**：分配新内存，复制数据
2. **const 参数**：表示只读，不修改原对象
3. **独立生命周期**：拷贝对象与原对象独立
