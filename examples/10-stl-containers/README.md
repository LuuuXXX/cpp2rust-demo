# 示例 10：STL 容器

## 特性概述

本示例展示通过 C++ 接口使用 **STL 容器**，包括 `std::vector`、`std::list`、`std::map`、`std::set`、`std::unordered_map`、`std::deque` 等标准库容器。hicc 通过 `hicc-std` 子库提供对 STL 容器的完整安全封装。

## C++ 特性说明

| STL 容器 | 说明 |
|----------|------|
| `std::vector<T>` | 动态数组，支持随机访问 |
| `std::list<T>` | 双向链表 |
| `std::map<K, V>` | 有序键值对（红黑树） |
| `std::set<T>` | 有序集合（红黑树） |
| `std::unordered_map<K, V>` | 哈希表键值对 |
| `std::unordered_set<T>` | 哈希集合 |
| `std::deque<T>` | 双端队列 |

### 代码结构

```cpp
// 通过引用传入 STL 容器的函数
int vector_sum(const std::vector<int>& v);
int vector_size(const std::vector<int>& v);
int list_sum(const std::list<int>& l);
int map_get(const std::map<int, int>& m, int key);
bool set_contains(const std::set<int>& s, int key);
int deque_get(const std::deque<int>& d, int idx);
```

## AST JSON 结构要点

运行以下命令生成 `ast.json`：

```bash
clang++ -Xclang -ast-dump=json -fsyntax-only main.cpp > ast.json
```

关键 AST 信息：

| AST 表现 | 含义 |
|----------|------|
| `qualType: "const std::vector<int> &"` | `const vector<int>&` 参数 |
| `qualType: "const std::map<int, int> &"` | `const map<int,int>&` 参数 |
| 函数参数中的 STL 类型 | 触发 `hicc_std` 提示注释 |

## hicc 处理方式

### `hicc-std` 库

`hicc-std` 是 hicc 生态中专门处理 C++ STL 容器的库，提供：
- 与 C++ STL 内存布局兼容的 Rust 包装类型
- 安全的 API（消除迭代器失效等内存安全问题）
- 完整的 STL 容器接口覆盖

| C++ STL 类型 | `hicc-std` Rust 类型 | 头文件 |
|-------------|----------------------|--------|
| `std::string` | `hicc_std::string` | `hicc/std/string.hpp` |
| `std::vector<T>` | `hicc_std::vector<T>` | `hicc/std/vector.hpp` |
| `std::map<K, V>` | `hicc_std::map<K, V>` | `hicc/std/map.hpp` |
| `std::set<T>` | `hicc_std::set<T>` | `hicc/std/set.hpp` |
| `std::unordered_map<K, V>` | `hicc_std::unordered_map<K, V>` | `hicc/std/unordered_map.hpp` |
| `std::deque<T>` | `hicc_std::deque<T>` | `hicc/std/deque.hpp` |
| `std::list<T>` | `hicc_std::list<T>` | `hicc/std/list.hpp` |

### 容器创建与使用

```rust
use hicc::AbiClass;

hicc::cpp! {
    #include <hicc/std/vector.hpp>
    #include <hicc/std/map.hpp>
    typedef std::vector<int> CppIntVec;
    typedef std::map<int, int> CppIntMap;
}

hicc::import_lib! {
    #![link_name = "example"]

    class IntVec = hicc_std::vector<hicc::Pod<i32>>;
    class IntMap = hicc_std::map<hicc::Pod<i32>, hicc::Pod<i32>>;

    #[cpp(func = "std::unique_ptr<CppIntVec> hicc::make_unique<CppIntVec>()")]
    fn int_vec_new() -> IntVec;

    #[cpp(func = "std::unique_ptr<CppIntMap> hicc::make_unique<CppIntMap>()")]
    fn int_map_new() -> IntMap;

    // 现有接口绑定
    #[cpp(func = "int vector_sum(const std::vector<int>&)")]
    fn vector_sum(v: &IntVec) -> i32;

    #[cpp(func = "int map_get(const std::map<int, int>&, int)")]
    fn map_get(m: &IntMap, key: i32) -> i32;
}

fn main() {
    let mut vec = int_vec_new();
    vec.push_back(&1);
    vec.push_back(&2);
    vec.push_back(&3);
    println!("sum = {}", vector_sum(&vec));

    let mut map = int_map_new();
    // map.insert(...)
}
```

### 迭代器安全封装

`hicc-std` 对迭代器进行了二次封装，消除了迭代器失效的内存安全风险：

```rust
// C++ vector::begin() 在 vector 被修改后可能失效
// hicc-std 将迭代器生命周期与容器绑定：
struct vector<T> {
    fn iter(&self) -> ContainerIter<'_, T> { ... }
}
// 生命周期 '_ 确保迭代器不会比容器存活更长
```

### STL 容器存储 Rust 数据（`RustAny`）

```rust
hicc::cpp! {
    #include <hicc/std/vector.hpp>
    #include <hicc/rust_any.hpp>
    typedef std::vector<RustAny> CppAnyVec;
}

hicc::import_lib! {
    #![link_name = "example"]

    class AnyVec = hicc_std::vector<hicc::RustAny<MyData>>;

    #[cpp(func = "std::unique_ptr<CppAnyVec> hicc::make_unique<CppAnyVec>()")]
    fn any_vec_new() -> AnyVec;
}

#[derive(Clone)]
struct MyData { val: i32 }

fn main() {
    let mut vec = any_vec_new();
    let item = hicc::RustAny::new_clone(MyData { val: 42 });
    vec.push_back(&item);
    println!("val = {}", vec.back().unwrap().val);
}
```

## 注意事项

1. **`POD` vs 类类型**：STL 容器的元素类型若为基础类型（`int`、`double`），需用 `hicc::Pod<T>` 包装；若为 C++ 类，直接用对应的 `hicc_std` 类型
2. **`build.rs` 配置**：使用 `hicc-std` 时需要在 `hicc_build::Build::new()` 后，头文件会通过 `DEP_HICC_STD_INCLUDE` 环境变量自动注入
3. **非缺省分配器**：`std::vector<T, CustomAlloc>` 等含自定义分配器的类型需要额外配置
4. **容器赋值**：容器赋值通过 `AbiClass::write()` 而非直接 `=` 赋值，以正确调用 C++ 赋值运算符
