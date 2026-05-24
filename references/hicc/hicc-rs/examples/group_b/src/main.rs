#![feature(specialization)]
use hicc_rs::export_class;
use hicc_rs::*;

pub struct Container<T>(T);

impl<T> Container<T> {
    fn get_ptr(&self) -> *const T { ::core::ptr::null() }
}

#[export_class]
impl<T> Container<T> {
    fn get_ptr(&self) -> *const T;
}

fn main() {
    unsafe {
        let v: AbiClass<Container<i32>> = transmute(<Container<i32> as AbiType>::into_abi(Container(42)));
        let m: ContainerMethods<i32> = transmute(v.methods.methods);
        if let Some(f) = m.get_ptr {
            let _: *const i32 = transmute(f(transmute(&v)));
        }
        println!("Group B (depth 1, single ref/ptr) example passed!");
    }
}
