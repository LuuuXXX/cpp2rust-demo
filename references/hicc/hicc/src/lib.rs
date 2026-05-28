//!
//! `hicc`提供调用`c++`接口的能力.
//!
//! 三个相关的库:
//! 1. `hicc`核心功能库.
//! 2. `hicc-std`为`c++`标准库中的容器提供`rust api`.
//! 3. `hicc-build`方便编译`c++`适配代码.
//!
//! **已知约束:
//! 1. 依赖`c++11`或更高版本的特性.**
//! 2. 不直接支持操作符重载函数,需要利用`hicc::cpp`封装
//!
//! ## 类型映射通用规则
//!
//! 使用者需要了解映射规则, 特别是`c++`类作为函数返回值时映射后的类型.
//!
//! `hicc::import_class`和`hicc::import_lib`宏会基于映射规则更改函数的参数和返回值类型，新类型支持自动解引用.
//!
//! 在自动解引用无法支持的场景需要显式调用相关接口实现类型转换.
//!
//! ### 函数返回类型映射规则.
//!
//! | `c++`返回类类型 | `rust`对应类型 |
//! |---|---|
//! |T|T|
//! |`T&&`|T|
//! |`std::unique_ptr<T>`|T|
//! |`std::unique_ptr<T, D>`|`hicc::unique_ptr<T>`|
//! |`std::shared_ptr<T, D>`|`hicc::shared_ptr<T>`|
//! |`const T&`|`hicc::ClassRef<'_, T>`|
//! |`T&`|`hicc::ClassRefMut<'_, T>`|
//! |`T*`|`hicc::ClassMutPtr<'_, T, 1>`|
//! |`const T*`|`hicc::ClassPtr<'_, T, 1>`|
//!
//! 注意:
//! 1. `hicc::ClassPtr<'_, T, N = 1>`支持多重指针的，所有的指针映射规则都相同,
//!    作为返回值时，其生命周期参数为`'static`.
//! 1. 所有`c++`类映射类型`T`都支持`hicc::AbiClass` trait.
//! 1. `T`, `ClassRef<'_, T>`, `ClassRefMut<'_,
//!    T>`, `ClassPtr<'_, T>`, `ClassMutPtr<'_, T>`这些类型内存布局相同，可相互转换.
//!
//! 合理使用`ClassRef<'_, T>`作为返回值类型，因为携带了生命周期参数，可消除`c++`接口存在的内存安全问题. 典型的是`stl`容器的迭代器接口,如下所示:
//!
//! ```text
//! hicc::import_class! {
//!     #[cpp(class = "std::vector<int>")]
//!     struct IntVec {
//!         #[cpp(method = "typename Self::const_iterator begin() const")]
//!         fn begin(&self) -> &IntVecIter;
//!     }
//!
//!     #[cpp(class = "std::vector<int>::const_iterator")]
//!     struct IntVecIter {
//!     }
//! }
//! ```
//!
//! 注: `hicc::import_class`宏会将返回类型`&IntVecIter`修改为`hicc::ClassRef<'_, IntVecIter>`.
//!
//! 上面例子中，将`rust`接口中返回值的类型修改为引用类型，如此可以避免`c++`中因为修改容器导致修改前获取的迭代器失效带来的内存安全问题.
//!
//! **注意**: 这也是为什么不从`c++`头文件自动生成`rust api`的原因,
//! 直接映射会导致很多内存安全问题，**`rust api`需要重新设计**.
//!
//! ### 函数参数类型映射规则.
//!
//! |`c++`参数类型|`rust`参数类型|
//! |---|---|
//! |T|T|
//! |`T&&`|T|
//! |`std::unique_ptr<T>`|T|
//! |`std::unique_ptr<T, D>`|`hicc::unique_ptr<T>`|
//! |`std::shared_ptr<T, D>`|`hicc::shared_ptr<T>`|
//! |`const T&`|`&T`|
//! |`T&`|`&mut T`|
//! |`const T*`|`&hicc::ClassPtr<'_, T, 1>`|
//! |`T*`|`&hicc::ClassMutPtr<'_, T, 1>`|
//!
//! 注意`hicc::ClassPtr<'_, T, N = 1>`支持多重指针的，所有的指针映射规则都相同.
//!
//! ## 模板(泛型)参数类型映射规则
//!
//! `hicc`支持映射`c++`模板类，模版类型参数可能是`c++`类型或者是`POD`类型.
//! 同样`hicc::import_class`和`hicc::import_lib`可以自动转换和`c++`交互函数的参数和返回类型.
//! 如果需要为模版类添加`rust`方法时，应该显式使用映射后的类型.
//!
//! 对于模板参数，类型映射规则如下:
//!
//! |c++模板参数类型|`rust`参数|`rust`返回值|
//! |---|---|---|
//! |T|`<T as AbiType>::InputType`|`<T as AbiType>::OutputType`|
//! |T&&|`<T as AbiType>::InputType`|`<T as AbiType>::OutputType`|
//! |const T&|`&<T as AbiType>::InputType`|`<T as AbiType>::OutputRef<'_>`|
//! |T&|`&mut <T as AbiType>::InputType`|`<T as AbiType>::OutputRefMut<'_>`|
//! |const T*|`<T as AbiType>::InputPtr<'_>`|`<T as AbiType>::OutputPtr<'_>`|
//! |T*|`<T as AbiType>::InputMutPtr<'_>`|`<T as AbiType>::OutputMutPtr<'_>`|
//!
//! 参考`hicc-std::vector`的`get`接口定义:
//!
//! ```text
//! impl<T: AbiType + 'static> vector<T> {
//!     pub fn get(&self) -> Option<T::OutputRef<'_>> {
//!         //...
//!     }
//! }
//! ```
//!
//! **非`class`类型用于`rust`泛型**，必须结合`hicc::Pod<T>`使用.
//!
//! 参考`hicc-std::basic_string<T>`的相关定义:
//!
//! ```text
//! type string = basic_string<hicc::Pod<i8>>;
//! type u16string = basic_string<hicc::Pod<i16>>;
//! type u32string = basic_string<hicc::Pod<i32>>;
//! ```
//!
//! ## 使用样例
//!
//!
//! ### `hello_world`
//!
//! `hicc::cpp`可嵌入`c++`代码, `hicc::import_lib`定义需要调用的`c++` api.
//!
//! ```text
//! hicc::cpp! {
//!     #include <iostream>
//!     static void hello_world() {
//!         std::cout << "hello world!" << std::endl;
//!     }
//! }
//!
//! hicc::import_lib! {
//!     #![link_name = "example"]
//!     
//!     #[cpp(func = "void hello_world()")]
//!     fn hello_world();
//! }
//!
//! fn main() {
//!     hello_world();
//! }
//! ```
//!
//! `build.rs`利用`hicc_buid::Build`编译此文件.
//!
//! ```text
//!     hicc_build::Build::new().rust_file("src/main.rs").compile("example");
//!     println!("cargo::rustc-link-lib=example");
//!     println!("cargo::rustc-link-lib=stdc++");
//!     println!("cargo::return-if-changed=src/main.rs");
//! ```
//!
//! ## 缺省参数和忽略返回值
//!
//! 如下场景，对于非变长参数的`c++`函数，允许映射不同的`rust`函数类型:
//!
//! 1. `rust`函数参数可以不包括`c++`函数的缺省参数
//! 2. `rust`函数可以忽略`c++`函数的返回值.
//! 2. `rust`函数可以将返回值包装在`hicc::Exception`中捕获`c++`异常.
//!
//! ```text
//! hicc::cpp! {
//!     #include <iostream>
//!     static int foo(int v1, int v2 = 0) {
//!         std::cout << "foo(v1 = " << v1 << ", v2 = " << v2 << ")" << std::endl;
//!         throw 3;
//!         return v1 + v2;
//!     }
//! }
//!
//! hicc::import_lib! {
//!     #![link_name = "example"]
//!     
//!     #[cpp(func = "int foo(int, int)")]
//!     fn foo(v: i32) -> hicc::Exception<()>;
//! }
//!
//! fn main() {
//!     println!("{:?}", foo(1).ok());
//! }
//! ```
//! ## `stl example`
//!
//! `hicc::import_class`中的关键字`class`将`c++`的类映射为`rust`的`struct`. 无函数体的方法利用`#[cpp(method = ...)]`映射为对应的`c++`类的成员函数.
//!
//! ```text
//! use std::ffi::CStr;
//!
//! hicc::cpp! {
//!     #include <string>
//!     static std::string hello_world() {
//!         return "hello_world";
//!     }
//! }
//!
//! hicc::import_class! {
//!     #[cpp(class = "std::string")]
//!     class string {
//!         #[cpp(method = "const char* c_str() const")]
//!         fn c_str(&self) -> *const i8;
//!         fn as_cstr(&self) -> &CStr {
//!             unsafe { CStr::from_ptr(self.c_str()) }
//!         }
//!     }
//! }
//!
//! hicc::import_lib! {
//!     #![link_name = "example"]
//!
//!     class string;
//!     
//!     #[cpp(func = "std::string hello_world()")]
//!     fn hello_world() -> string;
//! }
//!
//! fn main() {
//!     let hello = hello_world();
//!     println!("{:?}", hello.as_cstr());
//! }
//!
//! ```
//!
//! ## `hicc-std`使用样例.
//!
//! 注: `hicc-std`提供了对`stl`容器的完整支持.
//!
//! 业务依赖`std::map<int, std::string>`, 需要提供实例化类的构建函数即可.
//!
//! ```text
//! use hicc::AbiClass;
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
//! ## 继承`c++`抽象类
//!
//! 同样利用`hicc::import_class`实现`c++`抽象类的映射(当前不支持多继承).
//!
//! `#[interface]`声明这是一个纯抽象类，最终会映射为`rust`的`trait`.
//!
//! 利用内置函数`@make_proxy`和宏`#[interface(name = ...)]`, 基于组合模式，提供`Rust`继承`c++`抽象类的相似效果.
//!
//! 对于需要利用`@make_proxy`创建的类，必须在定义时提供`c++`构造函数的定义，下面例子中的:
//!
//! ```text
//!     #[cpp(class = "Baz", ctor = "Baz()")]
//!     class Baz: Bar {
//!         #[cpp(method = "void baz() const")]
//!         fn baz(&self);
//!     }
//! ```
//!
//! 完整样例:
//!
//! ```text
//! hicc::cpp! {
//!     #include <hicc/std/memory.hpp>
//!     #include <iostream>
//!     struct Foo {
//!         virtual ~Foo() {};
//!         virtual void foo() const = 0;
//!     };
//!
//!     struct Bar: public Foo {
//!         virtual void bar() const = 0;
//!     };
//!
//!     struct Baz: public Bar {
//!         virtual void foo() const override {
//!             std::cout << "C++ Baz::foo" << std::endl;
//!         }
//!         virtual void bar() const override {
//!             std::cout << "C++ Baz::bar" << std::endl;
//!         }
//!         void baz() const {
//!             std::cout << "C++ Baz::baz" << std::endl;
//!         }
//!         ~Baz() {
//!             std::cout << "C++ Baz::~Baz" << std::endl;
//!         }
//!     };
//! }
//!
//! hicc::import_class! {
//!     #[interface]
//!     class Foo {
//!         #[cpp(method = "void foo() const")]
//!         fn foo(&self);
//!     }
//!
//!     #[interface]
//!     class Bar: Foo {
//!         #[cpp(method = "void bar() const")]
//!         fn bar(&self);
//!     }
//!
//!     #[cpp(class = "Baz", ctor = "Baz()")]
//!     class Baz: Bar {
//!         #[cpp(method = "void baz() const")]
//!         fn baz(&self);
//!     }
//! }
//!
//! hicc::import_lib! {
//!     #![link_name = "example"]
//!
//!     class Baz;
//!
//!     #[cpp(func = "Baz @make_proxy<Baz>()")]
//!     #[interface(name = "Bar")]
//!     fn new_rust_baz(intf: hicc::Interface<Baz>) -> Baz;
//!
//!     #[cpp(func = "std::unique_ptr<Baz> std::make_unique<Baz>()")]
//!     fn new_cpp_baz() -> Baz;
//! }
//!
//! struct RustBaz;
//!
//! impl Bar for RustBaz {
//!     fn bar(&self) {
//!         println!("Rust Baz::bar");
//!     }
//! }
//!
//! impl Foo for RustBaz {
//!     fn foo(&self) {
//!         println!("Rust Baz::foo");
//!     }
//! }
//!
//! impl Drop for RustBaz {
//!     fn drop(&mut self) {
//!         println!("Rust Baz::~Baz");
//!     }
//! }
//!
//! fn main() {
//!     let cpp_baz = new_cpp_baz();
//!     cpp_baz.foo();
//!     cpp_baz.bar();
//!     cpp_baz.baz();
//!
//!     let rust_baz = new_rust_baz(RustBaz);
//!     rust_baz.foo();
//!     rust_baz.bar();
//!     rust_baz.baz();
//! }
//! ```

