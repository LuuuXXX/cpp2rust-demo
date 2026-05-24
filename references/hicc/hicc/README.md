# hicc

使用方式参考[使用指南](https://gitcode.com/xuanwu/hicc/blob/main/reference.md).

## 支持的功能介绍

### `v0.2.3`版本 

修正依赖的`hicc-build`的版本号错误问题.

### `v0.2.2`版本 

支持在rust的内存空间中构造C++类对象. 参见`examples/placement_new`.

### `v0.2.1`版本

新增`RustAny`支持`c++ stl`容器存储`rust`数据. 参见`examples/rust_any`

### `v0.2.0`版本

支持自动生成`c++`适配代码，开发人员只需要提供`rs`文件, 进一步方便使用. 

**约束**: 依赖`c++11`及更高版本的特性.

### 支持的数据类型.

|数据类型|是否支持|备注|
|---|---|---|
|`T`|support||
|`const T&`|support||
|`T&`|support||
|`T&&`|support|`rust`侧同`T`|
|`const T*`|support|程序员管理资源生命周期|
|`T*`|support|程序员管理资源生命周期|
|`const T**`多重指针|support|程序员管理资源生命周期|
|`T**`多重指针|support|程序员管理资源生命周期|
|`std::function<R(ArgTypes...)>`|support||

**说明**:
1. **字节对齐要求**: 要求所有`c++`对象都至少按照`size_t`字节数进行对齐, 未对齐地址被视为非法指针.

### 支持的函数类型.

|函数分类|是否支持|
|---|---|
|funtion`(external linkage/internal linkage/no linkage)`|support|
|function overloading|support|
|default parameters|support|
|template function|partial support(需实例化)|
|class member function|support|
|`noexcept(false)`|support|
|**template class**|support|
|`dynamic_cast`|support|
|realizing virtual function with rust|support|
|`va_list`|support|
|variadic(`...`)|partial support(仅全局函数，参数和返回值无类类型)|

### 使用样例

## `hello_world`

`hicc::cpp`可嵌入`c++`代码, `hicc::import_lib`定义需要调用的`c++` api.

```text
hicc::cpp! {
    #include <iostream>
    static void hello_world() {
        std::cout << "hello world!" << std::endl;
    }
}

hicc::import_lib! {
    #![link_name = "example"]
    
    #[cpp(func = "void hello_world()")]
    fn hello_world();
}

fn main() {
    hello_world();
}
```

最终还需要利用`build.rs`编译此文件:

```text
    hicc_build::Build::new().rust_file("src/main.rs").compile("example");
    println!("cargo::rustc-link-lib=example"); 
    println!("cargo::rustc-link-lib=stdc++"); 
    println!("cargo::return-if-changed=src/main.rs"); 
```

## 缺省参数和忽略返回值

如下场景，对于非变长参数的`c++`函数，允许映射不同的`rust`函数类型:

1. `rust`函数参数可以不包括`c++`函数的缺省参数
2. `rust`函数可以忽略`c++`函数的返回值.
2. `rust`函数可以将返回值包装在`hicc::Exception`中捕获`c++`异常.

```text
hicc::cpp! {
    #include <iostream>
    static int foo(int v1, int v2 = 0) {
        std::cout << "foo(v1 = " << v1 << ", v2 = " << v2 << ")" << std::endl;
        throw 3;
        return v1 + v2;
    }
}

hicc::import_lib! {
    #![link_name = "example"]
    
    #[cpp(func = "int foo(int, int)")]
    fn foo(v: i32) -> hicc::Exception<()>;
}

fn main() {
    println!("{:?}", foo(1).ok());
}
```

## `stl example`

`hicc::import_class`中的关键字`class`将`c++`的类映射为`rust`的`struct`. 无函数体的方法利用`#[cpp(method = ...)]`映射为对应的`c++`类的成员函数.

```text
use std::ffi::CStr;

hicc::cpp! {
    #include <string>
    static std::string hello_world() {
        return "hello_world";
    }
}

hicc::import_class! {
    #[cpp(class = "std::string")]
    class string {
        #[cpp(method = "const char* c_str() const")]
        fn c_str(&self) -> *const i8;
        fn as_cstr(&self) -> &CStr {
            unsafe { CStr::from_ptr(self.c_str()) } 
        }
    }
}

hicc::import_lib! {
    #![link_name = "example"]

    class string;
    
    #[cpp(func = "std::string hello_world()")]
    fn hello_world() -> string;
}

fn main() {
    let hello = hello_world();
    println!("{:?}", hello.as_cstr());
}

```

## `hicc-std`使用样例.

注: `hicc-std`提供了对`stl`容器的完整支持.

```rust!
use hicc::AbiClass;

hicc::cpp! {
    #include <hicc/std/map.hpp>
    #include <hicc/std/string.hpp>
    // 按需定义容器类型.
    typedef std::map<int, std::string> CppMap;
}

hicc::import_lib! {
    #![link_name = "example"]

    // 对应`c++`的`MyMap`
    class RustMap = hicc_std::map<hicc::Pod<i32>, hicc_std::string>;
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

## `stl`容器存储`rust`数据.

业务上需要用`stl`容器存储`rust`的数据，需要将`rust`数据转换为对应的`RustAny`,`RustKey`,`RustHashKey`类型.

```rust!
hicc::cpp! {
    #include <hicc/std/map.hpp>
    #include <hicc/std/unordered_set.hpp>
    #include <hicc/std/vector.hpp>
    #include <hicc/std/array.hpp>
    #include <hicc/std/string.hpp>
    #include <hicc/rust_any.hpp>
    // 按需定义容器类型.
    typedef std::map<RustAny, std::string> CppMap;
    typedef std::unordered_set<RustAny> CppSet;
    typedef std::vector<RustAny> CppVec;
    typedef std::array<RustAny, 5> CppArray;
}

hicc::import_lib! {
    #![link_name = "example"]

    // 对应`c++`的容器类型定义.
    class RustMap = hicc_std::map<hicc::RustKey<Key>, hicc_std::string>;
    class RustSet = hicc_std::set<hicc::RustHashKey<Key>>;
    class RustVec = hicc_std::vector<hicc::RustAny<Key>>;
    class RustArray = hicc_std::array<hicc::RustAny<Key>>;
    // 容器创建接口.
    #[cpp(func = "std::unique_ptr<CppMap> hicc::make_unique<CppMap>()")]
    fn rustmap_new() -> RustMap;
    #[cpp(func = "std::unique_ptr<CppSet> hicc::make_unique<CppSet>()")]
    fn rustset_new() -> RustSet;
    #[cpp(func = "std::unique_ptr<CppVec> hicc::make_unique<CppVec>()")]
    fn rustvec_new() -> RustVec;
    #[cpp(func = "std::unique_ptr<CppArray> hicc::make_unique<CppArray>()")]
    fn rustarray_new() -> RustArray;
}

// 希望保存的Rust数据.
#[derive(Clone, PartialEq, PartialOrd, Hash)]
struct Key {
    val: i32,
    key: i32,
}

impl Key {
    fn new(val: i32, key: i32) -> Self {
        Self{ val, key }
    }
}

fn main() {
    let mut map = rustmap_new();
    let mut set = rustset_new();
    let mut vec = rustvec_new();
    let mut array = rustarray_new();

    let key = hicc::RustAny::new_clone(Key::new(1, 1));

    vec.push_back(&key);
    let back = vec.back().unwrap();
    println!("back.key = {}", back.key);

    // 不能直接访问缺省构造的`RustAny`.
    array.fill(&key);
    let val1 = array.get(1).unwrap();
    println!("array[1].key = {}", val1.key);

    let k1 = hicc::RustHashKey::new_clone(Key::new(1, 1));
    set.insert(&k1);
    println!("set.count = {}", set.count(&k1));

    let k2 = hicc::RustKey::new_clone(Key::new(1, 1));
    map.insert(&k2, &hicc_std::string::from(c"hello"));
    println!("map.count = {}", map.count(&k2));
}
```

## 继承`c++`抽象类

同样利用`hicc::import_class`实现`c++`抽象类的映射(当前不支持多继承).

`#[interface]`声明这是一个纯抽象类，最终会映射为`rust`的`trait`.

利用内置函数`@make_proxy`和宏`#[interface(name = ...)]`, 基于组合模式，提供`Rust`继承`c++`抽象类的相似效果.

对于需要利用`@make_proxy`创建的类，必须在定义时提供`c++`构造函数的定义，下面例子中的:
```text
    #[cpp(class = "Baz", ctor = "Baz()")]
    class Baz: Bar { 
        #[cpp(method = "void baz() const")]
        fn baz(&self);
    }
```

```text
hicc::cpp! {
    #include <hicc/std/memory.hpp>
    #include <iostream>
    struct Foo {
        virtual ~Foo() {};
        virtual void foo() const = 0;
    };

    struct Bar: public Foo {
        virtual void bar() const = 0;
    };

    struct Baz: public Bar {
        virtual void foo() const override {
            std::cout << "C++ Baz::foo" << std::endl;
        }
        virtual void bar() const override {
            std::cout << "C++ Baz::bar" << std::endl;
        }
        void baz() const {
            std::cout << "C++ Baz::baz" << std::endl;
        }
        ~Baz() {
            std::cout << "C++ Baz::~Baz" << std::endl;
        }
    };
}

hicc::import_class! {
    #[interface]
    class Foo {
        #[cpp(method = "void foo() const")]
        fn foo(&self);
    }

    #[interface]
    class Bar: Foo {
        #[cpp(method = "void bar() const")]
        fn bar(&self);
    }

    #[cpp(class = "Baz", ctor = "Baz()")]
    class Baz: Bar { 
        #[cpp(method = "void baz() const")]
        fn baz(&self);
    }
}

hicc::import_lib! {
    #![link_name = "example"]

    class Baz;

    #[cpp(func = "Baz @make_proxy<Baz>()")]
    #[interface(name = "Bar")]
    fn new_rust_baz(intf: hicc::Interface<Baz>) -> Baz;

    #[cpp(func = "std::unique_ptr<Baz> std::make_unique<Baz>()")]
    fn new_cpp_baz() -> Baz;
}

struct RustBaz;

impl Bar for RustBaz {
    fn bar(&self) {
        println!("Rust Baz::bar");
    }
}

impl Foo for RustBaz {
    fn foo(&self) {
        println!("Rust Baz::foo");
    }
}

impl Drop for RustBaz {
    fn drop(&mut self) {
        println!("Rust Baz::~Baz");
    }
}

fn main() {
    let cpp_baz = new_cpp_baz();
    cpp_baz.foo();
    cpp_baz.bar();
    cpp_baz.baz();

    let rust_baz = new_rust_baz(RustBaz);
    rust_baz.foo();
    rust_baz.bar();
    rust_baz.baz();
}
```

## 开发指南.

详细内容参见`cargo doc`生成的文档.

