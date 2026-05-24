use crate::{export_class, IsClass, ValueType};

type Array<T, const N: usize> = [T; N];

#[export_class(in_hicc)]
impl<T: ValueType<Flag1 = IsClass>, const N: usize> Array<T, N> {
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

#[cfg(test)]
mod test {
    use super::super::option::OptionMethods;
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

            let abi_m: ArrayMethods<Option<i32>, 3> = transmute(abi_array.methods.methods);
            assert!(abi_m.get.is_some());
            assert!(abi_m.get_mut.is_some());

            let len: usize = transmute((abi_m.len)(transmute(&abi_array)));
            assert_eq!(len, 3);

            (abi_m.set)(
                transmute(&mut abi_array),
                transmute(0_usize),
                transmute(<Option<i32> as AbiType>::into_abi(Some(100))),
            );
            let abi_item: AbiClass<Option<i32>> = transmute((abi_m.get.unwrap())(
                transmute(&abi_array),
                transmute(0_usize),
            ));
            assert!(abi_item.is_pointer());
            assert!(abi_item.is_const());
            let opt_m: OptionMethods<i32> = transmute(abi_item.methods.methods);

            let val: &i32 = transmute((opt_m.as_ref.unwrap())(transmute(&abi_item)));
            assert_eq!(val, &100);
        }
    }
}