///
/// 提供映射`c++`全局函数的功能.
///
/// ## `#![link_name = ...]`
///
/// 必须定义`link_name`，必须保证多个`import_lib`的`link_name`不冲突.
///
/// ## `#[cpp(func = "...")]`
/// 每个`rust`函数都需要利用这个宏定义`c++`函数. **注意**
/// 1. 函数参数列表只能包含类型名.
/// 2. 对于模版函数也需要完整的函数返回值和参数列表.
/// 3. 函数类型和`c++`类型保持一致，除了如下情况:
/// > 1. `rust`函数可以忽略`c++`中的缺省参数.
/// > 2. `rust`函数可以忽略`c++`函数的返回值.
/// > 3. `rust`函数可以将返回值包装在`hicc::Exception`中捕获`c++`异常.
///
/// |`c++`函数|正确定义|错误定义|
/// |---|---|---|
/// |`void foo(int v)`|`#[cpp(func = "void foo(int)")]`|`#[cpp(func = "void foo(int v)")]`|
/// |`template<class T> T make()`|`#[cpp(func = "int make<int>()")]`|`#[cpp(func = "make<int>")]`|
///
/// ## `#[cpp(data = "...")]
/// 支持封装`c++`数据的访问，都是返回数据的地址，只读或者可写由`rust`函数来确定，需要使用者保证和数据实际读写属性的一致性.
///
/// ## `class`声明`c++`类型
///
/// 对于`c++`类的映射类型，函数参数和返回值需要按照映射规则进行自动转换，需要利用`class`告诉`import_lib`.
///
/// 需要自动映射的类型只能是引入到当前模块的类型名.
///
/// ```text
/// class string;
/// class vector = other::vector;
/// ```
///
/// ## `class`函数体语法
///
/// `class`声明支持带函数体的形式，可在宏内部直接定义关联函数(无`self`)和方法(有`self`).
///
/// - **关联函数(无`self`)**: 自动提取为`#[member]`全局函数, 生成`impl ClassName { fn ... }`.
/// - **方法(有`self`)**: 保留在类中, 生成`import_class!`调用.
///
/// **注意**:
/// 1. 关联函数必须使用`#[cpp(func = "...")]`标注实际`c++`函数.
/// 2. 关联函数中不能使用类的泛型参数,必须使用具体类型.
/// 3. 泛型类也同样支持, 非`class`类型的泛型参数需要用`hicc::Pod<T>`包裹.
///
/// ```text
/// hicc::import_lib! {
///     #![link_name = "example"]
///
///     class Foo;
///     class Generic<T>;
///
///     #[cpp(class = "Foo")]
///     class Foo {
///         #[cpp(method = "void bar() const")]
///         fn bar(&self);
///
///         #[cpp(func = "Foo* Foo::new_instance()")]
///         fn new() -> Foo;
///     }
///
///     #[cpp(class = "Generic")]
///     class Generic<T> {
///         #[cpp(method = "void display() const")]
///         fn display(&self);
///
///         #[cpp(func = "Generic<int>* hicc_new_generic_int()")]
///         fn new() -> Generic<hicc::Pod<i32>>;
///     }
/// }
/// ```
///
/// ## `#[member(class = ..., method = ...)]`
///
/// 可利用这个宏帮助生成本`crate`某个类的关联函数, 方便使用.
///
/// ```text
/// struct MyMap;
/// hicc::import_lib! {
///     #![link_name = "example")]
///
///     #[member(class = "MyMap", method = "new")]
///     #[cpp(func = "std::unique_ptr<std::map<int, int>> hicc::make_unique<std::map<int, int>>()")]
///     fn mapintint_new();
/// }
///
/// fn main() {
///     // 可以这样构建`std::map<int, int>`.
///     let mut map = MyMap::new();
/// }
/// ```
///
/// ## `hicc::cpp`
///
/// 可以嵌入`c++`代码, 在全局名字空间中.
///
/// ## 样例
///
/// ```text
/// hicc::cpp! {
///     #include <iostream>
///
///     int global_count = 0;
///
///     template<class T>
///     T foo(T v) {
///         std::cout << "foo(" << v << ")" << std::endl;
///         return v + global_count;
///     }
/// }
///
/// hicc::import_lib! {
///     #![link_name = "example"]
///
///     #[cpp(data = "global_count")]
///     fn global_count() -> &'static mut i32;
///
///     #[cpp(func = "int foo<int>(int)")]
///     fn foo_i32(v: i32) -> i32;
/// }
///
/// fn main() {
///     println!("{}", foo_i32(1));
///     *global_count() += 1;
///     println!("{}", foo_i32(2));
/// }
/// ```
///
/// ## 内置函数
///
/// 用于支持`c++`类类型相关操作.
///
/// 1. `T1 @dynamic_cast<T1>(T2)`
///
/// 用于支持`c++`的`dynamic_cast`操作.
///
/// ```text
/// hicc::import_lib! {
///     #![link_name = "example"]
///     class Baz;
///     class Boo;
///
///     #[cpp(func = "Boo* @dynamic_cast<Boo*>(Baz*)")]
///     fn baz_boo(baz: &Baz) -> *const Boo;
/// }
/// ```
///
/// 2. `T @make_proxy<T,...>(...)`
///
/// 用于将`rust`对象转换为`c++`抽象类的子类对象. 参见有个继承的样例.
/// 使用此接口必须包含头文件`hicc/std/memory.hpp`.
///
/// ```text
/// hicc::cpp! {
///     #include <hicc/std/memory.hpp>
/// }
///
/// hicc::import_lib! {
///     #![link_name = "example"]
///
///     class Baz;
///     #[cpp(fun = "Baz @make_proxy<Baz>(int)")]
///     #[interface(name = "BazTrait")]
///     fn baz_new(baz: hicc::Interface<Baz>, init: i32) -> Baz;
/// }
/// ```
///
/// **注意**:
/// 1. 这里还需要`#[interface(name = ...)]`定义`Baz`继承的的`trait`类型名，它对应到`c++`侧抽象类的全部虚函数的集合.
/// 1. 首参数必须是`hicc::Interface`类型，后续参数列表对应到`c++`侧类的构造函数的参数列表.
///
///
pub use hicc_macros::import_lib;

