# 核心概念

## Rust中的C++类的定义

Rust不知道C++类的内存布局, 所以只能是通过指针访问C++类的实例对象. 但Rust中并不是用一个裸指针来代表C++对象，而是如下定义形式:

```rust!
struct CppObject {
    methods: &'static CppObjectMethods,
    obj: *const (),
}
```

它代表C++类型在Rust中的代理类型，对Rust开发者屏蔽只能通过指针访问C++对象的内部实现，同时还屏蔽了两种语言中MOVE语义的差异，C++函数中的`T&&`类型，在Rust中它完全等同于`T`类型.

针对如下C++函数:

```c++!
std::string foo(std::string n1, const std::string& n2, std::string&& n3, const std::string* n4);
```

其映射后的Rust函数可以是如下形式:

```rust!
type CppObject = hicc_std::string;
fn foo(n1: CppObject, n2: &CppObject, n3: CppObject, n4: *const CppObject) -> CppObject;
```

C++指针可能为空，所有的CppObject都实现了AbiClass Trait，可以通过它定义的接口判断是否是空指针:

```rust!
impl AbiClass for CppObject {
    fn is_null(&self) -> bool { ... }
    fn write(val: &CppObject) { ... }
    ...
}
```

其中`write`接口等价于C++类的赋值操作，因为不能简单通过Rust的可写借用来覆写C++对象.

```rust!
use hicc::AbiClass;

let mut obj: CppObject = ...;
let robj = &mut obj;
let val: CppObject = ...;
// 等他于C++的obj = val
robj.write(&val);
// 释放obj指向的C++对象，重新指向val对应的C++对象.
*robj = val;
```

## 指针和引用返回类型

如果返回类型是个C++类的指针或者引用，其对应的Rust函数仍然可用CppObject来表示.

```c++!
std::string* string_new1();
std::unique<std::string> string_new2();
const std::string& string_new3();
```

其映射后的Rust函数可以是如下形式:

```rust!
type CppObject = hicc_std::string;
fn string_new1() -> CppObject;
fn string_new2() -> CppObject;
fn string_new3() -> CppObject;
```

但和C++函数类型不一致，不易用. 为了和C++接口返回值类型保持一致，Rust中也定义指针和引用类型.

前面例子中的Rust函数可以优化为如下形式:

```rust!
type CppObject = hicc_std::string;
fn string_new1() -> ClassMutPtr<'static, CppObject>;
fn string_new2() -> ClassMutPtr<'static, CppObject>;
fn string_new3() -> ClassRef<'static, CppObject>;
```

如下C++函数，对于多重指针的返回值和参数用`ClassPtr<'a, T, N>`来映射.

```c++!
std::string** string_array_clone(const std::string** in, size_t len);
```

映射后的Rust函数可以是如下形式:

```rust!
fn string_array_clone(in: ClassPtr<'_, CppObject, 2>, len: usize) -> ClassMutPtr<'static, CppObject, 2>;
```

`ClassPtr`,`ClassMutPtr`,`ClassRef`,`ClassRefMut`都实现了AbiClass Trait. 此Trait还定义了这些类型相互转换的接口.

**注意**: 如果`std::unique_ptr<T, D>`其中的`D`非缺省模板参数，则对应到`hicc::unique_ptr<T>`.

## 消除C++函数的内存安全风险

考虑如下C++接口定义:

```c++!
template<class T, class Alloc>
class vector {
    const_iterator begin() const { ... };
};
```

`vector<T>::begin`的返回值在`vector<T>`被修改后可能失效，导致内存安全问题.

Rust中为了消除这类内存安全的风险，其对应的Rust接口可以映射为如下形式:

```rust!
type CppObject = CppVecIter;
struct vector<T> {
    fn begin(&self) -> ClassRef<'_, CppObject> { ... }
}
```

利用Rust生命周期，则可以消除原C++接口的内存安全风险.

# 教程

## Rust中定义C++函数接口.

```rust!
hicc::cpp! {
    #include <hello.hpp>
}

hicc::import_lib! {
    #![link_name = "example"]
    
    #[cpp(func = "void hello_world()")]
    fn hello_world();
}
```

`hicc::cpp`中可以包含C++代码，一般用来包含依赖的C++头文件.

可定义多个`hicc::import_lib`代码块，需要保证每个代码块中的`link_name`全局唯一.

Rust函数和C++函数一一对应，C++函数通过宏`#[cpp(func = ...)]`来声明.

|正确声明|错误声明|
|---|---|
|`ret_type func_name(arg_type, ...)`|`ret_type func_name(arg_type arg_name, ...)`|
||`func_name`|

**注意**: 需要开发者保证声明类型和实际类型的一致性.  C++函数声明中，参数列表只包含参数类型，**不能**包含参数名. 

## Rust中调用C++模板函数

