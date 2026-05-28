use crate::export_class;

type Slice<T> = &'static [T];
type SliceMut<T> = &'static mut [T];

#[export_class(in_hicc)]
impl<T> Slice<T> {
    fn len(&self) -> usize {
        self.len()
    }
    fn get(&self, idx: usize) -> &T {
        &self[idx]
    }
}

#[export_class(in_hicc)]
impl<T> SliceMut<T> {
    fn len(&self) -> usize {
        self.len()
    }
    fn get(&self, idx: usize) -> &T {
        &self[idx]
    }

    fn get_mut(&mut self, idx: usize) -> &mut T {
        &mut self[idx]
    }

    fn set(&mut self, idx: usize, val: T) {
        self[idx] = val;
    }
}

/*
#[repr(C)]
pub struct HiccSliceMethods<T: ValueType + 'static> {
    pub len: unsafe extern "C" fn(
        <&HiccSlice<T> as AbiType>::InputType,
    ) -> <usize as AbiType>::OutputType,
    pub get: unsafe extern "C" fn(
        <&'static HiccSlice<T> as AbiType>::InputType,
        <usize as AbiType>::InputType,
    ) -> <&'static T as AbiType>::OutputType,
    pub get_mut: Option<
        unsafe extern "C" fn(
            <&'static mut HiccSliceMut<T> as AbiType>::InputType,
            <usize as AbiType>::InputType,
        ) -> <&'static mut T as AbiType>::OutputType,
    >,
    pub set: Option<
        unsafe extern "C" fn(
            <&mut HiccSliceMut<T> as AbiType>::InputType,
            <usize as AbiType>::InputType,
            <T as AbiType>::InputType,
        ),
    >,
}

unsafe extern "C" fn hicc_slice_len<T: ValueType + 'static>(
    this: <&HiccSlice<T> as AbiType>::InputType,
) -> <usize as AbiType>::OutputType {
    let this = <&HiccSlice<T> as AbiType>::from_abi(this);
    let val = this.len();
    <usize as AbiType>::into_abi(val)
}

unsafe extern "C" fn hicc_slice_get<T: ValueType + 'static>(
    this: <&'static HiccSlice<T> as AbiType>::InputType,
    idx: <usize as AbiType>::InputType,
) -> <&'static T as AbiType>::OutputType {
    let this = <&HiccSlice<T> as AbiType>::from_abi(this);
    let idx = <usize as AbiType>::from_abi(idx);
    let val = &this[idx];
    <&T as AbiType>::into_abi(val)
}

unsafe extern "C" fn hicc_slice_get_mut<T: ValueType + 'static>(
    this: <&'static mut HiccSliceMut<T> as AbiType>::InputType,
    idx: <usize as AbiType>::InputType,
) -> <&'static mut T as AbiType>::OutputType {
    let this = <&mut HiccSliceMut<T> as AbiType>::from_abi(this);
    let idx = <usize as AbiType>::from_abi(idx);
    let val = &mut this[idx];
    <&mut T as AbiType>::into_abi(val)
}

unsafe extern "C" fn hicc_slice_set<T: ValueType + 'static>(
    this: <&mut HiccSliceMut<T> as AbiType>::InputType,
    idx: <usize as AbiType>::InputType,
    val: <T as AbiType>::InputType,
) {
    let this = <&mut HiccSliceMut<T> as AbiType>::from_abi(this);
    let idx = <usize as AbiType>::from_abi(idx);
    let val = <T as AbiType>::from_abi(val);
    this[idx] = val;
}

const fn hicc_slice_methods<T: ValueType + 'static>() -> HiccSliceMethods<T> {
    HiccSliceMethods::<T> {
        len: hicc_slice_len::<T>,
        get: hicc_slice_get::<T>,
        get_mut: None,
        set: None,
    }
}

const fn hicc_slice_mut_methods<T: ValueType + 'static>() -> HiccSliceMethods<T> {
    HiccSliceMethods::<T> {
        len: hicc_slice_len::<T>,
        get: hicc_slice_get::<T>,
        get_mut: Some(hicc_slice_get_mut::<T>),
        set: Some(hicc_slice_set::<T>),
    }
}

impl<T> ValueType for HiccSlice<T> {
    const N: usize = 0;
    type Type = IsClass;
    type Value = IsValue;
}

impl<T: ValueType + 'static> ClassMethods for HiccSlice<T> {
    type Methods = HiccSliceMethods<T>;
    const METHODS: &'static AbiMethods<Self> = &AbiClass::new_methods(hicc_slice_methods());
    const REF_METHODS: &'static AbiRefMethods<Self> =
        &AbiClass::new_ref_methods(hicc_slice_methods());
    const REF_MUT_METHODS: &'static AbiRefMutMethods<Self> =
        &AbiClass::new_ref_mut_methods(hicc_slice_methods());
}

impl<T: ValueType + 'static> ClassMethods for HiccSliceMut<T> {
    type Methods = HiccSliceMethods<T>;
    const METHODS: &'static AbiMethods<Self> = &AbiClass::new_methods(hicc_slice_mut_methods());
    const REF_METHODS: &'static AbiRefMethods<Self> =
        &AbiClass::new_ref_methods(hicc_slice_mut_methods());
    const REF_MUT_METHODS: &'static AbiRefMutMethods<Self> =
        &AbiClass::new_ref_mut_methods(hicc_slice_mut_methods());
}

impl<T> ValueType for HiccSliceMut<T> {
    const N: usize = 0;
    type Type = IsClass;
    type Value = IsValue;
}
*/

