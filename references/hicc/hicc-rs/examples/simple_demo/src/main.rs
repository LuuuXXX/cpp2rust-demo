#![feature(specialization)]
use hicc_rs::export_class;
use hicc_rs::*;

pub struct MyValue(i32);

impl MyValue {
    fn get(&self) -> i32 { self.0 }
}

#[export_class]
impl MyValue {
    fn get(&self) -> i32;
}

fn main() {
    unsafe {
        let v: AbiClass<MyValue> = transmute(<MyValue as AbiType>::into_abi(MyValue(42)));
        let m: MyValueMethods = transmute(v.methods.methods);
        let val: i32 = transmute((m.get)(transmute(&v)));
        assert_eq!(val, 42);
        println!("Simple example passed!");
    }
}