```rust!
hicc::cpp! {
    #include <string>
    #include <memory>
}

hicc::import_lib! {
    #![link_name = "example"]

    #[cpp(func = "std::unique_ptr<std::string> std::make_unique<std::string, const char*>(const char*&&)")]
    unsafe fn string_from(s: *const i8) -> hicc_std::string;
}
```

**注意**: 需要开发者保证声明类型和实际类型的一致性.  模板函数的正确声明形式如下:

|正确声明|错误声明|
|---|---|
|`ret_type func_name<arg_type, ...>(arg_type, ...)`|`ret_type func_name<arg_type, ...>(arg_type arg1_name, ...)`|
||`ret_type func_name(arg_type, ...)`|
||`func_name<arg_type, ...>`|

Rust函数是否是`unsafe`需要由开发者决定.

## Rust中定义C++类接口.

```rust!
hicc::cpp! {
    #include <string>
}

hicc::import_class! {
    #[cpp(class =  "std::string")]
    class string {
        #[cpp(method = "const char* c_str() const")]
        fn c_str(&self) -> *const i8;
    }
}
```

`hicc::import_class`代码块中可以包含多个`class`定义. `class`对应到Rust的`struct`.

`#[cpp(class = ...)]`声明C++中的类类型名, `#[cpp(method = ...)]`声明类的成员函数，**对其声明形式的要求同全局函数**.

`class`中只能声明C++类的成员函数，对于构造函数以及类的静态函数，应该在`hicc::import_lib`中声明，但可以用Rust代码包装到Rust的关联函数中:

```rust!
hicc::cpp! {
    #include <string>
    #include <memory>
}

hicc::import_class! {
    #[cpp(class =  "std::string")]
    class string {
        #[cpp(method = "const char* c_str() const")]
        fn c_str(&self) -> *const i8;
        
        unsafe fn new(s: *const i8) -> Self {
            unsafe { string_new(s) }
        }
    }
}

hicc::import_lib! {
    #![link_name = "example"]

    #[cpp(func = "std::unique_ptr<std::string> std::make_unique<std::string, const char*>(const char*&&)")]
    unsafe fn string_new(s: *const i8) -> string;
}
```

也可以用一个宏`#[method(class = ..., name = ...)]`实现上面的包装功能:

```rust!
hicc::cpp! {
    #include <string>
    #include <memory>
}

hicc::import_class! {
    #[cpp(class =  "std::string")]
    class string {
        #[cpp(method = "const char* c_str() const")]
        fn c_str(&self) -> *const i8;
    }
}

hicc::import_lib! {
    #![link_name = "example"]

    #[cpp(func = "std::unique_ptr<std::string> std::make_unique<std::string, const char*>(const char*&&)")]
    #[method(class = string, name = new)]
    unsafe fn string_new(s: *const i8) -> string;
}
```

如果右值引用的成员函数，则对应到Rust中的`self`，行为等同于所有权转移.

```rust!
hicc::cpp! {
    #include <string>

    class Foo {
    public:
        std::string foo() &&;
    };
}

hicc::import_class! {
    #[cpp(class = "Foo")]
    class Foo {
        #[cpp(method = "std::string foo() &&")]
        fn name(self) -> hicc_std::string;
    }
}
```

## Rust中定义C++模板类

