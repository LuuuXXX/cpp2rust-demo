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

```rust
hicc::cpp! {
    #include <stddef.h>
    #include <iostream>
    #include <map>
    #include <string>
    #include <cstring>

    #include "map_basic.h"
}

hicc::import_class! {
    #[cpp(class = "StringIntMap", destroy = "string_int_map_delete")]
    pub class StringIntMap {
        #[cpp(method = "bool insert(const char* key, int val)")]
        fn insert(&mut self, key: *const i8, val: i32) -> bool;

        #[cpp(method = "int get(const char* key) const")]
        fn get(&self, key: *const i8) -> i32;

        #[cpp(method = "void set(const char* key, int val)")]
        fn set(&mut self, key: *const i8, val: i32);

        #[cpp(method = "bool erase(const char* key)")]
        fn erase(&mut self, key: *const i8) -> bool;

        #[cpp(method = "size_t size() const")]
        fn size(&self) -> usize;

        #[cpp(method = "bool empty() const")]
        fn empty(&self) -> bool;

        #[cpp(method = "void clear()")]
        fn clear(&mut self);
    }
}

hicc::import_class! {
    #[cpp(class = "IntStringMap", destroy = "int_string_map_delete")]
    pub class IntStringMap {
        #[cpp(method = "size_t size() const")]
        fn size(&self) -> usize;
    }
}

hicc::import_lib! {
    #![link_name = "map_basic"]

    class StringIntMap;
    class IntStringMap;

    #[cpp(func = "StringIntMap* string_int_map_new()")]
    fn string_int_map_new() -> StringIntMap;

    #[cpp(func = "IntStringMap* int_string_map_new()")]
    fn int_string_map_new() -> IntStringMap;
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

## 运行结果

```
=== 035_map_basic - std::map ===

--- StringIntMap Demo ---
Empty: true
Insert 'one' = 1: true
Insert 'two' = 2: true
Insert 'three' = 3: true
Insert 'four' = 4: true
Insert 'five' = 5: true
Size: 5
Get 'one': 1
Set 'one' = 100, now: 100
Erase 'five': true
Size after erase: 4
After clear, size: 0

Rust FFI: std::map 映射
1. map 是有序关联容器（红黑树实现）
2. 插入: insert(key, value) -> bool
3. 查找: find(key) -> iterator 或 end()
4. 删除: erase(key) -> size_t
5. 字符串键需要 CString 转换
```

## 冒烟测试

本示例包含集成冒烟测试（`rust_hicc/tests/smoke.rs`），验证生成的 Rust FFI 绑定可编译、
可链接 C++ 实现，且基本行为正确。

### 测试用例

| 测试函数 | 验证内容 |
|---------|---------|
| `smoke_string_int_map_new` | `string_int_map_new()` 后 `size()` = 0 |
| `smoke_string_int_map_insert_get` | `insert("key", 42)` 后 `get("key")` = 42 |
| `smoke_string_int_map_set_overwrite` | `set("key", 99)` 后 `get("key")` = 99 |
| `smoke_string_int_map_erase` | `erase("key")` 后 `contains("key")` = false |
| `smoke_string_int_map_clear` | `clear()` 后 `size()` = 0 |

### 运行方式

```bash
cd examples/035_map_basic/rust_hicc
cargo test --test smoke
```

### 各平台支持

| 平台 | 状态 | 备注 |
|------|------|------|
| Linux (Ubuntu) | ✅ | CI `l-smoke` job 已覆盖 |
| macOS | ✅ | 支持 |
| Windows MinGW | ✅ | 支持 |

## 总结

- std::map 是有序关联容器
- FFI 边界使用函数参数传递键值
- 查找结果通过返回值和输出参数返回
- 适用于需要有序键的场景