///
/// 提供映射`c++`类的功能.
///
///
/// ## `class`关键字
///
/// `c++`类映射为`rust`的`struct`, 在`import_class`宏中需要使用`class`关键字定义对应类型.
///
/// 如果依赖的`c++`类类型在其他`import_class`代码块中定义，应该用`class`声明告诉本`import_class`这是一个`c++`类的映射类型.
///
/// ```text
/// hicc::import_class! {
///     class Bar = other::Bar;
///     #[cpp(class = "Foo")]
///     class Foo {
///         #[cpp(method = "void (const other::Bar&)")]
///         void bar(&self, v: &Bar);
///     }
/// }
/// ```
///
/// ## `#[cpp(class = ..., destroy = ...)]`
///
/// 定义对应的`c++`类型名. 注意对于模板类，需要同时提供完整的模板参数和实例化形式.
///
/// |`c++`模版类|正确定义|错误定义|
/// |---|---|---|
/// |`template<class T> Foo`|`#[cpp(class = "template<class T> Foo<T>")]`|`#[cpp(class = "template<class T> Foo")]`|
///
/// `destroy`仅在`c++`类的析构函数不公开的情况下使用，此时需要独立函数来释放`c++`的对象指针.下面这种场景.
///
/// ```c++
/// class Foo {
///     ~Foo() {}
/// public:
///     static Foo* new_instance();
///     static void free_instance(Foo*);
/// };
/// ```
///
/// 需要如下定义:
/// ```text
/// hicc::import_class! {
///     #[cpp(class = "Foo", destroy = "Foo::free_intance")]
///     class Foo {
///     }
/// }
/// ```
///
/// ## `#[interface]`
///
/// 映射为`rust trait`.
///
/// 这里应该只包含`c++`类中的虚函数. 可以将`rust`实现了此`trait`的对象转换为`c++`抽象类的子类.
///
/// ## `#[cpp(class = ..., ctor = ...)]`
///
/// 当前需要将`rust`特定对象转换为`c++`抽象类的子类时，`c++`侧抽象类的构造函数需要在`ctor`中准确定义.
///
/// 其参数列表最终应该和`imort_lib`中对应`#[interface]`修饰的函数参数列表保持一致.
/// 参见`import_lib`中的说明.
///
/// ## `#[cpp(method = ...)]`
///
/// 映射`c++`类的成员函数. 函数类型的要求同`import_lib`.
///
/// ## `#[cpp(func = ...)]`
///
/// 可将`c++`的全局函数映射为`rust`的方法，前提是`c++`函数的首参数应该是对应的类类型.
/// 函数类型的要求同`import_lib`.
///
/// ## `#[cpp(field = ...)]`
///
/// 映射`c++`类的成员变量，获取成员变量的地址.
///
/// ## `#[cpp(data = ...)]`
///
/// 映射`c++`的全局变量, 获取其变量的地址.
/// 在`rust`中实际建立了类对象和全局变量的生命周期的关联关系，在特定场景下可以增强内存安全.
///
/// ## `hicc::cpp`
///
/// 可以利用`hicc::cpp`嵌入`c++`代码，这些代码并不在全局名字空间，而是在一个特定类的空间内.
/// 因此如果要定义函数，应该有`static`定义为特定类的静态成员函数, 不应该出现`#include ...`这类语句.
///
/// 在这个名字空间中有三个预定义的类型名可以访问:
/// 1. `Self`: 对应到需要映射的`c++`类型.
/// 2. `SelfMethods`: 当前特定类型本身
/// 3. `SelfContainer`: 如果是准备映射`std::vector<T>::iterator`,
///    则`SelfContainer`对应到其容器类`std::vector<T>`. 一般使用者无需关注.
///
/// ## 使用样例
///
/// ```text
/// hicc::cpp! {
///     #include <hicc/std/memory.hpp>
///     #include <iostream>
///
///     struct Foo {
///         virtual ~Foo() {
///             std::cout << "C++ Foo::~Foo" << std::endl;
///         };
///         virtual void foo() const {
///             std::cout << "C++ Foo::foo" << std::endl;
///         }
///     };
/// }
///
/// hicc::import_class! {
///     #[interface]
///     class FooTrait {
///         #[cpp(method = "void foo() const")]
///         fn foo(&self);
///     }
///
///     #[cpp(class = "Foo", ctor = "Foo()")]
///     class Foo: FooTrait {
///         hicc::cpp!{
///             static void bar(const Foo&) {
///                 std::cout << "C++ Foo::bar" << std::endl;
///             }
///         }
///         #[cpp(func = "void SelfMethods::bar(const Foo&)")]
///         fn bar(&self);
///     }
/// }
///
/// hicc::import_lib! {
///     #![link_name = "example"]
///
///     class Foo;
///
///     #[cpp(func = "Foo @make_proxy<Foo>()")]
///     #[interface(name = "FooTrait")]
///     fn foo_new(f: hicc::Interface<Foo>) -> Foo;
/// }
///
/// struct FooImpl;
///
/// impl FooTrait for FooImpl {
///     fn foo(&self) {
///         println!("Rust FooImpl::foo");
///     }
/// }
///
/// fn main() {
///     let foo = foo_new(FooImpl);
///     foo.foo();
///     foo.bar();
/// }
/// ```
pub use hicc_macros::import_class;

///
/// 支持嵌入任意`c++`代码，一般用于包含必要的`c++`头文件.
///
/// 对于特定函数，如果不支持的特殊类型或者因为bug导致自动映射失效，则可以用`hicc::cpp`包装后再映射.
///
/// 需要注意，如果是在`class`中嵌入，它位于特定类的名字空间中.
///
/// ```text
/// hicc::cpp! {
///     #include <hicc/std/memory.hpp>
///
///     static void foo() {
///         //...
///     }
/// }
///
/// hicc::import_lib! {
///     #![link_name = "example"]
///
///     #[cpp(func = "void foo()")]
///     fn foo();
/// }
/// ```
///
pub use hicc_macros::cpp;

mod class;
pub use class::*;

mod generic;
pub use generic::*;

mod exception;
pub use exception::*;

mod function;
pub use function::*;

mod memory;
pub use memory::*;

mod any;
pub use any::*;

/// 辅助`import_lib`宏的实现.
pub trait ImportLib {
    fn import() -> &'static Self
    where
        Self: Sized;
}
