//! ## 使用方式
//!
//! 注：
//! 1. `hicc-std`仅提供了`std::string`, `std::u16string`, `std::u32string`相关的构造函数.
//! 1. `v0.2.0`版本, 通过增加参数校验，消除了原接口中可能抛出的各种异常.
//!    部分接口的返回值类型发生了变化.
//!
//! 因为都是模板类，需要使用者显式实现实例化模版类的构建函数. 参见如下代码:
//!
//! ```text
//! use hicc::AbicClass;
//!
//! hicc::cpp! {
//!     // c++侧需要引用hicc提供的头文件.
//!     #include <hicc/std/map.hpp>
//!     #include <hicc/std/string.hpp>
//!
//!     // 按需定义容器类型. 可以包含非缺省的Allocator等模版参数类型.
//!     typedef std::map<int, std::string> CppMap;
//! }
//!
//! hicc::import_lib! {
//!     #![link_name = "example"]
//!
//!     // 对应`c++`的`CppMap`
//!     class RustMap = hicc_std::map<hicc::Pod<i32>, hicc_std::string>;
//!
//!     // 创建容器接口.
//!     #[cpp(func = "std::unique_ptr<CppMap> hicc::make_unique<CppMap>()")]
//!     fn rustmap_new() -> RustMap;
//! }
//!
//! fn main() {
//!     let mut map = rustmap_new();
//!     let name = hicc_std::string::from(c"hello");
//!     map.insert(&0, &name);
//!     assert_eq!(map.get(&1), None);
//!     assert_eq!(map.get(&0), Some(name.as_ref()));
//! }
//! ```
//!
//! **注意**:
//! > 1. 模版参数类型只能是`c++`类或者可直接在`CABI`接口上传递使用的`POD`数据类型，后者只能结合`hicc::Pod<T>`使用.
//!
//! 3. `build.rs`编译`c++`代码
//!
//! ```text
//! fn main() {
//!     hicc_build::Build::new().rust_file("src/main.rs").compile("example");
//!     println!("cargo::rustc-link-lib=example");
//!     println!("cargo::rustc-link-lib=stdc++");
//!     println!("cargo::rerun-if-changed=src/main.rs");
//! }
//! ```
//!
//! `hicc_build`仅支持生成静态库, 需要最终构建为可执行程序或者动态库时指定所依赖的`c++`标准库.
//!
//! ## 迭代器接口说明
//!
//! `c++`容器基于迭代器实现插入删除等接口违背`rust`的借用规则, `hicc-std`将迭代器做了二次封装，提供容器遍历和插入删除功能.
//!
//! ## 测试
//!
//! `doc test`需要开启`test feature`, 提供了测试用例用到的容器实例化类型的构建函数.
//!
//! ```text
//! # cargo test --features "test"
//! ```
//!
mod std_string;
pub use std_string::*;

mod std_array;
pub use std_array::*;

mod std_deque;
pub use std_deque::*;

mod std_vector;
pub use std_vector::*;

mod std_stack;
pub use std_stack::*;

mod std_queue;
pub use std_queue::*;

mod std_list;
pub use std_list::*;

mod std_forward_list;
pub use std_forward_list::*;

mod std_set;
pub use std_set::*;

mod std_unordered_set;
pub use std_unordered_set::*;

mod std_map;
pub use std_map::*;

mod std_unordered_map;
pub use std_unordered_map::*;

#[cfg(feature = "test")]
mod std_test;
#[cfg(feature = "test")]
pub use std_test::*;

hicc::import_lib! {
    #![link_name = "hicc_std_string"]

    hicc::cpp! {
        #include <hicc/std/string.hpp>
    }

    class string;

    #[cpp(func = "std::unique_ptr<std::string> hicc::make_unique<std::string>()")]
    fn string_new() -> string;

    #[cpp(func = "std::unique_ptr<std::string> hicc::make_unique<std::string, const char*>(const char* &&)")]
    unsafe fn string_with_cstr(s: *const i8) -> string;

    #[cpp(func = "std::unique_ptr<std::string> hicc::make_unique<std::string, const char*, size_t>(const char* &&, size_t&&)")]
    unsafe fn string_with_buf(s: *const u8, len: usize) -> string;

    #[cpp(func = "std::unique_ptr<std::u16string> hicc::make_unique<std::u16string>()")]
    fn u16string_new() -> u16string;

    #[cpp(func = "std::unique_ptr<std::u16string> hicc::make_unique<std::u16string, const char16_t*, size_t>(const char16_t* &&, size_t&&)")]
    unsafe fn u16string_with_buf(s: *const u16, len: usize) -> u16string;

    #[cpp(func = "std::unique_ptr<std::u32string> hicc::make_unique<std::u32string>()")]
    fn u32string_new() -> u32string;

    #[cpp(func = "std::unique_ptr<std::u32string> hicc::make_unique<std::u32string, const char32_t*, size_t>(const char32_t* &&, size_t&&)")]
    unsafe fn u32string_with_buf(s: *const u32, len: usize) -> u32string;
}
