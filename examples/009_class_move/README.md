# 009_class_move - 移动语义

## C++ 特性

本示例展示 C++ 移动语义，通过 FFI 实现资源所有权的转移。

## C++ 代码

### class_move.h

```cpp
// 移动语义：将 src 的资源转移给 dest
void unique_vector_move(struct UniqueVector* dest, struct UniqueVector* src);
```

### class_move.cpp

```cpp
void unique_vector_move(struct UniqueVector* dest, struct UniqueVector* src) {
    // 先清空 dest 的原有资源
    delete[] dest->data;

    // 转移所有权
    dest->data = src->data;
    dest->size = src->size;

    // src 清空
    src->data = nullptr;
    src->size = 0;
}
```

## 移动语义与 FFI

### 什么是移动语义

移动语义避免不必要的内存拷贝：
1. 将资源（如堆内存）从一个对象转移到另一个对象
2. 源对象被"掏空"，变为有效但空的状态
3. 避免深拷贝的性能开销

### C++ 右值引用

C++11 引入右值引用支持移动语义：

```cpp
class Widget {
public:
    Widget(Widget&& other) noexcept;  // 移动构造函数
};
```

### FFI 中的移动

由于 Rust 不能直接使用 C++ 的移动构造函数，通过显式函数实现：

```cpp
void move(Widget* dest, Widget* src);  // dest = std::move(src)
```

## Rust FFI 代码

```rust
#[cpp(func = "void unique_vector_move(struct UniqueVector*, struct UniqueVector*)")]
unsafe fn unique_vector_move(dest: *mut UniqueVector, src: *mut UniqueVector);
```

## 关键点

### 移动前后的状态

| 对象 | 移动前 | 移动后 |
|------|--------|--------|
| dest | 原资源 | 获得新资源 |
| src | 原资源 | 被清空（nullptr, size=0） |

### 移动后原对象仍需销毁

移动后源对象仍然存在，只是内部资源被转移：

```rust
unique_vector_delete(src);  // 仍然需要调用，但此时是空操作
```

### 与 Rust 移动语义的对比

| Rust | C++ FFI |
|------|----------|
| `let b = a;` 移动 | `move(dest, src)` |
| `a` 不再可用 | `src->data = nullptr` |
| 编译器检查 | 运行时检查 |

## 运行结果

```
UniqueVector created: empty
UniqueVector created: size=5
src_with_data size: 5
src_with_data[0]: 10
UniqueVector created: empty
dest size before move: 0
Moving UniqueVector: 5 -> 0
dest size after move: 5
dest[0]: 10
src_with_data size after move: 0
UniqueVector deleted: size=5
UniqueVector deleted: size=0

Rust FFI: Move semantics work!
```

## 总结

1. **移动避免拷贝**：资源直接转移，不复制数据
2. **源对象被清空**：移动后源对象状态变为有效但空
3. **仍需销毁**：源对象虽然空了，但仍需调用析构函数
4. **Rust 移动 vs C++ 移动**：Rust 编译器检查，C++ FFI 运行时检查
