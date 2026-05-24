use crate::export_class;

type Slice<'a, T> = &'a [T];
type SliceMut<'a, T> = &'a mut [T];

#[export_class(in_hicc)]
impl<'a, T> Slice<'a, T> {
    fn len(&self) -> usize {
        self.len()
    }
    fn get(&self, idx: usize) -> &T {
        &self[idx]
    }

    fn get_mut(&mut self, idx: usize) -> &mut T {
        panic!();
    }

    fn set(&mut self, idx: usize, val: T) {
        panic!();
    }
}

#[export_class(in_hicc)]
impl<'a, T> SliceMut<'a, T> {
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

#[cfg(test)]
mod test {
    use super::super::option::OptionMethods;
    use super::*;
    use crate::*;

    #[test]
    fn test_const_slice_i32() {
        unsafe {
            let abi_slice: AbiClass<&[i32]> = transmute(<&[i32] as AbiType>::into_abi(transmute(
                [1, 2, 3].as_slice(),
            )));
            let abi_m: SliceMethods<'_, i32> = transmute(abi_slice.methods.methods);
            assert!(abi_m.set.is_none());
            assert!(abi_m.get.is_some());
            assert!(abi_m.get_mut.is_none());
            let len: usize = transmute((abi_m.len)(transmute(&abi_slice)));
            assert_eq!(len, 3);

            let val: &i32 = transmute((abi_m.get.unwrap())(
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
            let abi_m: SliceMethods<'_, i32> = transmute(abi_slice.methods.methods);
            assert!(abi_m.set.is_some());
            assert!(abi_m.get.is_some());
            assert!(abi_m.get_mut.is_some());
            let len: usize = transmute((abi_m.len)(transmute(&abi_slice)));
            assert_eq!(len, 3);

            (abi_m.set.unwrap())(
                transmute(&mut abi_slice),
                transmute(0_usize),
                transmute(100),
            );
            let val: &i32 = transmute((abi_m.get.unwrap())(
                transmute(&abi_slice),
                transmute(0_usize),
            ));
            assert_eq!(val, &100);

            let val: &mut i32 = transmute((abi_m.get_mut.unwrap())(
                transmute(&mut abi_slice),
                transmute(1_usize),
            ));
            assert_eq!(val, &2);
            *val = 0;
            let val: &i32 = transmute((abi_m.get.unwrap())(
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
            let abi_m: SliceMethods<'_, Item> = transmute(abi_slice.methods.methods);
            assert!(abi_m.set.is_none());
            assert!(abi_m.get.is_some());
            let len: usize = transmute((abi_m.len)(transmute(&abi_slice)));
            assert_eq!(len, 4);

            let abi_item: AbiClass<Item> = transmute((abi_m.get.unwrap())(
                transmute(&abi_slice),
                transmute(0_usize),
            ));
            let opt_m: OptionMethods<i32> = transmute(abi_item.methods.methods);
            assert!(abi_item.is_pointer());
            assert!(abi_item.is_const());

            let val: &i32 = transmute((opt_m.as_ref.unwrap())(transmute(&abi_item)));
            assert_eq!(val, &1);

            let abi_item: AbiClass<Item> = transmute((abi_m.get.unwrap())(
                transmute(&abi_slice),
                transmute(1_usize),
            ));
            let val: *const i32 = transmute((opt_m.as_ref.unwrap())(transmute(&abi_item)));
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
            let abi_m: SliceMethods<'_, Item> = transmute(abi_slice.methods.methods);
            assert!(abi_m.set.is_some());
            assert!(abi_m.get.is_some());
            assert!(abi_m.get_mut.is_some());
            let len: usize = transmute((abi_m.len)(transmute(&abi_slice)));
            assert_eq!(len, 4);

            (abi_m.set.unwrap())(
                transmute(&mut abi_slice),
                transmute(0_usize),
                transmute(<Option<i32> as AbiType>::into_abi(Some(100))),
            );

            let abi_item: AbiClass<Item> = transmute((abi_m.get.unwrap())(
                transmute(&abi_slice),
                transmute(0_usize),
            ));
            let opt_m: OptionMethods<i32> = transmute(abi_item.methods.methods);
            assert!(abi_item.is_pointer());
            assert!(abi_item.is_const());

            let val: &i32 = transmute((opt_m.as_ref.unwrap())(transmute(&abi_item)));
            assert_eq!(val, &100);

            let abi_item: AbiClass<Item> = transmute((abi_m.get.unwrap())(
                transmute(&abi_slice),
                transmute(1_usize),
            ));
            let val: *const i32 = transmute((opt_m.as_ref.unwrap())(transmute(&abi_item)));
            assert!(val.is_null());
        }
    }
}
