# hicc-std

基于`hicc`为`c++`标准库中的容器类提供安全的`rust api`. 

注：`0.2.0`版本为原可能抛出各种异常的调用接口增加入参检查消除异常, 部分这类接口的函数类型发生了变化.

## 平台支持说明

| 平台 | 编译器 | 支持状态 | 说明 |
|------|--------|----------|------|
| Linux | GCC / Clang | ✅ 支持 | 链接 `libstdc++` |
| Windows (MinGW) | GCC | ✅ 支持 | 链接 `libstdc++` |
| Windows (MSVC) | MSVC | ✅ 支持 | 自动链接 C++ 标准库 |
| macOS | Apple Clang | ⚠️ 已知编译问题 | 见下方说明 |

### macOS Apple Clang 编译问题

`hicc-std 0.2` 在 macOS Apple Clang 下存在编译问题，**根本原因**：

`build.rs` 在非 MSVC 平台统一链接 `stdc++`（GNU libstdc++）：

```rust
#[cfg(not(all(target_os = "windows", target_env = "msvc")))]
println!("cargo::rustc-link-lib=stdc++");
```

而 macOS 使用 Apple Clang，默认 C++ 标准库为 `libc++`，应链接 `-lc++` 而非 `-lstdc++`。
在 macOS 上执行 `cargo build` 时会因找不到 `stdc++` 而失败。

**推荐做法**：
- 在 macOS 上，若需使用 STL 容器类型，建议通过 C++ 侧自定义包装类将其暴露为
  `extern "C"` 接口，Rust 侧直接绑定包装类，无需依赖 `hicc-std`。
- 如确需在 macOS 上使用 `hicc-std`，可在 `build.rs` 中根据平台选择正确的 C++ 标准库：

```rust
// macOS 使用 libc++，其他 Unix/Linux 使用 libstdc++
#[cfg(target_os = "macos")]
println!("cargo::rustc-link-lib=c++");
#[cfg(not(any(target_os = "macos", all(target_os = "windows", target_env = "msvc"))))]
println!("cargo::rustc-link-lib=stdc++");
```

支持`c++`标准库的全部容器类型:

1. `std::basic_string<charT>` (`hicc/std/string.hpp`)
1. `std::vector<T>`(`hicc/std/vector.hpp`)
1. `std::vector<bool>`(`hicc/std/vector.hpp`)
1. `std::deque<T>`(`hicc/std/deque.hpp`)
1. `std::set<T>`(`hicc/std/set.hpp`)
1. `std::multiset<T>`(`hicc/std/set.hpp`)
1. `std::unordered_set<T>`(`hicc/std/unordered_set.hpp`)
1. `std::unordered_multiset<T>`(`hicc/std/unordered_set.hpp`)
1. `std::map<T>`(`hicc/std/map.hpp`)
1. `std::multimap<T>`(`hicc/std/map.hpp`)
1. `std::unordered_map<T>`(`hicc/std/unordered_map.hpp`)
1. `std::unordered_multimap<T>`(`hicc/std/unordered_map.hpp`)
1. `std::array<T>`(`hicc/std/array.hpp`)
1. `std::list<T>`(`hicc/std/list.hpp`)
1. `std::forward_list<T>`(`hicc/std/forward_list.hpp`)
1. `std::queue<T>`(`hicc/std/queue.hpp`)
1. `std::priority_queue<T>`(`hicc/std/queue.hpp`)
1. `std::stack<T>`(`hicc/std/stack.hpp`)

## 使用方式

注：`hicc-std`仅提供了`std::string`, `std::u16string`, `std::u32string`相关的构造函数.

因为都是模板类，需要使用者显式实现实例化模版类的构建函数. 参见如下代码:

```
use hicc::AbicClass;

hicc::cpp! {
    // c++侧需要引用hicc提供的头文件.
    #include <hicc/std/map.hpp>
    #include <hicc/std/string.hpp>

    // 按需定义容器类型. 可以包含非缺省的Allocator等模版参数类型.
    typedef std::map<int, std::string> CppMap;
}

hicc::import_lib! {
    #![link_name = "example"]

    // 对应`c++`的`CppMap`
    class RustMap = hicc_std::map<hicc::Pod<i32>, hicc_std::string>;

    // 创建容器接口.
    #[cpp(func = "std::unique_ptr<CppMap> hicc::make_unique<CppMap>()")]
    fn rustmap_new() -> RustMap;
}

fn main() {
    let mut map = rustmap_new();
    let name = hicc_std::string::from(c"hello");
    map.insert(&0, &name);
    assert_eq!(map.get(&1), None);
    assert_eq!(map.get(&0), Some(name.as_ref()));
}
```

**注意**: 
> 1. 模版参数类型只能是`c++`类或者可直接在`CABI`接口上传递使用的`POD`数据类型，后者只能结合`hicc::Pod<T>`使用.

3. `build.rs`编译`c++`代码

```rust
fn main() {
    hicc_build::Build::new().rust_file("src/main.rs").compile("example");
    println!("cargo::rustc-link-lib=example");
    // 根据平台选择正确的 C++ 标准库（macOS 使用 libc++，Linux/MinGW 使用 libstdc++）
    #[cfg(target_os = "macos")]
    println!("cargo::rustc-link-lib=c++");
    #[cfg(not(any(target_os = "macos", all(target_os = "windows", target_env = "msvc"))))]
    println!("cargo::rustc-link-lib=stdc++");
    println!("cargo::rerun-if-changed=src/main.rs");
}
```

**注意**: 需要最终构建为可执行程序或者动态库时指定所依赖的**`c++`标准库**.

## 迭代器接口说明

`c++`容器基于迭代器实现插入删除等接口违背`rust`的借用规则, `hicc-std`将迭代器做了二次封装，提供容器遍历和插入删除功能.

## 测试

`doc test`需要开启`test feature`, 提供了测试用例用到的容器实例化类型的构建函数.

```text
# cargo test --features "test"
```

