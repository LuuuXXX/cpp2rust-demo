//! 013_inheritance_single: 单继承（命名空间类 + make_unique 工厂）。
//!
//! hicc 直出：基类 `Animal` 与派生类 `Dog : public Animal` 各自以
//! `#[cpp(class = "inheritance_single_ns::T")]` 直接绑定真实命名空间类。
//! `name()`/`speak()`/`bark()` 返回 `std::string`，用 `hicc_std::string` 映射；
//! 每个公有构造派生一条 `hicc::make_unique` 工厂，析构交给 hicc `Drop`。
//! Dog 复用基类 `name_` 数据成员（其 `speak()`/`bark()` 输出含构造名），但
//! **不在 Dog 绑定块内重复声明基类的 `name()`**：hicc 对派生类绑定继承而来的
//! 引用返回方法会在运行期产生错误的 this 偏移（实测 SIGSEGV），故继承的访问器
//! 仅在基类 `Animal` 绑定块声明，派生类只绑定自身方法 `bark()`/`speak()`。
//!
//! 工具默认仅自动映射可直出类型，`std::string` 成员需手写补全，
//! 故本示例 `lib.rs` 在默认支架（`lib_scaffold.rs`）之上手写完整绑定。

hicc::cpp! {
    #include "inheritance_single.h"
    #include <hicc/std/string.hpp>
}

hicc::import_class! {
    class string = hicc_std::string;

    #[cpp(class = "inheritance_single_ns::Animal")]
    pub class Animal {
        #[cpp(method = "const std::string& name() const")]
        pub fn name(&self) -> &string;

        #[cpp(method = "std::string speak() const")]
        pub fn speak(&self) -> string;

        pub fn new(name: string) -> Self { animal_new(name) }
    }
}

hicc::import_class! {
    #[cpp(class = "inheritance_single_ns::Dog")]
    pub class Dog {
        #[cpp(method = "std::string bark() const")]
        pub fn bark(&self) -> string;

        #[cpp(method = "std::string speak() const")]
        pub fn speak(&self) -> string;

        pub fn new(name: string) -> Self { dog_new(name) }
    }
}

hicc::import_lib! {
    #![link_name = "inheritance_single"]

    #[cpp(func = "std::unique_ptr<inheritance_single_ns::Animal> hicc::make_unique<inheritance_single_ns::Animal, std::string>(std::string&&)")]
    pub fn animal_new(name: hicc_std::string) -> Animal;

    #[cpp(func = "std::unique_ptr<inheritance_single_ns::Dog> hicc::make_unique<inheritance_single_ns::Dog, std::string>(std::string&&)")]
    pub fn dog_new(name: hicc_std::string) -> Dog;
}
