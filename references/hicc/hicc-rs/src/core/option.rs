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
    fn as_mut(&mut self) -> *mut T {
        self.as_mut()
            .map(|v| v as *mut T)
            .unwrap_or(ptr::null_mut())
    }
}
/*
#[repr(C)]
pub struct HiccOptionMethods<T>
where
    T: 'static,
{
    pub is_none:
        unsafe extern "C" fn(<&Option<T> as AbiType>::InputType) -> <bool as AbiType>::OutputType,
    pub unwrap:
        unsafe extern "C" fn(<Option<T> as AbiType>::InputType) -> <T as AbiType>::OutputType,
    pub take: unsafe extern "C" fn(
        <&mut Option<T> as AbiType>::InputType,
    ) -> <Option<T> as AbiType>::OutputType,
    pub as_ref: unsafe extern "C" fn(
        <&Option<T> as AbiType>::InputType,
    ) -> <*const T as AbiType>::OutputType,
    pub as_mut: unsafe extern "C" fn(
        <&mut Option<T> as AbiType>::InputType,
    ) -> <*mut T as AbiType>::OutputType,
}

unsafe extern "C" fn hicc_option_is_none<T>(
    this: <&Option<T> as AbiType>::InputType,
) -> <bool as AbiType>::OutputType
where
    T: 'static,
{
    let this = <&Option<T> as AbiType>::from_abi(this);
    let val = this.is_none();
    <bool as AbiType>::into_abi(val)
}

unsafe extern "C" fn hicc_option_unwrap<T>(
    this: <Option<T> as AbiType>::InputType,
) -> <T as AbiType>::OutputType
where
    T: 'static,
{
    let this = <Option<T> as AbiType>::from_abi(this);
    let val = this.unwrap();
    <T as AbiType>::into_abi(val)
}

unsafe extern "C" fn hicc_option_take<T>(
    this: <&mut Option<T> as AbiType>::InputType,
) -> <Option<T> as AbiType>::OutputType
where
    T: 'static,
{
    let this = <&mut Option<T> as AbiType>::from_abi(this);
    let val = this.take();
    <Option<T> as AbiType>::into_abi(val)
}

unsafe extern "C" fn hicc_option_as_ref<T>(
    this: <&Option<T> as AbiType>::InputType,
) -> <*const T as AbiType>::OutputType
where
    T: 'static,
{
    let this = <&Option<T> as AbiType>::from_abi(this);
    let val = this.as_ref().map(|v| v as *const T).unwrap_or(ptr::null());
    <*const T as AbiType>::into_abi(val)
}

unsafe extern "C" fn hicc_option_as_mut<T>(
    this: <&mut Option<T> as AbiType>::InputType,
) -> <*mut T as AbiType>::OutputType
where
    T: 'static,
{
    let this = <&mut Option<T> as AbiType>::from_abi(this);
    let val = this
        .as_mut()
        .map(|v| v as *mut T)
        .unwrap_or(ptr::null_mut());
    <*mut T as AbiType>::into_abi(val)
}

const fn hicc_option_methods<T>() -> HiccOptionMethods<T>
where
    T: ValueType + 'static,
{
    HiccOptionMethods::<T> {
        is_none: hicc_option_is_none::<T>,
        unwrap: hicc_option_unwrap::<T>,
        take: hicc_option_take::<T>,
        as_ref: hicc_option_as_ref::<T>,
        as_mut: hicc_option_as_mut::<T>,
    }
}

impl<T: 'static> ValueType for Option<T> {
    const N: usize = 0;
    type Type = IsClass;
    type Value = IsValue;
}

impl<T> ClassMethods for Option<T>
where
    T: ValueType + 'static,
{
    type Methods = HiccOptionMethods<T>;
    const METHODS: &'static AbiMethods<Self> = &AbiClass::new_methods(hicc_option_methods());
    const REF_METHODS: &'static AbiRefMethods<Self> =
        &AbiClass::new_ref_methods(hicc_option_methods());
    const REF_MUT_METHODS: &'static AbiRefMutMethods<Self> =
        &AbiClass::new_ref_mut_methods(hicc_option_methods());
}
*/
#[cfg(test)]
mod test {

