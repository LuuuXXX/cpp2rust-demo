use std::cmp::{Ordering, PartialEq, PartialOrd};
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;
use std::mem::ManuallyDrop;
use std::ops::{Deref, DerefMut};
use std::ptr;
use std::rc::Rc;
use std::sync::Arc;

#[repr(C)]
struct RustAnyMethods {
    destroy: unsafe extern "C" fn(*const ()),
    clone: unsafe extern "C" fn(*const ()) -> *const (),
    less: unsafe extern "C" fn(&RustAny, &RustAny) -> bool,
    equal: unsafe extern "C" fn(&RustAny, &RustAny) -> bool,
    hash: unsafe extern "C" fn(&RustAny) -> usize,
}

/// `c++`容器需要保存`rust`的数据时，`rust`的数据类型需要转换为`RustAny`保存.
///
/// # panic
///
/// 需要遵循如下使用规则，否则运行时会导致panic.
/// 1. `map/set`请使用`RustKey`
/// 2. `unordered_map/unordered_set`请使用`RustHashKey`.
/// 3. 缺省构造的数据不能直接访问，必须覆写. 比如`array<RustAny,
///    N>`应该用`fill(RustAny::new_clone(...))`初始化后才能访问.
/// 4. 依赖拷贝构造函数的接口，参数需要用`RustAny::new_clone`生成.
///
/// 可参考`examples/rust_any`.
///
#[repr(C)]
pub struct RustAny<T = ()> {
    methods: &'static RustAnyMethods,
    val: *const T,
    mark: PhantomData<T>,
}

pub enum RustPtr<T> {
    Box(Box<T>),
    Rc(Rc<T>),
    Arc(Arc<T>),
    Null,
}

impl<T> Drop for RustAny<T> {
    fn drop(&mut self) {
        if !self.val.is_null() {
            unsafe { (self.methods.destroy)(self.val.cast::<()>()) };
        }
    }
}

unsafe impl<T: Send> Send for RustAny<T> {}
unsafe impl<T: Sync> Sync for RustAny<T> {}

impl<T> Deref for RustAny<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        unsafe { &*self.val }
    }
}

impl<T> DerefMut for RustAny<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.val.cast_mut() }
    }
}

impl<T> RustAny<T> {
    pub fn new(val: T) -> Self {
        Self::with_box(Box::new(val))
    }
    pub fn with_box(val: Box<T>) -> Self {
        Self {
            methods: &Self::BOX_METHODS,
            val: Box::leak(val),
            mark: PhantomData,
        }
    }
    /// # Safety
    ///
    /// 等同`Box<T>`使用，业务保证不会出现并发冲突.
    ///
    pub unsafe fn with_arc(val: Arc<T>) -> Self {
        Self {
            methods: &Self::ARC_METHODS,
            val: Arc::into_raw(val),
            mark: PhantomData,
        }
    }
    /// # Safety
    ///
    /// 等同`Box<T>`使用，业务保证不会出现并发冲突.
    ///
    pub unsafe fn with_rc(val: Rc<T>) -> Self {
        Self {
            methods: &Self::RC_METHODS,
            val: Rc::into_raw(val),
            mark: PhantomData,
        }
    }

    pub fn take(self) -> RustPtr<T> {
        let this = ManuallyDrop::new(self);
        if this.val.is_null() {
            RustPtr::Null
        } else if ptr::eq(this.methods, &Self::RC_METHODS) {
            RustPtr::Rc(unsafe { Rc::from_raw(this.val) })
        } else if ptr::eq(this.methods, &Self::ARC_METHODS) {
            RustPtr::Arc(unsafe { Arc::from_raw(this.val) })
        } else {
            RustPtr::Box(unsafe { Box::from_raw(this.val.cast_mut()) })
        }
    }

    unsafe extern "C" fn destroy_box(val: *const ()) {
        let _ = unsafe { Box::from_raw(val.cast::<T>().cast_mut()) };
    }
    unsafe extern "C" fn destroy_arc(val: *const ()) {
        let _ = unsafe { Arc::from_raw(val.cast::<T>()) };
    }
    unsafe extern "C" fn destroy_rc(val: *const ()) {
        let _ = unsafe { Rc::from_raw(val.cast::<T>()) };
    }
    unsafe extern "C" fn clone_arc(this: *const ()) -> *const () {
        Arc::increment_strong_count(this.cast::<T>());
        this
    }
    unsafe extern "C" fn clone_rc(this: *const ()) -> *const () {
        Rc::increment_strong_count(this.cast::<T>());
        this
    }
    unsafe extern "C" fn no_clone(_: *const ()) -> *const () {
        panic!("don't support clone");
    }
    unsafe extern "C" fn no_less(_: &RustAny, _: &RustAny) -> bool {
        panic!("don't support less");
    }
    unsafe extern "C" fn no_equal(_: &RustAny, _: &RustAny) -> bool {
        panic!("don't support equal");
    }
    unsafe extern "C" fn no_hash(_: &RustAny) -> usize {
        panic!("don't support hash");
    }
    const BOX_METHODS: RustAnyMethods = RustAnyMethods {
        destroy: Self::destroy_box,
        clone: Self::no_clone,
        less: Self::no_less,
        equal: Self::no_equal,
        hash: Self::no_hash,
    };
    const ARC_METHODS: RustAnyMethods = RustAnyMethods {
        destroy: Self::destroy_arc,
        clone: Self::clone_arc,
        less: Self::no_less,
        equal: Self::no_equal,
        hash: Self::no_hash,
    };
    const RC_METHODS: RustAnyMethods = RustAnyMethods {
        destroy: Self::destroy_rc,
        clone: Self::clone_rc,
        less: Self::no_less,
        equal: Self::no_equal,
        hash: Self::no_hash,
    };
}

