use core::mem::{self, ManuallyDrop};
use core::ptr;

pub struct AbiRefMethods<T>(AbiMethods<T>);
pub struct AbiRefMutMethods<T>(AbiMethods<T>);

pub trait ClassMethods: Sized {
    type Methods: 'static;
    const METHODS: &'static AbiMethods<Self::Methods>;
    const REF_METHODS: &'static AbiRefMethods<Self::Methods>;
    const REF_MUT_METHODS: &'static AbiRefMutMethods<Self::Methods>;
}

// 仅用于测试.
#[repr(C)]
#[derive(Debug)]
#[allow(dead_code)]
pub(crate) struct BaseMethods<T: ClassMethods> {
    pub(crate) destroy: unsafe extern "C" fn(AbiClass<T>),
    pub(crate) make_unique: unsafe extern "C" fn(AbiClass<T>) -> AbiClass<T>,
    pub(crate) make_ref_mut: unsafe extern "C" fn(&mut AbiClass<T>) -> AbiClass<T>,
    pub(crate) size_of: unsafe extern "C" fn() -> usize,
    pub(crate) write: unsafe extern "C" fn(&mut AbiClass<T>, AbiClass<T>),
    pub(crate) make_ref: unsafe extern "C" fn(&AbiClass<T>) -> AbiClass<T>,
}

#[repr(C)]
#[derive(Debug)]
pub struct AbiMethods<M> {
    pub base: [*const (); 6],
    pub methods: M,
}

#[repr(C)]
#[derive(Debug)]
pub struct AbiClass<T: ClassMethods> {
    pub methods: &'static AbiMethods<T::Methods>,
    pub this: *const (),
    pub level: usize,
}

impl<T: ClassMethods> Drop for AbiClass<T> {
    fn drop(&mut self) {
        unsafe {
            let destroy: unsafe extern "C" fn(Self) = mem::transmute(self.methods.base[0]);
            destroy(Self { ..*self });
        }
    }
}

impl<T: ClassMethods> AbiClass<T> {
    pub const fn new_methods(methods: T::Methods) -> AbiMethods<T::Methods> {
        AbiMethods::<T::Methods> {
            base: [
                Self::abi_destroy_boxed as *const (),
                Self::abi_make_unique as *const (),
                Self::abi_make_ref_mut as *const (),
                Self::abi_size_of as *const (),
                Self::abi_write as *const (),
                Self::abi_make_ref as *const (),
            ],
            methods,
        }
    }

    pub const fn new_ref_methods(methods: T::Methods) -> AbiRefMethods<T::Methods> {
        AbiRefMethods(AbiMethods::<T::Methods> {
            base: [
                Self::abi_destroy_ref as *const (),
                Self::abi_make_unique as *const (),
                Self::abi_make_ref_mut as *const (),
                Self::abi_size_of as *const (),
                Self::abi_write as *const (),
                Self::abi_make_ref as *const (),
            ],
            methods,
        })
    }

    pub const fn new_ref_mut_methods(methods: T::Methods) -> AbiRefMutMethods<T::Methods> {
        AbiRefMutMethods(AbiMethods::<T::Methods> {
            base: [
                Self::abi_destroy_ref_mut as *const (),
                Self::abi_make_unique as *const (),
                Self::abi_make_ref_mut as *const (),
                Self::abi_size_of as *const (),
                Self::abi_write as *const (),
                Self::abi_make_ref as *const (),
            ],
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

impl<T: ClassMethods> AbiClass<T> {
    pub fn with_boxed(obj: Box<T>) -> Self {
        let this = Box::into_raw(obj).cast();
        Self {
            methods: T::METHODS,
            this,
            level: 0,
        }
    }

    pub fn with_ptr(this: *const (), level: usize) -> Self {
        Self {
            methods: &T::REF_METHODS.0,
            this: this.cast_mut(),
            level,
        }
    }

    pub fn with_mut_ptr(this: *mut (), level: usize) -> Self {
        Self {
            methods: &T::REF_MUT_METHODS.0,
            this,
            level,
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
