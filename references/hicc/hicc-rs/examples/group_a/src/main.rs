#![feature(specialization)]
use hicc_rs::export_class;
use hicc_rs::*;

pub struct Container<T>(T);

impl<T> Container<T> {
    fn take(self) -> T { self.0 }
    fn count(&self) -> i32 { 1 }
}

#[export_class]
impl<T> Container<T> {
    fn take(self) -> T;
    fn count(&self) -> i32;
}

fn main() {
    unsafe {
        let v: AbiClass<Container<i32>> = transmute(<Container<i32> as AbiType>::into_abi(Container(42)));
        let m: ContainerMethods<i32> = transmute(v.methods.methods);
        let cnt: i32 = transmute((m.count)(transmute(&v)));
        assert_eq!(cnt, 1);
        let val: i32 = transmute((m.take)(transmute(v)));
        assert_eq!(val, 42);
        println!("Group A (depth 0, no ref/ptr) example passed!");
    }
}