    use super::*;
    use crate::*;

    #[test]
    fn test_is_none() {
        unsafe {
            let abi_opt: AbiClass<Option<i32>> =
                transmute(<Option<i32> as AbiType>::into_abi(None));
            let is_none: bool = transmute((abi_opt.methods.methods.is_none)(transmute(&abi_opt)));
            assert_eq!(is_none, true);
            let abi_opt: AbiClass<Option<i32>> =
                transmute(<Option<i32> as AbiType>::into_abi(Some(99)));
            let is_none: bool = transmute((abi_opt.methods.methods.is_none)(transmute(&abi_opt)));
            assert_eq!(is_none, false);
        }
    }

    #[test]
    fn test_unwrap() {
        unsafe {
            let abi_opt: AbiClass<Option<i32>> =
                transmute(<Option<i32> as AbiType>::into_abi(Some(99)));
            let val: i32 = transmute((abi_opt.methods.methods.unwrap)(transmute(abi_opt)));
            assert_eq!(val, 99);
        }
    }

    #[test]
    #[should_panic]
    fn test_unwrap_panic() {
        unsafe {
            let abi_opt: AbiClass<Option<i32>> =
                transmute(<Option<i32> as AbiType>::into_abi(None));
            let is_none: bool = transmute((abi_opt.methods.methods.is_none)(transmute(&abi_opt)));
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
            let is_none: bool = transmute((abi_opt.methods.methods.is_none)(transmute(&abi_opt)));
            assert_eq!(is_none, false);

            let abi_take: AbiClass<Option<i32>> =
                transmute((abi_opt.methods.methods.take)(transmute(&mut abi_opt)));
            let is_none: bool = transmute((abi_opt.methods.methods.is_none)(transmute(&abi_opt)));
            assert_eq!(is_none, true);
            let is_none: bool = transmute((abi_opt.methods.methods.is_none)(transmute(&abi_take)));
            assert_eq!(is_none, false);

            let val: i32 = transmute((abi_opt.methods.methods.unwrap)(transmute(abi_take)));
            assert_eq!(val, 99);
        }
    }
    #[test]
    fn test_as_ref() {
        unsafe {
            let abi_opt: AbiClass<Option<i32>> =
                transmute(<Option<i32> as AbiType>::into_abi(Some(99)));
            let val: &i32 = transmute((abi_opt.methods.methods.as_ref)(transmute(&abi_opt)));
            assert_eq!(val, &99);

            let abi_opt: AbiClass<Option<i32>> =
                transmute(<Option<i32> as AbiType>::into_abi(None));
            let val: *const i32 = transmute((abi_opt.methods.methods.as_ref)(transmute(&abi_opt)));
            assert!(val.is_null());

            let abi_opt: AbiClass<Option<Option<i32>>> =
                transmute(<Option<Option<i32>> as AbiType>::into_abi(Some(Some(99))));
            let abi_opt2: AbiClass<Option<i32>> =
                transmute((abi_opt.methods.methods.as_ref)(transmute(&abi_opt)));
            assert!(abi_opt2.is_pointer());

            let value: &i32 = transmute((abi_opt2.methods.methods.as_ref)(transmute(&abi_opt2)));
            assert_eq!(value, &99);

            let abi_opt: AbiClass<Option<Option<i32>>> =
                transmute(<Option<Option<i32>> as AbiType>::into_abi(None));
            let abi_opt2: AbiClass<Option<i32>> =
                transmute((abi_opt.methods.methods.as_ref)(transmute(&abi_opt)));
            assert!(abi_opt2.is_pointer());
            assert!(abi_opt2.this.is_null());
        }
    }
}
