#![feature(specialization)]

use hicc_rs::export_class;
use hicc_rs::*;

pub struct Wrapper<T>(T);

impl<T> Wrapper<T> {
    fn get_val(self) -> T { self.0 }
}

#[export_class]
impl<T> Wrapper<T> {
    fn get_val(self) -> T;
}

fn main() {
    unsafe {
        let m: WrapperMethods<i32> = {
            let abi_val: AbiClass<Wrapper<i32>> =
                transmute(<Wrapper<i32> as AbiType>::into_abi(Wrapper(42)));
            transmute(abi_val.methods.methods)
        };
        let v: i32 = {
            let abi_val: AbiClass<Wrapper<i32>> =
                transmute(<Wrapper<i32> as AbiType>::into_abi(Wrapper(42)));
            transmute((m.get_val)(transmute(abi_val)))
        };
        assert_eq!(v, 42);
        println!("Lifetime param example passed!");
    }
}
