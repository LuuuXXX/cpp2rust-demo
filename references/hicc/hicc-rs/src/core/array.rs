use crate::export_class;

type Array<T, const N: usize> = [T; N];

#[export_class(in_hicc)]
impl<T: ValueType<Type = IsClass>, const N: usize> Array<T, N> {
    fn len(&self) -> usize {
        N
    }
    fn set(&mut self, idx: usize, val: T) {
        self[idx] = val;
    }
    fn get(&self, idx: usize) -> &T {
        &self[idx]
    }
    fn get_mut(&mut self, idx: usize) -> &mut T {
        &mut self[idx]
    }
}
/*
#[repr(C)]
pub struct HiccArrayMethods<T, const N: usize>
where
    T: ValueType + 'static,
{
    pub len: unsafe extern "C" fn(
        <&HiccArray<T, N> as AbiType>::InputType,
    ) -> <usize as AbiType>::OutputType,
    pub set: unsafe extern "C" fn(
        <&mut HiccArray<T, N> as AbiType>::InputType,
        <usize as AbiType>::InputType,
        <T as AbiType>::InputType,
    ),
    pub get: unsafe extern "C" fn(
        <&'static HiccArray<T, N> as AbiType>::InputType,
        <usize as AbiType>::InputType,
    ) -> <&'static T as AbiType>::OutputType,
    pub get_mut: unsafe extern "C" fn(
        <&'static mut HiccArray<T, N> as AbiType>::InputType,
        <usize as AbiType>::InputType,
    ) -> <&'static mut T as AbiType>::OutputType,
}

unsafe extern "C" fn hicc_array_len<T, const N: usize>(
    this: <&HiccArray<T, N> as AbiType>::InputType,
) -> <usize as AbiType>::OutputType
where
    T: ValueType + 'static,
{
    let this = <&HiccArray<T, N> as AbiType>::from_abi(this);
    let val = this.len();
    <usize as AbiType>::into_abi(val)
}

unsafe extern "C" fn hicc_array_set<T, const N: usize>(
    this: <&mut HiccArray<T, N> as AbiType>::InputType,
    idx: <usize as AbiType>::InputType,
    val: <T as AbiType>::InputType,
) where
    T: ValueType + 'static,
{
    let this = <&mut HiccArray<T, N> as AbiType>::from_abi(this);
    let idx = <usize as AbiType>::from_abi(idx);
    let val = <T as AbiType>::from_abi(val);
    this[idx] = val;
}

unsafe extern "C" fn hicc_array_get<T, const N: usize>(
    this: <&'static HiccArray<T, N> as AbiType>::InputType,
    idx: <usize as AbiType>::InputType,
) -> <&'static T as AbiType>::OutputType
where
    T: ValueType + 'static,
{
    let this = <&HiccArray<T, N> as AbiType>::from_abi(this);
    let idx = <usize as AbiType>::from_abi(idx);
    let val = &this[idx];
    <&T as AbiType>::into_abi(val)
}

unsafe extern "C" fn hicc_array_get_mut<T, const N: usize>(
    this: <&'static mut HiccArray<T, N> as AbiType>::InputType,
    idx: <usize as AbiType>::InputType,
) -> <&'static mut T as AbiType>::OutputType
where
    T: ValueType + 'static,
{
    let this = <&mut HiccArray<T, N> as AbiType>::from_abi(this);
    let idx = <usize as AbiType>::from_abi(idx);
    let val = &mut this[idx];
    <&mut T as AbiType>::into_abi(val)
}

const fn hicc_array_methods<T, const N: usize>() -> HiccArrayMethods<T, N>
where
    T: ValueType + 'static,
{
    HiccArrayMethods {
        len: hicc_array_len::<T, N>,
        set: hicc_array_set::<T, N>,
        get: hicc_array_get::<T, N>,
        get_mut: hicc_array_get_mut::<T, N>,
    }
}

impl<T: ValueType<Type = IsClass>, const N: usize> ValueType for HiccArray<T, N> {
    const N: usize = 0;
    type Type = IsClass;
    type Value = IsValue;
}

impl<T, const N: usize> ClassMethods for HiccArray<T, N>
where
    T: ValueType<Type = IsClass> + 'static,
{
    type Methods = HiccArrayMethods<T, N>;
    const METHODS: &'static AbiMethods<Self> = &AbiClass::new_methods(hicc_array_methods());
    const REF_METHODS: &'static AbiRefMethods<Self> =
        &AbiClass::new_ref_methods(hicc_array_methods());
    const REF_MUT_METHODS: &'static AbiRefMutMethods<Self> =
        &AbiClass::new_ref_mut_methods(hicc_array_methods());
}
*/

#[cfg(test)]
mod test {
    use super::super::option::HiccOptionMethods;
    use super::*;
    use crate::*;

    #[test]
    fn test_array_i32() {
        type Item = [i32; 3];
        unsafe {
            let mut abi_array: [i32; 3] =
                transmute(<Item as AbiType>::into_abi(transmute([1, 2, 3])));
            assert_eq!(abi_array[0], 1);
            assert_eq!(abi_array[1], 2);
            assert_eq!(abi_array[2], 3);
        }
    }

    #[test]
    fn test_array_opt_i32() {
        type Item = [Option<i32>; 3];
        unsafe {
            let mut abi_array: AbiClass<Item> =
                transmute(<Item as AbiType>::into_abi(transmute([
                    None,
                    None,
                    Some(3),
                ])));
            assert!(abi_array.is_value());
            assert!(abi_array.is_mut());
            assert!(!abi_array.this.is_null());

            let len: usize = transmute((abi_array.methods.methods.len)(transmute(&abi_array)));
            assert_eq!(len, 3);

            (abi_array.methods.methods.set)(
                transmute(&mut abi_array),
                transmute(0_usize),
                transmute(<Option<i32> as AbiType>::into_abi(Some(100))),
            );
            let abi_item: AbiClass<Option<i32>> = transmute((abi_array.methods.methods.get)(
                transmute(&abi_array),
                transmute(0_usize),
            ));
            assert!(abi_item.is_pointer());
            assert!(abi_item.is_const());

            let val: &i32 = transmute((abi_item.methods.methods.as_ref)(transmute(&abi_item)));
            assert_eq!(val, &100);
        }
    }
}
