use crate::ValueType;
use core::marker::PhantomData;
use core::mem::{self, ManuallyDrop};
use core::ptr;

pub struct AbiRefMethods<T: ClassMethods>(AbiMethods<T>);
pub struct AbiRefMutMethods<T: ClassMethods>(AbiMethods<T>);

/// 实际实现的Methods必须在首部包含AbiMethods的接口定义，相当于是其基类.
/// 跨语言调用的数据必须是具有'static生命周期，如果没有也需要Box包装后才能传递.
pub trait ClassMethods: ValueType + 'static {
    type Methods: 'static;
    const METHODS: &'static AbiMethods<Self>;
    const REF_METHODS: &'static AbiRefMethods<Self>;
    const REF_MUT_METHODS: &'static AbiRefMutMethods<Self>;
}

/// fn as_ref(&Option<T>) -> Option<&T>
/// Rust的单例化机制会导致上面的函数编译错误. 这种情况下,
///
pub trait RustMethods<T> {
    type Methods: 'static;
}

#[repr(C)]
pub struct BaseMethods<T: ClassMethods> {
    pub destroy: unsafe extern "C" fn(AbiClass<T>),
    pub make_unique: unsafe extern "C" fn(AbiClass<T>) -> AbiClass<T>,
    pub make_ref_mut: unsafe extern "C" fn(&mut AbiClass<T>) -> AbiClass<T>,
    pub size_of: unsafe extern "C" fn() -> usize,
    pub write: unsafe extern "C" fn(&mut AbiClass<T>, AbiClass<T>),
    pub make_ref: unsafe extern "C" fn(&AbiClass<T>) -> AbiClass<T>,
}

#[repr(C)]
pub struct AbiMethods<T: ClassMethods> {
    pub base: BaseMethods<T>,
    pub methods: T::Methods,
}

#[repr(C)]
pub struct AbiClass<T: ClassMethods> {
    pub methods: &'static AbiMethods<T>,
    pub this: *const (),
    pub level: usize,
    _mark: PhantomData<T>, // 实际拥有T的所有权.
}

impl<T> Drop for AbiClass<T>
where
    T: ClassMethods,
{
    fn drop(&mut self) {
        unsafe {
            (self.methods.base.destroy)(Self { ..*self });
        }
    }
}

impl<T> AbiClass<T>
where
    T: ClassMethods,
{
    const fn methods() -> BaseMethods<T> {
        BaseMethods::<T> {
            destroy: Self::abi_destroy_boxed,
            make_unique: Self::abi_make_unique,
            make_ref_mut: Self::abi_make_ref_mut,
            size_of: Self::abi_size_of,
            write: Self::abi_write,
            make_ref: Self::abi_make_ref,
        }
    }

    const fn ref_methods() -> BaseMethods<T> {
        BaseMethods::<T> {
            destroy: Self::abi_destroy_ref,
            ..Self::methods()
        }
    }

    const fn ref_mut_methods() -> BaseMethods<T> {
        BaseMethods::<T> {
            destroy: Self::abi_destroy_ref_mut,
            ..Self::methods()
        }
    }

    pub const fn new_methods(methods: T::Methods) -> AbiMethods<T> {
        AbiMethods::<T> {
            base: Self::methods(),
            methods,
        }
    }

    pub const fn new_ref_methods(methods: T::Methods) -> AbiRefMethods<T> {
        AbiRefMethods(AbiMethods::<T> {
            base: Self::ref_methods(),
            methods,
        })
    }

    pub const fn new_ref_mut_methods(methods: T::Methods) -> AbiRefMutMethods<T> {
        AbiRefMutMethods(AbiMethods::<T> {
            base: Self::ref_mut_methods(),
            methods,
        })
    }

    unsafe extern "C" fn abi_destroy_boxed(self) {
        let obj = ManuallyDrop::new(self);
        let _ = unsafe { Box::from_raw(obj.this.cast::<T>().cast_mut()) };
    }
    // abi_destroy_ref/abi_destroy_ref_mut
    // 内部现需要利用这两个常量区分只读指针还是可写指针.
    unsafe extern "C" fn abi_destroy_ref(self) {
        mem::forget(self);
    }
    unsafe extern "C" fn abi_destroy_ref_mut(self) {
        mem::forget(self);
    }
    unsafe extern "C" fn abi_make_unique(mut self) -> Self {
        if self.level == 0 {
            self.methods = T::METHODS;
        }
        self
    }
    unsafe extern "C" fn abi_make_ref_mut(&mut self) -> Self {
        Self {
            methods: if self.is_mut() {
                &T::REF_MUT_METHODS.0
            } else {
                &T::REF_METHODS.0
            },
            ..*self
        }
    }
    unsafe extern "C" fn abi_make_ref(&self) -> Self {
        Self {
            methods: &T::REF_METHODS.0,
            ..*self
        }
    }
    unsafe extern "C" fn abi_size_of() -> usize {
        mem::size_of::<T>()
    }
    unsafe extern "C" fn abi_write(&mut self, value: Self) {
        if self.is_mut() && self.level == 0 && value.level == 0 {
            let this = self.this.cast::<T>().cast_mut();
            unsafe { this.write(value.take_inner()) }
        }
    }
}

impl<T> AbiClass<T>
where
    T: ClassMethods,
{
    pub fn with_boxed(obj: Box<T>) -> Self {
        let this = Box::into_raw(obj).cast();
        Self {
            methods: T::METHODS,
            this,
            level: 0,
            _mark: PhantomData,
        }
    }
    pub fn with_ptr(this: *const (), level: usize) -> Self {
        Self {
            methods: &T::REF_METHODS.0,
            this: this.cast_mut(),
            level,
            _mark: PhantomData,
        }
    }

    pub fn with_mut_ptr(this: *mut (), level: usize) -> Self {
        Self {
            methods: &T::REF_MUT_METHODS.0,
            this,
            level,
            _mark: PhantomData,
        }
    }

    pub fn this_mut_ptr(&self, level: usize) -> *mut () {
        if self.is_mut() {
            return self.this_ptr(level).cast_mut();
        }
        panic!("not mut pointer");
    }

    pub fn this_ptr(&self, level: usize) -> *const () {
        if level > self.level {
            panic!(
                "expect {}-level pointer, but is {}-level pointer",
                level + 1,
                self.level + 1
            );
        }
        let mut this = self.this as *mut *mut ();
        for _ in level..self.level {
            this = unsafe { (*this).cast() };
        }
        this.cast()
    }

    pub fn take_inner(self) -> T {
        if self.is_value() {
            assert_eq!(self.level, 0);
            let this = ManuallyDrop::new(self);
            let this = unsafe { Box::from_raw(this.this_ptr(0).cast::<T>().cast_mut()) };
            return *this;
        }
        panic!("not Box<T>");
    }

    pub fn is_value(&self) -> bool {
        ptr::eq(self.methods, T::METHODS)
    }
    pub fn is_pointer(&self) -> bool {
        !self.is_value()
    }
    pub fn is_const(&self) -> bool {
        ptr::eq(self.methods, &T::REF_METHODS.0)
    }
    pub fn is_mut(&self) -> bool {
        !self.is_const()
    }
}
