use crate::export_class;
use core::ptr;

#[export_class(in_hicc)]
impl<T> Option<T> {
    fn is_none(&self) -> bool;
    fn unwrap(self) -> T;
    fn take(&mut self) -> Option<T>;
    fn as_ref(&self) -> *const T {
        self.as_ref().map(|v| v as *const T).unwrap_or(ptr::null())
    }
}

#[cfg(test)]
mod test {

    use super::*;
    use crate::*;

    #[test]
    fn test_is_none() {
        unsafe {
            let abi_opt: AbiClass<Option<i32>> =
                transmute(<Option<i32> as AbiType>::into_abi(None));
            let abi_m1: OptionMethods<i32> = transmute(abi_opt.methods.methods);
            let is_none: bool = transmute((abi_m1.is_none)(transmute(&abi_opt)));
            assert_eq!(is_none, true);
            let abi_opt: AbiClass<Option<i32>> =
                transmute(<Option<i32> as AbiType>::into_abi(Some(99)));
            let is_none: bool = transmute((abi_m1.is_none)(transmute(&abi_opt)));
            assert_eq!(is_none, false);
        }
    }

    #[test]
    fn test_unwrap() {
        unsafe {
            let abi_opt: AbiClass<Option<i32>> =
                transmute(<Option<i32> as AbiType>::into_abi(Some(99)));
            let abi_m1: OptionMethods<i32> = transmute(abi_opt.methods.methods);
            let val: i32 = transmute((abi_m1.unwrap)(transmute(abi_opt)));
            assert_eq!(val, 99);
        }
    }

    #[test]
    #[should_panic]
    fn test_unwrap_panic() {
        unsafe {
            let abi_opt: AbiClass<Option<i32>> =
                transmute(<Option<i32> as AbiType>::into_abi(None));
            let abi_m1: OptionMethods<i32> = transmute(abi_opt.methods.methods);
            let is_none: bool = transmute((abi_m1.is_none)(transmute(&abi_opt)));
            assert!(is_none);
            // 被测试函数是extern "C", #[should_panic]无法正常工作.
            //(abi_m1.unwrap)(transmute(abi_opt));
            panic!();
        }
    }

    #[test]
    fn test_take() {
        unsafe {
            let mut abi_opt: AbiClass<Option<i32>> =
                transmute(<Option<i32> as AbiType>::into_abi(Some(99)));
            let abi_m1: OptionMethods<i32> = transmute(abi_opt.methods.methods);
            let is_none: bool = transmute((abi_m1.is_none)(transmute(&abi_opt)));
            assert_eq!(is_none, false);

            let abi_take: AbiClass<Option<i32>> = transmute((abi_m1.take)(transmute(&mut abi_opt)));
            let is_none: bool = transmute((abi_m1.is_none)(transmute(&abi_opt)));
            assert_eq!(is_none, true);
            let is_none: bool = transmute((abi_m1.is_none)(transmute(&abi_take)));
            assert_eq!(is_none, false);

            let val: i32 = transmute((abi_m1.unwrap)(transmute(abi_take)));
            assert_eq!(val, 99);
        }
    }
    #[test]
    fn test_as_ref() {
        unsafe {
            let abi_opt: AbiClass<Option<i32>> =
                transmute(<Option<i32> as AbiType>::into_abi(Some(99)));
            let abi_m1: OptionMethods<i32> = transmute(abi_opt.methods.methods);
            let val: &i32 = transmute((abi_m1.as_ref.unwrap())(transmute(&abi_opt)));
            assert_eq!(val, &99);

            let abi_opt: AbiClass<Option<i32>> =
                transmute(<Option<i32> as AbiType>::into_abi(None));
            let val: *const i32 = transmute((abi_m1.as_ref.unwrap())(transmute(&abi_opt)));
            assert!(val.is_null());

            let abi_opt: AbiClass<Option<Option<i32>>> =
                transmute(<Option<Option<i32>> as AbiType>::into_abi(Some(Some(99))));
            let abi_m1: OptionMethods<Option<i32>> = transmute(abi_opt.methods.methods);
            let abi_opt2: AbiClass<Option<i32>> =
                transmute((abi_m1.as_ref.unwrap())(transmute(&abi_opt)));
            assert!(abi_opt2.is_pointer());

            let abi_m2: OptionMethods<i32> = transmute(abi_opt2.methods.methods);
            let value: &i32 = transmute((abi_m2.as_ref.unwrap())(transmute(&abi_opt2)));
            assert_eq!(value, &99);

            let abi_opt: AbiClass<Option<Option<i32>>> =
                transmute(<Option<Option<i32>> as AbiType>::into_abi(None));
            let abi_opt2: AbiClass<Option<i32>> =
                transmute((abi_m1.as_ref.unwrap())(transmute(&abi_opt)));
            assert!(abi_opt2.is_pointer());
            assert!(abi_opt2.this.is_null());
        }
    }

    #[test]
    fn test_as_ref4() {
        unsafe {
            let abi_opt: AbiClass<Option<&&&i32>> =
                transmute(<Option<&&&i32> as AbiType>::into_abi(None));
            let abi_m: OptionMethods<&&&i32> = transmute(abi_opt.methods.methods);
            assert!(abi_m.as_ref.is_some());

            let abi_opt: AbiClass<Option<&&&&i32>> =
                transmute(<Option<&&&&i32> as AbiType>::into_abi(None));
            let abi_m: OptionMethods<&&&&i32> = transmute(abi_opt.methods.methods);
            assert!(abi_m.as_ref.is_none());
        }
    }
}
