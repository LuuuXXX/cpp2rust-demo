#![feature(specialization)]
use hicc_rs::export_class;
use hicc_rs::*;

pub struct Foo<T, U, V>(T, U, V);

impl<T, U, V> Foo<T, U, V> {
    fn get_first(&self) -> *const T { &self.0 as *const T }
}

#[export_class]
impl<T, U, V> Foo<T, U, V> {
    fn get_first(&self) -> *const T;
}

fn main() {
    unsafe {
        let v: AbiClass<Foo<i32, f64, bool>> = transmute(<Foo<i32, f64, bool> as AbiType>::into_abi(Foo(42, 1.0, true)));
        let m: FooMethods<i32, f64, bool> = transmute(v.methods.methods);
        println!("Multi-param unused example passed!");
    }
}
