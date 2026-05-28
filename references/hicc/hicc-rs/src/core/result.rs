use crate::export_class;

#[export_class(in_hicc)]
impl<T, E> Result<T, E> {
    fn is_ok(&self) -> bool;
    fn is_err(&self) -> bool;
    fn ok(self) -> T {
        self.ok().unwrap()
    }
    fn err(self) -> E {
        self.err().unwrap()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::*;

    #[test]
    fn test_i32_bool() {
        unsafe {
            let abi_rlt: AbiClass<Result<i32, bool>> =
                transmute(<Result<i32, bool> as AbiType>::into_abi(Err(false)));
            let is_err: bool = transmute((abi_rlt.methods.methods.is_err)(transmute(&abi_rlt)));
            assert_eq!(is_err, true);
            let is_ok: bool = transmute((abi_rlt.methods.methods.is_ok)(transmute(&abi_rlt)));
            assert_eq!(is_ok, false);

            let err: bool = transmute((abi_rlt.methods.methods.err)(transmute(abi_rlt)));
            assert_eq!(err, false);

            let abi_rlt: AbiClass<Result<i32, bool>> =
                transmute(<Result<i32, bool> as AbiType>::into_abi(Ok(88)));

            let is_err: bool = transmute((abi_rlt.methods.methods.is_err)(transmute(&abi_rlt)));
            assert_eq!(is_err, false);
            let is_ok: bool = transmute((abi_rlt.methods.methods.is_ok)(transmute(&abi_rlt)));
            assert_eq!(is_ok, true);

            let ok: i32 = transmute((abi_rlt.methods.methods.ok)(transmute(abi_rlt)));
            assert_eq!(ok, 88);
        }
    }

    #[test]
    fn test_opt_i32_bool() {
        unsafe {
            let abi_rlt: AbiClass<Result<Option<i32>, Option<bool>>> =
                transmute(<Result<Option<i32>, Option<bool>> as AbiType>::into_abi(
                    Err(Some(false)),
                ));
            let is_err: bool = transmute((abi_rlt.methods.methods.is_err)(transmute(&abi_rlt)));
            assert_eq!(is_err, true);
            let is_ok: bool = transmute((abi_rlt.methods.methods.is_ok)(transmute(&abi_rlt)));
            assert_eq!(is_ok, false);

            let abi_err: AbiClass<Option<bool>> =
                transmute((abi_rlt.methods.methods.err)(transmute(abi_rlt)));
            assert!(abi_err.is_value());
            let val: bool = transmute((abi_err.methods.methods.unwrap)(transmute(abi_err)));
            assert_eq!(val, false);

            let abi_rlt: AbiClass<Result<Option<i32>, Option<bool>>> =
                transmute(<Result<Option<i32>, Option<bool>> as AbiType>::into_abi(
                    Ok(Some(88)),
                ));

            let is_err: bool = transmute((abi_rlt.methods.methods.is_err)(transmute(&abi_rlt)));
            assert_eq!(is_err, false);
            let is_ok: bool = transmute((abi_rlt.methods.methods.is_ok)(transmute(&abi_rlt)));
            assert_eq!(is_ok, true);

            let abi_ok: AbiClass<Option<i32>> =
                transmute((abi_rlt.methods.methods.ok)(transmute(abi_rlt)));
            assert!(abi_ok.is_value());

            let val: i32 = transmute((abi_ok.methods.methods.unwrap)(transmute(abi_ok)));
            assert_eq!(val, 88);
        }
    }
}
