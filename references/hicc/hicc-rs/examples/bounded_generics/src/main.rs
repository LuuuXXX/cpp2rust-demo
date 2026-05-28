#![feature(specialization)]

use hicc_rs::export_class;
use hicc_rs::*;

pub struct Foo<T, U, V>(T, U, V);

impl<T: ::std::fmt::Debug, U: 'static, V: ::std::hash::Hash + 'static> Foo<T, U, V> {
    fn get_t(&self) -> i32 { 42 }
}

#[export_class]
impl<T, U, V> Foo<T, U, V>
where
    T: ::std::fmt::Debug,
    U: 'static,
    V: ::std::hash::Hash + 'static,
{
    fn get_t(&self) -> i32;
}

fn main() {
    unsafe {
        let m: FooMethods<i32, bool, u64> = {
            let val = Foo(42i32, true, 123u64);
            let abi: AbiClass<Foo<i32, bool, u64>> =
                transmute(<Foo<i32, bool, u64> as AbiType>::into_abi(val));
            transmute(abi.methods.methods)
        };
        println!("Bounded generics example passed!");
    }
}
