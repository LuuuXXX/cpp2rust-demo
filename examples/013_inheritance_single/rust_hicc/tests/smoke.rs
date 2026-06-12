//! 013_inheritance_single 冒烟测试
//!
//! 验证生成的 Rust FFI 绑定可编译、可链接 C++ 实现，且基本行为正确。

use inheritance_single::*;

#[test]
fn smoke_animal_create_and_get_name() {
    let name = "Generic Animal\0";
    let animal = unsafe { animal_new(name.as_ptr() as *const i8) };
    let got = decode_cstr(animal.get_name());
    assert_eq!(got, "Generic Animal", "Animal 名称应为 'Generic Animal'");
}

#[test]
fn smoke_dog_create_and_get_name() {
    let name = "Buddy\0";
    let dog = unsafe { dog_new(name.as_ptr() as *const i8) };
    let got = decode_cstr(dog.get_name());
    assert_eq!(got, "Buddy", "Dog 名称应为 'Buddy'");
}

#[test]
fn smoke_dog_inherits_animal_behavior() {
    let animal_name = "Cat\0";
    let animal = unsafe { animal_new(animal_name.as_ptr() as *const i8) };
    let dog_name = "Rex\0";
    let dog = unsafe { dog_new(dog_name.as_ptr() as *const i8) };

    // Both should have the getName method (inherited for Dog)
    let animal_got = decode_cstr(animal.get_name());
    let dog_got = decode_cstr(dog.get_name());
    assert_eq!(animal_got, "Cat", "Animal 名称应为 'Cat'");
    assert_eq!(dog_got, "Rex", "Dog 名称应为 'Rex'");

    // Dog also has its own bark method
    dog.bark(); // should not panic
}
