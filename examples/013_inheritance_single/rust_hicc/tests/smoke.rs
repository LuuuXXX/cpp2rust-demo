//! 013_inheritance_single 冒烟测试
//!
//! 验证单继承下基类方法提升进子类、且子类可调用自有方法。

use inheritance_single::*;

fn decode_cstr(ptr: *const i8) -> String {
    if ptr.is_null() {
        return String::new();
    }
    unsafe { std::ffi::CStr::from_ptr(ptr) }
        .to_string_lossy()
        .into_owned()
}

#[test]
fn smoke_animal_name() {
    let animal = unsafe { animal_new(b"Generic Animal\0".as_ptr() as *const i8) };
    assert_eq!(decode_cstr(animal.get_name()), "Generic Animal", "Animal::getName 应返回构造名");
}

#[test]
fn smoke_dog_inherits_name() {
    let dog = unsafe { dog_new(b"Buddy\0".as_ptr() as *const i8) };
    // getName 为继承自 Animal 的方法，提升进 Dog。
    assert_eq!(decode_cstr(dog.get_name()), "Buddy", "Dog 继承的 getName 应返回构造名");
    // Dog 自有方法可调用且不 panic。
    dog.bark();
    dog.speak();
}
