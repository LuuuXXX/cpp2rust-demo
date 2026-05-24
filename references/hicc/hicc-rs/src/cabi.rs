use crate::{AbiClass, ClassMethods, Depth, Depth0, Depth0_3};
use core::mem;

pub struct IsClass;
pub struct IsPOD;
pub struct IsValue;
pub struct IsPtr;
pub struct IsMutPtr;
pub struct IsRef;
pub struct IsRefMut;

pub trait ValueType {
    const N: usize;
    type Result;
    type Flag1;
    type Flag2;
    type Depth: Depth;
}

impl<T> ValueType for T {
    default const N: usize = 0;
    default type Result = Self;
    default type Flag1 = IsPOD;
    default type Flag2 = IsValue;
    default type Depth = Depth0;
}

impl<'a, T> ValueType for &'a T
where
    T: ValueType,
    T::Depth: Depth0_3,
{
    const N: usize = T::N + 1;
    type Result = T::Result;
    type Flag1 = T::Flag1;
    type Flag2 = IsRef;
    type Depth = <T::Depth as Depth>::Next;
}

impl<'a, T> ValueType for &'a mut T
where
    T: ValueType,
    T::Depth: Depth0_3,
{
    const N: usize = T::N + 1;
    type Result = T::Result;
    type Flag1 = T::Flag1;
    type Flag2 = IsRefMut;
    type Depth = <T::Depth as Depth>::Next;
}

impl<T> ValueType for *const T
where
    T: ValueType,
    T::Depth: Depth0_3,
{
    const N: usize = T::N + 1;
    type Result = T::Result;
    type Flag1 = T::Flag1;
    type Flag2 = IsPtr;
    type Depth = <T::Depth as Depth>::Next;
}

impl<T> ValueType for *mut T
where
    T: ValueType,
    T::Depth: Depth0_3,
{
    const N: usize = T::N + 1;
    type Result = T::Result;
    type Flag1 = T::Flag1;
    type Flag2 = IsMutPtr;
    type Depth = <T::Depth as Depth>::Next;
}

pub trait AbiType: ValueType {
    type InputType;
    type OutputType;
    fn into_abi(self) -> Self::OutputType;
    fn from_abi(src: Self::InputType) -> Self;
}

impl<T: ValueType> AbiType for T {
    default type InputType = <RustType<Self, Self::Flag1, Self::Flag2> as AbiHelper<Self>>::InputType;
    default type OutputType = <RustType<Self, Self::Flag1, Self::Flag2> as AbiHelper<Self>>::OutputType;
    default fn into_abi(self) -> Self::OutputType {
        unsafe {
            transmute(<RustType<Self, Self::Flag1, Self::Flag2> as AbiHelper<
                Self,
            >>::into_abi(self))
        }
    }
    default fn from_abi(src: Self::InputType) -> Self {
        unsafe {
            transmute(<RustType<Self, Self::Flag1, Self::Flag2> as AbiHelper<
                Self,
            >>::from_abi(transmute(src)))
        }
    }
}

pub trait AbiHelper<T: ValueType> {
    type InputType;
    type OutputType;
    fn from_abi(src: Self::InputType) -> T;
    fn into_abi(src: T) -> Self::OutputType;
}

pub struct RustType<T: ValueType, F1, F2>(T, F1, F2);

impl<T: ValueType, F1, F2> AbiHelper<T> for RustType<T, F1, F2> {
    default type InputType = T;
    default type OutputType = T;
    default fn from_abi(_src: Self::InputType) -> T {
        todo!()
    }
    default fn into_abi(_src: T) -> Self::OutputType {
        todo!()
    }
}

impl<T: ValueType, F2> AbiHelper<T> for RustType<T, IsPOD, F2> {
    type InputType = T;
    type OutputType = T;
    fn from_abi(src: Self::InputType) -> T {
        src
    }
    fn into_abi(src: T) -> Self::OutputType {
        src
    }
}