可参考[hicc-std](https://gitcode.com/xuanwu/hicc/tree/main/hicc-std)的实现.

这里定义C++的`std::vector`:

```rust!
hicc::cpp! {
    #include <vector>
}

hicc::import_class! {
    #[cpp(class = "template<class T, class Allocator> std::vector<T, Allocator>")]
    pub class vector<T> {
        #[cpp(method = "bool is_empty() const")]
        pub fn is_empty(&self) -> bool;

        #[cpp(method = "const T& back() const")]
        fn _back(&self) -> &T;

        pub fn back(&self) -> Option<T::OutputRef<'_>> {
            if self.is_empty() {
                return None;
            }
            Some(self._back())
        }
    }
}
```

**注意**: `#[cpp(class = ...)]`声明对应的C++模板类, 对模板类的生命形式要求如下:

|正确声明|错误声明|
|---|---|
|`template<class T, class Allocator> std::vector<T, Allocator>`|`template<class T, class Allocator> std::vector`|
||`std::vector<T, Allocator>`|

因为`std::vector<T, Allocator>`的成员函数的类型只由`T`决定，因此Rust中只需要定义泛型参数`T`.

Rust中最终生成的泛型是如下形式:

```rust!
struct vector<T: AbiType> {
    ...
}
```

每个泛型类型都必须实现`AbiType`, 所有`class`生成的类型都支持`AbiType`, 对应的是C++类类型. 因此可以如下使用:

```rust!
type VecString = vector<hicc_std::string>;
type Vec3String = vector<hicc_std::set<vector<hicc_std::string>>>;
```

如果模板的实例化类型不是C++类类型，是一个POD类型，则在Rust侧需要利用`hicc::Pod`来实例化.

```rust!
type VecI32 = vector<hicc::Pod<i32>>;
```

对于直接映射C++成员函数的Rust函数，如果参数和返回类型是泛型`T`或者其引用或者其指针类型，则会被转换为`AbiType::Output`或`AbiType::OutputRef`或`AbiType::OutputPtr`. 

如果是开发者自己写的函数(比如这里例子中的`back`函数)需要显示使用`AbiType`定义的对应类型.

同样, 模板类构造函数只能在`hicc::import_lib`中定义.

## 自动适配引用和指针类型

如下代码:

```rust!
hicc::cpp! {
    #include <string>
    const std::string& string_ref(const std::string& in) { return in; }
}

hicc::import_class! {
    #[cpp(class = "std::string")]
    class string { }
}

hicc::import_lib! {
    #![link_name = "example"]

    #[cpp(func = "const std::string& string_ref(const std::string&)")]
    fn string_ref(val: &string) -> ClassRef<'_, string>;
    // 返回值不能是&string
    // fn string_ref(val: &string) -> &string;
}
```

对于返回类型是C++类的指针或者引用的函数，映射为Rust的函数，返回类型需要正确设置，以上不能是`&string`.

`hicc::import_lib`和`hicc_import_class`提供了自动将返回值`&T`转换为`ClassRef<'_, T>`的功能，前提是开发者需要利用`class <ident> = <path>`来说明哪些类型是C++类型.

上面的代码可以改写如下:

```rust!
hicc::cpp! {
    #include <string>
    const std::string& string_ref(const std::string& in) { return in; }
}

hicc::import_class! {
    #[cpp(class = "std::string")]
    class string { }
}

hicc::import_lib! {
    #![link_name = "example"]

    // 本代码块的string代表C++类型.
    class string;

    #[cpp(func = "const std::string& string_ref(const std::string&)")]
    // 宏展开后返回值变为`ClassRef<'_, string>`
    fn string_ref(val: &string) -> &string;
}
```

`hicc::import_class`中也适用这个规则. 比如:

```rust!
hicc::import_class! {
    // Bar代表C++类型
    class Bar = bar::Bar;

    #[cpp(class = "Foo")]
    class Foo {
        // 宏展开后返回值变为`ClassRef<'_, Bar>`
        #[cpp(method = "const bar::Bar& bar() const")]
        fn bar(&self) -> &Bar;
    }
}
```

对于多重指针，还可以自动适配函数参数类型.

# 构建方式

`hicc::import_class`和`hicc::import_lib`生成的cpp代码需要在build.rs中利用hicc-build来构建.

```text
    hicc_build::Build::new().rust_file("src/main.rs").compile("example");
    println!("cargo::rustc-link-lib=example");
    println!("cargo::rustc-link-lib=stdc++");
    println!("cargo::return-if-changed=src/main.rs");
```

利用`rust_file`加入所有包含`hicc::impot_class`,`hicc::import_lib`,`hicc::cpp`宏的rust文件，他们会编译为一个静态库，然后将此静态库链接到rust二进制件中.

**注意**:

1. C++编译器支持c++11或以后版本.
2. 需要链接C++标准库.

# Rust调用C++

可参考[examples](https://gitcode.com/xuanwu/hicc/tree/main/examples).

## 忽略C++函数的缺省参数.

```rust!
hicc::cpp! {
    int foo(int v1, int v2 = 1) { return v1 + v2; }
}

hicc::import_lib! {
    #![link_name = "example"]

    #[cpp(func = "int foo(int, int)")]
    fn foo(v: i32) -> i32;
}

fn main() {
    let r = foo(1);
}
```

**注意**:
1. C++函数的声明必须包含完整的参数类型列表.
1. Rust函数的参数无需定义需忽略的缺省参数.
1. 如果忽略的不是缺省参数会有编译错误.

## 忽略C++函数的返回值

```rust!
hicc::cpp! {
   int foo(int& v1, int v2) { v1 += v2; return v1; }
}

hicc::import_lib! {
    #![link_name = "example"]

    #[cpp(func = "int foo(int&, int)")]
    fn foo(v1: &mut i32, v2: i32);
}
fn main() {
    let mut n = 0;
    foo(&mut n, 1);
}
```

只需要Rust函数忽略返回值即可. 如果忽略的返回值类型是C++类，则对性能有利.

**注意**: Rust函数返回类型不要使用`ffi::c_void`类型.

## 捕获C++异常.

```rust!
hicc::cpp! {
    int foo(int v1) { throw "exception"; return v1; }
}

