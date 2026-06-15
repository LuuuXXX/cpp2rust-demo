//! 013_inheritance_single 冒烟测试：单继承基类/派生类绑定与行为。

use inheritance_single::*;

fn show(s: &hicc_std::string) -> String {
    let cs = unsafe { std::ffi::CStr::from_ptr(s.c_str()) };
    cs.to_str().unwrap().to_string()
}

#[test]
fn animal_name_and_speak() {
    let a = Animal::new(hicc_std::string::from(c"Generic"));
    assert_eq!(show(&a.name()), "Generic");
    assert_eq!(show(&a.speak()), "Generic makes a sound");
}

#[test]
fn dog_inherits_base_name_field() {
    // Dog 继承基类 name_ 字段：其 speak()/bark() 输出包含构造时传入的名字，
    // 证明派生类复用了基类数据成员。
    let d = Dog::new(hicc_std::string::from(c"Buddy"));
    assert!(show(&d.bark()).starts_with("Buddy"));
}

#[test]
fn dog_overrides_speak_and_has_bark() {
    let d = Dog::new(hicc_std::string::from(c"Rex"));
    assert_eq!(show(&d.speak()), "Rex barks: Woof! Woof!");
    assert_eq!(show(&d.bark()), "Rex barks: Woof! Woof!");
}
