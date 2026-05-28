use crate::{
    AbiClass, ClassMutPtr, ClassPtr, ClassRef, ClassRefMut, RustAny, RustHashKey, RustKey,
};
use std::marker::PhantomData;

/// 用于支持`c++`模板类. 对应`rust`侧泛型参数必须支持`AbiType`.
pub trait AbiType: Sized {
    type InputType;
    type InputPtr<'a, P, const N: usize>
    where
        Self: 'a;
    type InputMutPtr<'a, P, const N: usize>
    where
        Self: 'a;
    type OutputType;
    type OutputRef<'a>
    where
        Self: 'a;
    type OutputRefMut<'a>
    where
        Self: 'a;
    type OutputPtr<'a, P, const N: usize>
    where
        Self: 'a;
    type OutputMutPtr<'a, P, const N: usize>
    where
        Self: 'a;
}

/// `c++`模板参数, 非`c++类`必须包装在`pod<T>`中使用.
/// ```rust!
/// let std_vec = hicc::vector<hicc::Pod<i32>>::new();
/// ```
pub struct Pod<T: Sized>(PhantomData<T>);

unsafe impl<T: Send> Send for Pod<T> {}
unsafe impl<T: Sync> Sync for Pod<T> {}

impl<T: Sized> AbiType for Pod<T> {
    type InputType = T;
    type InputPtr<'a, P, const N: usize>
        = P
    where
        T: 'a;
    type InputMutPtr<'a, P, const N: usize>
        = P
    where
        T: 'a;
    type OutputType = T;
    type OutputRef<'a>
        = &'a T
    where
        T: 'a;
    type OutputRefMut<'a>
        = &'a mut T
    where
        T: 'a;
    type OutputPtr<'a, P, const N: usize>
        = P
    where
        T: 'a;
    type OutputMutPtr<'a, P, const N: usize>
        = P
    where
        T: 'a;
}

impl<T> AbiType for RustAny<T> {
    type InputType = RustAny<T>;
    type InputPtr<'a, P, const N: usize>
        = P
    where
        T: 'a;
    type InputMutPtr<'a, P, const N: usize>
        = P
    where
        T: 'a;
    type OutputType = RustAny<T>;
    type OutputRef<'a>
        = &'a RustAny<T>
    where
        T: 'a;
    type OutputRefMut<'a>
        = &'a mut RustAny<T>
    where
        T: 'a;
    type OutputPtr<'a, P, const N: usize>
        = P
    where
        T: 'a;
    type OutputMutPtr<'a, P, const N: usize>
        = P
    where
        T: 'a;
}

impl<T> AbiType for RustKey<T> {
    type InputType = RustKey<T>;
    type InputPtr<'a, P, const N: usize>
        = P
    where
        T: 'a;
    type InputMutPtr<'a, P, const N: usize>
        = P
    where
        T: 'a;
    type OutputType = RustKey<T>;
    type OutputRef<'a>
        = &'a RustKey<T>
    where
        T: 'a;
    type OutputRefMut<'a>
        = &'a mut RustKey<T>
    where
        T: 'a;
    type OutputPtr<'a, P, const N: usize>
        = P
    where
        T: 'a;
    type OutputMutPtr<'a, P, const N: usize>
        = P
    where
        T: 'a;
}

impl<T> AbiType for RustHashKey<T> {
    type InputType = RustHashKey<T>;
    type InputPtr<'a, P, const N: usize>
        = P
    where
        T: 'a;
    type InputMutPtr<'a, P, const N: usize>
        = P
    where
        T: 'a;
    type OutputType = RustHashKey<T>;
    type OutputRef<'a>
        = &'a RustHashKey<T>
    where
        T: 'a;
    type OutputRefMut<'a>
        = &'a mut RustHashKey<T>
    where
        T: 'a;
    type OutputPtr<'a, P, const N: usize>
        = P
    where
        T: 'a;
    type OutputMutPtr<'a, P, const N: usize>
        = P
    where
        T: 'a;
}

/// `c++`类可直接作为`c++`模板参数使用.
///
/// ```rust!
/// let vec_vec_string = hicc::vector<hicc::vector<hicc::string>>::new();
/// ```
impl<T: AbiClass> AbiType for T {
    type InputType = T;
    type InputPtr<'a, P, const N: usize>
        = &'a ClassPtr<'a, T, N>
    where
        T: 'a;
    type InputMutPtr<'a, P, const N: usize>
        = &'a ClassMutPtr<'a, T, N>
    where
        T: 'a;
    type OutputType = T;
    type OutputRef<'a>
        = ClassRef<'a, T>
    where
        T: 'a;
    type OutputRefMut<'a>
        = ClassRefMut<'a, T>
    where
        T: 'a;
    type OutputPtr<'a, P, const N: usize>
        = ClassPtr<'a, T, N>
    where
        T: 'a;
    type OutputMutPtr<'a, P, const N: usize>
        = ClassMutPtr<'a, T, N>
    where
        T: 'a;
}
