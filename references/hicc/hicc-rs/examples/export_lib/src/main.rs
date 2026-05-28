#![feature(specialization)]

use hicc_rs::export_lib;
use hicc_rs::{AbiClass, AbiType, transmute};

// Function implementations at crate root
fn opt_is_none(val: &Option<i32>) -> bool { val.is_none() }
fn opt_unwrap(val: Option<i32>) -> i32 { val.unwrap() }
fn double_it(x: i32) -> i32 { x * 2 }

#[export_lib(export_name = "get_examples")]
mod exports {
    // Declaration only - body from crate root function
    fn opt_is_none(val: &Option<i32>) -> bool;
    // Declaration with by-value param
    fn opt_unwrap(val: Option<i32>) -> i32;
    // Declaration with simple type
    fn double_it(x: i32) -> i32;
}

fn main() {
    unsafe {
        let lib = exports::get_examples();
        
        // Test 1: &Option param, bool return
        let obj: AbiClass<Option<i32>> =
            transmute(<Option<i32> as AbiType>::into_abi(Some(42)));
        let is_none: bool = transmute((lib.opt_is_none)(transmute(&obj)));
        assert!(!is_none);
        
        // Test 2: by-value Option, i32 return
        let obj2: AbiClass<Option<i32>> =
            transmute(<Option<i32> as AbiType>::into_abi(Some(99)));
        let val: i32 = transmute((lib.opt_unwrap)(transmute(obj2)));
        assert_eq!(val, 99);
        
        // Test 3: simple i32 param and return
        let v: i32 = transmute((lib.double_it)(transmute(21i32)));
        assert_eq!(v, 42);
        
        println!("Export lib comprehensive example passed!");
    }
}