impl<T: Clone> RustAny<T> {
    pub fn new_clone(val: T) -> Self {
        Self::with_clone(Box::new(val))
    }
    pub fn with_clone(val: Box<T>) -> Self {
        Self {
            methods: &Self::CLONE_BOX_METHODS,
            val: Box::leak(val),
            mark: PhantomData,
        }
    }
    unsafe extern "C" fn clone_box(this: *const ()) -> *const () {
        let this = unsafe { &*this.cast::<T>() };
        let cloned: *mut T = Box::leak(Box::new(this.clone()));
        cloned.cast::<()>()
    }
    const CLONE_BOX_METHODS: RustAnyMethods = RustAnyMethods {
        destroy: Self::destroy_box,
        clone: Self::clone_box,
        less: Self::no_less,
        equal: Self::no_equal,
        hash: Self::no_hash,
    };
}

impl<T: PartialOrd> RustAny<T> {
    fn with_ord(val: Box<T>) -> Self {
        Self {
            methods: &Self::ORD_METHODS,
            val: Box::leak(val),
            mark: PhantomData,
        }
    }
    unsafe extern "C" fn do_less(this: &RustAny, other: &RustAny) -> bool {
        matches!(this.cmp(&**other), Ordering::Less)
    }
    const ORD_METHODS: RustAnyMethods = RustAnyMethods {
        destroy: Self::destroy_box,
        clone: Self::no_clone,
        less: Self::do_less,
        equal: Self::no_equal,
        hash: Self::no_hash,
    };
}

impl<T: Clone + PartialOrd> RustAny<T> {
    fn with_ord_clone(val: Box<T>) -> Self {
        Self {
            methods: &Self::ORD_CLONED_METHODS,
            val: Box::leak(val),
            mark: PhantomData,
        }
    }
    const ORD_CLONED_METHODS: RustAnyMethods = RustAnyMethods {
        destroy: Self::destroy_box,
        clone: Self::clone_box,
        less: Self::do_less,
        equal: Self::no_equal,
        hash: Self::no_hash,
    };
}

impl<T: PartialEq + Hash> RustAny<T> {
    fn with_hash(val: Box<T>) -> Self {
        Self {
            methods: &Self::HASH_METHODS,
            val: Box::leak(val),
            mark: PhantomData,
        }
    }
    const HASH_METHODS: RustAnyMethods = RustAnyMethods {
        destroy: Self::destroy_box,
        clone: Self::no_clone,
        less: Self::no_less,
        equal: Self::do_equal,
        hash: Self::do_hash,
    };
    unsafe extern "C" fn do_equal(this: &RustAny, other: &RustAny) -> bool {
        this.eq(&**other)
    }
    unsafe extern "C" fn do_hash(this: &RustAny) -> usize {
        #[rustversion::since(1.76.0)]
        type DefaultHasher = std::hash::DefaultHasher;
        #[rustversion::before(1.76.0)]
        type DefaultHasher = std::collections::hash_map::DefaultHasher;

        let mut h = DefaultHasher::new();
        this.hash(&mut h);
        h.finish() as usize
    }
}

impl<T: Clone + PartialEq + Hash> RustAny<T> {
    fn with_hash_clone(val: Box<T>) -> Self {
        Self {
            methods: &Self::HASH_CLONE_METHODS,
            val: Box::leak(val),
            mark: PhantomData,
        }
    }
    const HASH_CLONE_METHODS: RustAnyMethods = RustAnyMethods {
        destroy: Self::destroy_box,
        clone: Self::clone_box,
        less: Self::no_less,
        equal: Self::do_equal,
        hash: Self::do_hash,
    };
}

#[repr(transparent)]
pub struct RustKey<T>(RustAny<T>);

unsafe impl<T: Send> Send for RustKey<T> {}
unsafe impl<T: Sync> Sync for RustKey<T> {}

impl<T: PartialOrd> RustKey<T> {
    pub fn new(val: T) -> Self {
        Self::with_box(Box::new(val))
    }
    pub fn with_box(val: Box<T>) -> Self {
        Self(RustAny::<T>::with_ord(val))
    }
    pub fn take(self) -> RustPtr<T> {
        self.0.take()
    }
}

impl<T: Clone + PartialOrd> RustKey<T> {
    pub fn new_clone(val: T) -> Self {
        Self::with_clone(Box::new(val))
    }
    pub fn with_clone(val: Box<T>) -> Self {
        Self(RustAny::<T>::with_ord_clone(val))
    }
}

impl<T> Deref for RustKey<T> {
    type Target = RustAny<T>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for RustKey<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[repr(transparent)]
pub struct RustHashKey<T>(RustAny<T>);

unsafe impl<T: Send> Send for RustHashKey<T> {}
unsafe impl<T: Sync> Sync for RustHashKey<T> {}

impl<T: PartialEq + Hash> RustHashKey<T> {
    pub fn new(val: T) -> Self {
        Self::with_box(Box::new(val))
    }
    pub fn with_box(val: Box<T>) -> Self {
        Self(RustAny::<T>::with_hash(val))
    }
    pub fn take(self) -> RustPtr<T> {
        self.0.take()
    }
}

impl<T: Clone + PartialEq + Hash> RustHashKey<T> {
    pub fn new_clone(val: T) -> Self {
        Self::with_clone(Box::new(val))
    }
    pub fn with_clone(val: Box<T>) -> Self {
        Self(RustAny::<T>::with_hash_clone(val))
    }
}

impl<T> Deref for RustHashKey<T> {
    type Target = RustAny<T>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for RustHashKey<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