#[cfg(test)]
mod test {
    use super::*;
    use crate::*;

    #[test]
    fn test_const_slice_i32() {
        unsafe {
            let abi_slice: AbiClass<&[i32]> = transmute(<&[i32] as AbiType>::into_abi(transmute(
                [1, 2, 3].as_slice(),
            )));
            let len: usize = transmute((abi_slice.methods.methods.len)(transmute(&abi_slice)));
            assert_eq!(len, 3);

            let val: &i32 = transmute((abi_slice.methods.methods.get)(
                transmute(&abi_slice),
                transmute(0_usize),
            ));
            assert_eq!(val, &1);
        }
    }

    #[test]
    fn test_mut_slice_i32() {
        unsafe {
            let mut abi_slice: AbiClass<&mut [i32]> = transmute(<&mut [i32] as AbiType>::into_abi(
                transmute([1, 2, 3].as_mut_slice()),
            ));
            let len: usize = transmute((abi_slice.methods.methods.len)(transmute(&abi_slice)));
            assert_eq!(len, 3);

            (abi_slice.methods.methods.set)(
                transmute(&mut abi_slice),
                transmute(0_usize),
                transmute(100),
            );
            let val: &i32 = transmute((abi_slice.methods.methods.get)(
                transmute(&abi_slice),
                transmute(0_usize),
            ));
            assert_eq!(val, &100);

            let val: &mut i32 = transmute((abi_slice.methods.methods.get_mut)(
                transmute(&mut abi_slice),
                transmute(1_usize),
            ));
            assert_eq!(val, &2);
            *val = 0;
            let val: &i32 = transmute((abi_slice.methods.methods.get)(
                transmute(&abi_slice),
                transmute(1_usize),
            ));
            assert_eq!(val, &0);
        }
    }

    #[test]
    fn test_const_slice_opt_i32() {
        type Item = Option<i32>;
        unsafe {
            let abi_slice: AbiClass<&[Item]> = transmute(<&[Item] as AbiType>::into_abi(
                transmute([Some(1), None, None, None].as_slice()),
            ));
            let len: usize = transmute((abi_slice.methods.methods.len)(transmute(&abi_slice)));
            assert_eq!(len, 4);

            let abi_item: AbiClass<Item> = transmute((abi_slice.methods.methods.get)(
                transmute(&abi_slice),
                transmute(0_usize),
            ));

            let val: &i32 = transmute((abi_item.methods.methods.as_ref)(transmute(&abi_item)));
            assert_eq!(val, &1);

            let abi_item: AbiClass<Item> = transmute((abi_slice.methods.methods.get)(
                transmute(&abi_slice),
                transmute(1_usize),
            ));
            let val: *const i32 =
                transmute((abi_item.methods.methods.as_ref)(transmute(&abi_item)));
            assert!(val.is_null());
        }
    }

    #[test]
    fn test_mut_slice_opt_i32() {
        type Item = Option<i32>;
        unsafe {
            let mut abi_slice: AbiClass<&mut [Item]> =
                transmute(<&mut [Item] as AbiType>::into_abi(transmute(
                    [None, None, None, Some(1)].as_mut_slice(),
                )));
            let len: usize = transmute((abi_slice.methods.methods.len)(transmute(&abi_slice)));
            assert_eq!(len, 4);

            (abi_slice.methods.methods.set)(
                transmute(&mut abi_slice),
                transmute(0_usize),
                transmute(<Option<i32> as AbiType>::into_abi(Some(100))),
            );

            let abi_item: AbiClass<Item> = transmute((abi_slice.methods.methods.get)(
                transmute(&abi_slice),
                transmute(0_usize),
            ));

            let val: &i32 = transmute((abi_item.methods.methods.as_ref)(transmute(&abi_item)));
            assert_eq!(val, &100);

            let abi_item: AbiClass<Item> = transmute((abi_slice.methods.methods.get)(
                transmute(&abi_slice),
                transmute(1_usize),
            ));
            let val: *const i32 =
                transmute((abi_item.methods.methods.as_ref)(transmute(&abi_item)));
            assert!(val.is_null());
        }
    }
}
