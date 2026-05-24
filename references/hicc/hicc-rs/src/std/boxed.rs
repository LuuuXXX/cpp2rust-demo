use crate::export_class;

#[export_class(in_hicc)]
impl<T> Box<T> {
    fn get(&self) -> &T { &**self }
    fn get_mut(&mut self) -> &mut T { &mut **self }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::*;

    #[test]
    fn test_i32() {
        unsafe {
            let mut abi_box: AbiClass<Box<i32>> =
                transmute(<Box<i32> as AbiType>::into_abi(Box::new(100)));
            let box_m: BoxMethods<i32> = transmute(abi_box.methods.methods);

            if let Some(f) = box_m.get {
                let pval: *const i32 = transmute(f(transmute(&abi_box)));
                assert!(!pval.is_null());
            }
            if let Some(f) = box_m.get_mut {
                let pval: *mut i32 = transmute(f(transmute(&mut abi_box)));
                assert!(!pval.is_null());
            }
        }
    }
}