hicc::import_lib! {
    #![link_name = "example"]

    #[cpp(func = "int foo(int)")]
    fn foo(v: i32) -> hicc::Exception<i32>;
}

fn main() {
    if foo(1).ok().is_err() {
    }
}
```

`hicc::Exception`接受C++异常，尽力将C++异常转换为可读的字符串信息.

## 传递`std::function`类型.

和Rust的闭包对应. 参见代码[main.rs](https://gitcode.com/xuanwu/hicc/blob/main/examples/functional/src/main.rs).

## 传递可变参数

```rust!
hicc::cpp! {
    #include <string>
    #include <cstdarg>
    void foo(const std::string& name, va_list args) { ... }
}

hicc::import_lib! {
    #![link_name = "example"]

    #[cpp(func = "void foo(const std::string&, va_list)")]
    unsafe fn foo(name: &hicc_std::string, ...);
}

fn main() {
    // 注意调用方式
    unsafe { foo()(&hicc_std::string::from(c"hello"), 0) };
}
```

**说明**:

1. 生成的rust函数是一个无参数的函数返回变长参数的函数指针.
1. 如果C++函数最后参数是`va_list`, 则其他参数和函数返回值可以是C++类相关类型.
2. 如果C++函数最后参数是`...`, 则其他参数和函数返回值**不能是C++类相关类型**.


## 继承C++抽象类

参见代码[main.rs](https://gitcode.com/xuanwu/hicc/blob/main/examples/interface/src/main.rs).

**说明**:

1. `#[interface]`转换为Rust的Trait, 其中只能包括C++类中的(纯)虚函数. 可将C++类包括其继承的所有虚函数汇总定义在一起.
2. `@make_proxy`是一个内置函数，利用组合模式实现C++继承的功能行为. 必须结合宏`#[interface(name = ...)]`使用.
3. `@make_proxy`创建的类型必须指定构造函数`#[cpp(class = ..., ctor = ...)]`, 构造函数的参数类型列表必须和`@make_proxy`的参数类型列表一致, 而对应的Rust函数多一个输入参数，即Rust的实现类, 其参数类型必须是`hicc::Interface<T>`.

## 调用C++的`dynamic_cast`

参见代码[main.rs](https://gitcode.com/xuanwu/hicc/blob/main/examples/dynamic_cast/src/main.rs).

**说明**:
1. `@dyanmc_cast`内置函数，Rust侧不支持泛型，需要针对具体实现类型定义其转换接口.

## 私有析构的C++类

参见代码[main.rs](https://gitcode.com/xuanwu/hicc/blob/main/examples/destroy/src/main.rs).

**说明**:

1. C++类的私有析构函数需要利用`#[cpp(class = ..., destroy = ...)]`定义.

## 读写C++变量

参见代码[main.rs](https://gitcode.com/xuanwu/hicc/blob/main/examples/datas/src/main.rs).

**说明**:

1. C++类的成员变量利用`[cpp(field = ...)]`声明.
1. C++类的全局变量，类的静态变量利用`[cpp(data = ...)]`声明.
1. 接口都是返回变量的引用，只读或者可写由Rust接口决定，是否unsafe也有Rust接口决定.
1. 对任何数据类型，Rust中提供返回C++数据只读借用或者可写借用的接口.

## C++容器保存Rust数据.

参见代码[main.rs](https://gitcode.com/xuanwu/hicc/blob/main/examples/rust_any/src/main.rs).

**说明**:

1. 对于`map/set`，要求key支持`std::less`，对应到`RustKey`.
1. 对于`unordered_map/unordered_set`，要求key支持`std::hash`，对应到`RustHashKey`.
1. 对于`vector`等其他了日常要求支持`Clone trait`.
1. C++侧只有一个类型就是`RustAny`，如果Rust中的使用和以上约束不一致，会在运行时panic.

## Rust内存空间创建C++对象

参见代码[main.rs](https://gitcode.com/xuanwu/hicc/blob/main/examples/placement_new/src/main.rs).

**说明**:

1. Rust接口的返回值应该是指针或者引用，其生命周期应该和输入的内存空间的生命周期相关联.

## `hicc::cpp`支持灵活适配

因为`hicc`功能约束或者缺陷导致某些C++接口无法自动适配，开发者可利用`hicc::cpp`直接在Rust文件中实现可用的适配函数.

```rust!
hicc::import_class! {
    #[cpp(class = "Foo")]
    class Foo {
        hicc::cpp! {
            static void bar(const Self& self) {
                self.bar(100);
            }
        }
        #[cpp(func = "void SelfMethods::bar(const Self&)")]
        fn bar(&self);
    }
}
```

**注意**:

1. 如在`class`中定义，只能是静态函数，静态函数所在的类空间名是`SelfMethods`, 具体的C++类型可由`Self`引用.
2. 因为是成员函数，首参数必须是`Self`或其引用或其指针.
