#![feature(specialization)]

use hicc_rs::export_class;
use hicc_rs::*;

pub struct Bounded<T>(T);

impl<T> Bounded<T> {
    fn get_val(self) -> T { self.0 }
}

#[export_class]
impl<T> Bounded<T> {
    fn get_val(self) -> T;
}

fn main() {
    unsafe {
        let m: BoundedMethods<i32> = {
            let abi_val: AbiClass<Bounded<i32>> =
                transmute(<Bounded<i32> as AbiType>::into_abi(Bounded(42)));
            transmute(abi_val.methods.methods)
        };
        let v: i32 = {
            let abi_val: AbiClass<Bounded<i32>> =
                transmute(<Bounded<i32> as AbiType>::into_abi(Bounded(42)));
            transmute((m.get_val)(transmute(abi_val)))
        };
        assert_eq!(v, 42);
        println!("Bounded params example passed!");
    }
}
