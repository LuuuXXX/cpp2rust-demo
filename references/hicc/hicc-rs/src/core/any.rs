use crate::export_class;

type AnyTrait<'a> = &'a dyn ::core::any::Any;
type AnyMutTrait<'a> = &'a mut dyn ::core::any::Any;

#[export_class(in_hicc)]
impl<'a> AnyTrait<'a> {
    fn type_id(&self) -> [u8; 16] {
        unsafe { ::core::mem::transmute((*self).type_id()) }
    }
}

#[export_class(in_hicc)]
impl<'a> AnyMutTrait<'a> {
    fn type_id(&self) -> [u8; 16] {
        unsafe { ::core::mem::transmute((*self).type_id()) }
    }
}

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
            let any_m: AnyTraitMethods = transmute(abi_any.methods.methods);
            let abi_id = unsafe { (any_m.type_id)(transmute(&abi_any)) };
            assert_eq!(transmute::<_, TypeId>(abi_id), Foo.type_id());
            let abi_base: BaseMethods<&dyn Any> = transmute(abi_any.methods.base);
            (abi_base.destroy)(abi_any);
        }
    }
}
