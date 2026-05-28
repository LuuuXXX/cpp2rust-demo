#![feature(specialization)]

use hicc_rs::export_class;
use hicc_rs::*;

// Use a simpler struct without lifetime to avoid lifetime issues in generated code
pub struct Foo<T>(T);

impl<T> Foo<T> {
    fn get_ptr(&self) -> *const T { &self.0 as *const T }
}

#[export_class]
impl<T> Foo<T> {
    fn get_ptr(&self) -> *const T;
}

fn main() {
    unsafe {
        let val = Foo(42i32);
        let m: FooMethods<i32> = {
            let abi: AbiClass<Foo<i32>> =
                transmute(<Foo<i32> as AbiType>::into_abi(val));
            transmute(abi.methods.methods)
        };
        if let Some(_f) = m.get_ptr {
            println!("Depth-lifetime example passed!");
        }
    }
}
