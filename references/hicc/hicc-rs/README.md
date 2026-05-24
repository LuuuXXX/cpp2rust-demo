# hicc-rs

在Rust和C/C++混合编程的场景下，将已有的Rust接口封装为C接口供其他语言调用是一件费时费力的事情. hicc-rs提供这方面的帮助:

1. 支持将任意的Rust数据包括泛型类型包装为C接口
2. C接口也保留部分Rust的安全特性：区分了所有权，只读借用和可写借用：
> - 返回的借用不能传递给需要所有权的接口，否则会导致panic
> - 返回的只读借用不能传递给需要可写借用的接口，否则会导致panic
> - 其他安全需要接口调用者保证：调用者保证借用数据的有效性，调用者保证多写的并发安全.

其他约束：

1. 依赖`#![feature(speicialization)]`, 构建需要nightly版本或者设置`RUSTC_BOOTSTRAP=1`环境变量.
1. 最多支持4重指针: `Option<&&&&T>`或者`&&&&Option<T>`
1. 部分接口仅支持3重指针: `fn as_ref(&Option<T>) -> Option<&T>`, `Option<&&&&T>`不支持`as_ref`, 调用者需要判断函数指针是否为空.
> 备注: 实际对应的C接口被定义为`fn as_ref(&Option<T>) -> *const T`.

Rust类型的关联方法特别多，并不需要将每个方法都转换为C接口，需要根据C语言特点选择必须的几个方法即可. 

# 样例

参见`core`和`std`下的实现.

# 宏接口

计划在hicc-rs的基础上提供一套宏接口，进一步方便使用. 宏接口的大致形式如下:

```rust
#[export_rust]
impl<T> Option<T> {
    fn is_none(&self) -> bool;
    fn unwrap(self) -> T;
    fn as_ref(&self) -> *const T {
        self.as_ref().map(|v| v as *const T).unwrap_or(ptr::null())
    }
}
```

上面宏定义最终可以生成`hicc-rs/src/core/option.rs`中的等效代码.

如果不改变函数签名，只需要声明即可，如果改变函数签名, 需要完整实现.

