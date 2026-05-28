#![feature(specialization)]
use hicc_rs::export_class;
use hicc_rs::*;

pub struct MyContainer<T>(T);

impl<T> MyContainer<T> {
    fn is_none(&self) -> bool { false }
    fn unwrap(self) -> T { self.0 }
    fn as_ref(&self) -> *const T { ::core::ptr::null() }
}

#[export_class]
impl<T> MyContainer<T> {
    fn is_none(&self) -> bool;
    fn unwrap(self) -> T;
    fn as_ref(&self) -> *const T;
}

fn main() {
    unsafe {
        let v: AbiClass<MyContainer<i32>> = transmute(<MyContainer<i32> as AbiType>::into_abi(MyContainer(42)));
        let m: MyContainerMethods<i32> = transmute(v.methods.methods);
        let is_none: bool = transmute((m.is_none)(transmute(&v)));
        assert!(!is_none);
        let val: i32 = transmute((m.unwrap)(transmute(v)));
        assert_eq!(val, 42);
        println!("Option-style example passed!");
    }
}
