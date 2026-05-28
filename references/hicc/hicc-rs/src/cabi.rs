use crate::{AbiClass, ClassMethods};
use core::mem;

pub struct IsClass;
pub struct IsPOD;
pub struct IsValue;
pub struct IsPtr;
pub struct IsMutPtr;
pub struct IsRef;
pub struct IsRefMut;

pub trait ValueType: Sized {
    const N: usize;
    type Result;
    type Type: 'static;
    type Value: 'static;
}

impl<T> ValueType for T {
    default const N: usize = 0;
    default type Result = Self;
    default type Type = IsPOD;
    default type Value = IsValue;
}

impl<'a, T> ValueType for &'a T
where
    T: ValueType<Type = IsClass>,
{
    const N: usize = T::N + 1;
    type Result = T::Result;
    type Type = T::Type;
    type Value = IsRef;
}

impl<'a, T> ValueType for &'a mut T
where
    T: ValueType<Type = IsClass>,
{
    const N: usize = T::N + 1;
    type Result = T::Result;
    type Type = T::Type;
    type Value = IsRefMut;
}

impl<T> ValueType for *const T
where
    T: ValueType<Type = IsClass>,
{
    const N: usize = T::N + 1;
    type Result = T::Result;
    type Type = T::Type;
    type Value = IsPtr;
}

impl<T> ValueType for *mut T
where
    T: ValueType<Type = IsClass>,
{
    const N: usize = T::N + 1;
    type Result = T::Result;
    type Type = T::Type;
    type Value = IsMutPtr;
}

pub trait AbiType: ValueType {
    type InputType;
    type OutputType;
    fn into_abi(self) -> Self::OutputType;
    fn from_abi(src: Self::InputType) -> Self;
}

impl<T> AbiType for T
where
    T: ValueType,
{
    type InputType = <RustType<T, T::Type, T::Value> as AbiHelper<T>>::InputType;
    type OutputType = <RustType<T, T::Type, T::Value> as AbiHelper<T>>::OutputType;
    fn into_abi(self) -> Self::OutputType {
        <RustType<T, T::Type, T::Value> as AbiHelper<T>>::into_abi(self)
    }
    fn from_abi(src: Self::InputType) -> Self {
        <RustType<T, T::Type, T::Value> as AbiHelper<T>>::from_abi(src)
    }
}

pub trait AbiHelper<T> {
    type InputType;
    type OutputType;
    fn from_abi(src: Self::InputType) -> T;
    fn into_abi(src: T) -> Self::OutputType;
}

pub struct RustType<T, U, V>(T, U, V);

impl<T, U, V> AbiHelper<T> for RustType<T, U, V> {
    default type InputType = T;
    default type OutputType = T;
    default fn from_abi(_src: Self::InputType) -> T {
        todo!()
    }
    default fn into_abi(_src: T) -> Self::OutputType {
        todo!()
    }
}

impl<T: ValueType, V> AbiHelper<T> for RustType<T, IsPOD, V> {
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
    // 这里运行时判断大小是否一致.
    assert!(mem::size_of::<OUT>() == mem::size_of::<IN>());
    let p = &src as *const IN;
    let target = unsafe { p.cast::<OUT>().read() };
    mem::forget(src);
    target
}