impl<T> AbiHelper<T> for RustType<T, IsClass, IsPtr>
where
    T: ValueType,
    T::Result: ClassMethods,
{
    type InputType = *mut AbiClass<T::Result>;
    type OutputType = AbiClass<T::Result>;
    fn from_abi(src: Self::InputType) -> T {
        // T一定是瘦指针.
        if !src.is_null() {
            let src = unsafe { &*src };
            let obj = src.this_ptr(T::N - 1);
            return unsafe { transmute(obj) };
        }
        unsafe { mem::zeroed() }
    }
    fn into_abi(src: T) -> Self::OutputType {
        // 这里T是指针，都是瘦指针, Self::Result就是AbiClass<T>;
        let this = unsafe { transmute(src) };
        let obj = AbiClass::<T::Result>::with_ptr(this, T::N - 1);
        unsafe { transmute(obj) }
    }
}

impl<T> AbiHelper<T> for RustType<T, IsClass, IsMutPtr>
where
    T: ValueType,
    T::Result: ClassMethods,
{
    type InputType = *mut AbiClass<T::Result>;
    type OutputType = AbiClass<T::Result>;
    fn from_abi(src: Self::InputType) -> T {
        // T一定是瘦指针.
        if !src.is_null() {
            let src = unsafe { &*src };
            let obj = src.this_mut_ptr(T::N - 1);
            return unsafe { transmute(obj) };
        }
        unsafe { mem::zeroed() }
    }
    fn into_abi(src: T) -> Self::OutputType {
        // 这里T是指针，都是瘦指针, Self::Result就是AbiClass<T>;
        let this = unsafe { transmute(src) };
        let obj = AbiClass::<T::Result>::with_mut_ptr(this, T::N - 1);
        unsafe { transmute(obj) }
    }
}

impl<T> AbiHelper<T> for RustType<T, IsClass, IsRef>
where
    T: ValueType,
    T::Result: ClassMethods,
{
    type InputType = *mut AbiClass<T::Result>;
    type OutputType = AbiClass<T::Result>;
    fn from_abi(src: Self::InputType) -> T {
        if !src.is_null() {
            let src = unsafe { &*src };
            let obj = src.this_ptr(T::N - 1);
            // T一定是引用，瘦指针.
            return unsafe { transmute(obj) };
        }
        panic!("not reference, null pointer");
    }
    fn into_abi(src: T) -> Self::OutputType {
        // 这里T是引用，都是瘦指针, Self::Result就是AbiClass<T>;
        let this = unsafe { transmute(src) };
        let obj = AbiClass::<T::Result>::with_ptr(this, T::N - 1);
        unsafe { transmute(obj) }
    }
}

impl<T> AbiHelper<T> for RustType<T, IsClass, IsRefMut>
where
    T: ValueType,
    T::Result: ClassMethods,
{
    type InputType = *mut AbiClass<T::Result>;
    type OutputType = AbiClass<T::Result>;
    fn from_abi(src: Self::InputType) -> T {
        if !src.is_null() {
            let src = unsafe { &*src };
            let obj = src.this_mut_ptr(T::N - 1);
            // T一定是引用，瘦指针.
            return unsafe { transmute(obj) };
        }
        panic!("not mut reference, null pointer");
    }
    fn into_abi(src: T) -> Self::OutputType {
        // 这里T是引用，都是瘦指针, Self::Result就是AbiClass<T>;
        let this = unsafe { transmute(src) };
        let obj = AbiClass::<T::Result>::with_mut_ptr(this, T::N - 1);
        unsafe { transmute(obj) }
    }
}

impl<T> AbiHelper<T> for RustType<T, IsClass, IsValue>
where
    T: ValueType,
    T: ClassMethods,
{
    type InputType = AbiClass<T>;
    type OutputType = AbiClass<T>;
    fn from_abi(src: Self::InputType) -> T {
        src.take_inner()
    }
    fn into_abi(src: T) -> Self::OutputType {
        AbiClass::<T>::with_boxed(Box::new(src))
    }
}

pub const unsafe fn transmute<IN, OUT>(src: IN) -> OUT {
    // 泛型无法正确推导出类型信息，编译器无法判断类型大小，不能使用标准库的transmute
    // 这里通过运行时判断来实现.
    assert!(mem::size_of::<OUT>() == mem::size_of::<IN>());
    let p = &src as *const IN;
    let target = unsafe { p.cast::<OUT>().read() };
    mem::forget(src);
    target
}

#[macro_export]
macro_rules! ExportClass {
    () => {
        const N: usize = 0;
        type Result = Self;
        type Flag1 = $crate::IsClass;
        type Flag2 = $crate::IsValue;
        type Depth = $crate::Depth0;
    };
}
