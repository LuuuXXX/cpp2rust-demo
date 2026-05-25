# 035_map_basic - std::map

## C++ 特性

本示例展示 C++ `std::map`（有序关联容器）的基本操作，以及如何通过 FFI 导出给 Rust 使用。

## C++ 代码

### map_basic.h

```cpp
#pragma once
#ifdef __cplusplus
extern "C" {
#endif

struct StringIntMap;

struct StringIntMap* string_int_map_new(void);
void string_int_map_delete(struct StringIntMap* self);

size_t string_int_map_size(struct StringIntMap* self);
int string_int_map_empty(struct StringIntMap* self);

// 插入（返回 1 表示成功，0 表示键已存在）
int string_int_map_insert(struct StringIntMap* self, const char* key, int value);

// 查找（返回 1 表示找到，0 表示未找到）
int string_int_map_find(struct StringIntMap* self, const char* key, int* out_value);

// 删除（返回 1 表示删除成功，0 表示键不存在）
int string_int_map_erase(struct StringIntMap* self, const char* key);

#ifdef __cplusplus
}
#endif
```

### map_basic.cpp

```cpp
#include "map_basic.h"
#include <map>
#include <string>

struct StringIntMap {
    std::map<std::string, int> data;
};

struct StringIntMap* string_int_map_new() {
    return new StringIntMap();
}

int string_int_map_insert(struct StringIntMap* self, const char* key, int value) {
    return self->data.insert({std::string(key), value}).second;
}

int string_int_map_find(struct StringIntMap* self, const char* key, int* out_value) {
    auto it = self->data.find(std::string(key));
    if (it != self->data.end()) {
        *out_value = it->second;
        return 1;
    }
    return 0;
}
```

## std::map 特点

| 操作 | C++ | Rust 等效 |
|------|-----|-----------|
| 创建 | `map<K,V> m` | `HashMap::new()` |
| 插入 | `m.insert(k,v)` | `m.insert(k,v)` |
| 查找 | `m.find(k)` | `m.get(k)` |
| 删除 | `m.erase(k)` | `m.remove(k)` |
| 大小 | `m.size()` | `m.len()` |
| 空检查 | `m.empty()` | `m.is_empty()` |

### std::map vs std::unordered_map

| 特性 | std::map | std::unordered_map |
|------|----------|-------------------|
| 底层结构 | 红黑树 | 哈希表 |
| 元素顺序 | 有序 | 无序 |
| 查找复杂度 | O(log n) | O(1) 平均 |
| 插入复杂度 | O(log n) | O(1) 平均 |

## Rust FFI 代码

### main.rs

```rust
hicc::import_lib! {
    #![link_name = "map_basic"]

    struct StringIntMap;

    #[cpp(func = "struct StringIntMap* string_int_map_new(void)")]
    fn string_int_map_new() -> *mut StringIntMap;

    #[cpp(func = "int string_int_map_insert(struct StringIntMap*, const char*, int)")]
    unsafe fn string_int_map_insert(map: *mut StringIntMap, key: *const i8, value: i32) -> i32;

    #[cpp(func = "int string_int_map_find(struct StringIntMap*, const char*, int*)")]
    unsafe fn string_int_map_find(map: *mut StringIntMap, key: *const i8, out_value: *mut i32) -> i32;
}
```

## FFI 对比分析

| 方面 | C++ std::map | Rust FFI |
|------|--------------|----------|
| 键值对 | `std::pair<const K, V>` | 分离的参数 |
| 查找结果 | 迭代器 | 返回值 + 输出参数 |
| 字符串键 | `std::string` | `const char*` |
| 模板参数 | 编译时确定 | FFI 函数重载 |

## 关键点

1. **有序性**：std::map 保持键的顺序
2. **唯一键**：每个键最多一个值
3. **查找语义**：返回 1/0 表示成功/失败
4. **输出参数**：使用指针返回查找结果

## 总结

- std::map 是有序关联容器
- FFI 边界使用函数参数传递键值
- 查找结果通过返回值和输出参数返回
- 适用于需要有序键的场景
