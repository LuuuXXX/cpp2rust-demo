use crate::export_class;

type AnyTrait = &'static dyn::core::any::Any;
type AnyMutTrait = &'static mut dyn::core::any::Any;

#[export_class(in_hicc)]
impl AnyTrait {
    fn type_id(&self) -> [u8; 16] {
        unsafe { ::core::mem::transmute((*self).type_id()) }
    }
}

#[export_class(in_hicc)]
impl AnyMutTrait {
    fn type_id(&self) -> [u8; 16] {
        unsafe { ::core::mem::transmute((*self).type_id()) }
    }
}

/*
#[repr(C)]
pub struct HiccAnyMethods {
    pub type_id: unsafe extern "C" fn(
        <&HiccAnyTrait as AbiType>::InputType,
    ) -> <[u8; 16] as AbiType>::OutputType,
}

unsafe extern "C" fn hicc_any_type_id(
    this: <&HiccAnyTrait as AbiType>::InputType,
) -> <[u8; 16] as AbiType>::OutputType {
    let this = <&HiccAnyTrait as AbiType>::from_abi(this);
    let id = unsafe { ::core::mem::transmute((*this).type_id()) };
    <[u8; 16] as AbiType>::into_abi(id)
}

const fn hicc_any_methods() -> HiccAnyMethods {
    HiccAnyMethods {
        type_id: hicc_any_type_id,
    }
}

impl ValueType for HiccAnyTrait {
    const N: usize = 0;
    type Type = IsClass;
    type Value = IsValue;
}

impl ClassMethods for HiccAnyTrait {
    type Methods = HiccAnyMethods;
    const METHODS: &'static AbiMethods<Self> = &AbiClass::new_methods(hicc_any_methods());
    const REF_METHODS: &'static AbiRefMethods<Self> =
        &AbiClass::new_ref_methods(hicc_any_methods());
    const REF_MUT_METHODS: &'static AbiRefMutMethods<Self> =
        &AbiClass::new_ref_mut_methods(hicc_any_methods());
}

impl ValueType for HiccAnyMutTrait {
    const N: usize = 0;
    type Type = IsClass;
    type Value = IsValue;
}

impl ClassMethods for HiccAnyMutTrait {
    type Methods = HiccAnyMethods;
    const METHODS: &'static AbiMethods<Self> = &AbiClass::new_methods(hicc_any_methods());
    const REF_METHODS: &'static AbiRefMethods<Self> =
        &AbiClass::new_ref_methods(hicc_any_methods());
    const REF_MUT_METHODS: &'static AbiRefMutMethods<Self> =
        &AbiClass::new_ref_mut_methods(hicc_any_methods());
}
*/

#[cfg(test)]
mod test {
    use super::*;
    use crate::*;
    use ::core::any::{Any, TypeId};

    struct Foo;

    #[test]
    fn test() {
        unsafe {
            let any: &dyn Any = &Foo;
            let abi_any: AbiClass<&dyn Any> = transmute(<&dyn Any as AbiType>::into_abi(any));
            let abi_id = unsafe { (abi_any.methods.methods.type_id)(transmute(&abi_any)) };
            assert_eq!(transmute::<_, TypeId>(abi_id), Foo.type_id());
        }
    }
}
