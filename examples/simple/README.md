# 简单示例：自由函数

该示例演示如何从 C++ 源码（含命名空间与重载函数）生成 Rust FFI。

## 源码文件

- `mylib.hpp`：`namespace mylib` 下的自由函数声明
- `mylib.cpp`：对应实现（需单独编译）

## 运行步骤

在仓库根目录执行：

```bash
# 第 1 步：生成分组 FFI
cpp2rust-demo init --link mylib -- clang -x c++ -fsyntax-only examples/simple/mylib.cpp

# 第 2 步：合并输出
cpp2rust-demo merge

# 第 3 步：查看结果
ls .cpp2rust/default/rust/src/
cat .cpp2rust/default/rust/src/merged_ffi.rs
```

## 预期生成结果

执行后可在 `merged_ffi.rs` 中看到类似内容：

```rust
hicc::import_lib! {
    #![link_name = "mylib"]

    #[cpp(func = "int mylib::add(int, int)")]
    fn add(a: i32, b: i32) -> i32;

    #[cpp(func = "double mylib::scale(double, double)")]
    fn scale(x: f64, factor: f64) -> f64;

    #[cpp(func = "int mylib::string_length(const char *)")]
    fn string_length(str: *const i8) -> i32;

    #[cpp(func = "int mylib::log_message(const char *, const char *)")]
    fn log_message(level: *const i8, msg: *const i8) -> i32;

    // Overloaded functions get a numeric suffix.
    #[cpp(func = "void mylib::process(int)")]
    fn process(value: i32);

    #[cpp(func = "void mylib::process(double)")]
    fn process_2(value: f64);

    #[cpp(func = "void mylib::process(const char *)")]
    fn process_3(value: *const i8);
}
```

## 使用生成项目进行编译

```bash
# 拷贝生成的 Rust 工程
cp -r .cpp2rust/default/rust/ mylib-ffi/
cd mylib-ffi/

# 编译 C++ 静态库（也可改为 .so）
clang++ -std=c++14 -c -fPIC ../../examples/simple/mylib.cpp -o mylib.o
ar rcs libmylib.a mylib.o

# 构建 Rust crate（需要让链接器可找到库）
LIBRARY_PATH=. cargo build
```

## 重载命名规则

`cpp2rust-demo` 默认通过追加数字后缀解决重载冲突，后缀从 `_2` 开始：

| C++ 重载 | Rust 名称 |
|---|---|
| `void process(int)` | `process` |
| `void process(double)` | `process_2` |
| `void process(const char*)` | `process_3` |

该策略实现位于 `src/ast.rs` 的函数提取逻辑中。
