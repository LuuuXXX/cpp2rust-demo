use std::cmp::{Eq, Ord, Ordering, PartialEq, PartialOrd};
use std::iter::Iterator;
use std::marker::PhantomData;
use std::mem::{size_of, size_of_val};
use std::ops::{Deref, DerefMut};

///
/// 遵循`RAII`原则管理`c++`资源的生命周期.
///
pub trait AbiClass: Sized {
    /// # Safety
    ///
    /// 对应到`c++`接口返回指针的应用场景.业务需要主动释放指针分配的资源时调用.
    /// 需要确保此时`self`是资源的唯一引用者.
    unsafe fn into_unique(self) -> Self;
    ///
    /// 更新对象本身.
    ///
    /// `rust`都是通过指针持有`c++`对象，必须调用`write`接口才能改写对象本身的值.
    ///
    /// 注意: 只有`std::is_assignable<T&, T>::value == true`的类型才可以成功改写，否则只是一个空操作.
    ///
    /// `hicc-std`提供的容器模板类，如果容器保存的是`c++`类类型，那么通过`iterator`改写容器保存的值，需要调用此接口.
    ///
    /// ```test
    /// let mut map = hicc_std::map::<hicc::Pod<i32>, hicc_std::string>::new();
    /// map.insert(&1, &hicc::string::from(c"hello"));
    /// map.insert(&2, &hicc::string::from(c"world"));
    /// //...
    /// // 注意对于`c++`类，直接改写可写引用的值仅仅是改写了`rust`侧的指针，实际并未修改`c++`侧的资源
    /// // map.iter_mut().for_each(|_, v| *v = hicc::string::from(c"null"));
    /// // 正确使用方式是调用`AbiClass::write`或者是`c++`类对象本身提供的成员函数完成修改.
    /// map.iter_mut().for_each(|k, v| v.write(hicc::string::from(c"null")));
    /// ```
    ///
    fn write(&mut self, val: Self);
    /// 返回`c++`对象的大小.
    fn size_of(&self) -> usize;
    /// rust侧的值类型实际都对应到c++侧的对象指针, 检查是否为空指针.
    fn is_null(&self) -> bool {
        self.get_obj().is_null()
    }

    /// `T`, `ClassRef<'_, T>`, `ClassRefMut<'_, T>`, `ClassPtr<'_, T>`, `ClassMutPtr<'_,
    /// T>`作为返回值实质是等价.
    ///
    /// 不同接口的返回值可能需要比较等操作时可调用.
    fn as_ref(&self) -> ClassRef<'_, Self> {
        ClassRef::new(unsafe { self.make_ref(self.get_obj()) })
    }

    /// `T`, `ClassRef<'_, T>`, `ClassRefMut<'_, T>`, `ClassPtr<'_, T>`, `ClassMutPtr<'_,
    /// T>`作为返回值实质是等价.
    ///
    /// 不同接口的返回值可能需要比较等操作时可调用.
    fn as_mut(&mut self) -> ClassRefMut<'_, Self> {
        ClassRefMut::new(unsafe { self.make_ref(self.get_obj()) })
    }

    /// `T`, `ClassRef<'_, T>`, `ClassRefMut<'_, T>`, `ClassPtr<'_, T>`, `ClassMutPtr<'_,
    /// T>`作为返回值实质是等价.
    ///
    /// `AbiType::InputPtr<'_, T>`参数类型实际是`&ClassPtr<'_, T>`,
    /// 当需要传递这类参数时可调用此接口.
    fn as_ptr(&self) -> ClassPtr<'_, Self> {
        ClassPtr::new(unsafe { self.make_ref(self.get_obj()) })
    }

    /// `AbiType::InputMutPtr<'_, T>`参数类型实际是`&ClassMutPtr<'_, T>`,
    /// 当需要传递这类参数时可调用此接口.
    fn as_mut_ptr(&mut self) -> ClassMutPtr<'_, Self> {
        ClassMutPtr::new(unsafe { self.make_ref(self.get_obj()) })
    }

    /// `T`, `ClassRef<'_, T>`, `ClassRefMut<'_, T>`, `ClassPtr<'_, T>`, `ClassMutPtr<'_,
    /// T>`作为返回值实质是等价.
    fn into_ref(self) -> ClassRef<'static, Self> {
        ClassRef::new(self)
    }

    /// `T`, `ClassRef<'_, T>`, `ClassRefMut<'_, T>`, `ClassPtr<'_, T>`, `ClassMutPtr<'_,
    /// T>`作为返回值实质是等价.
    fn into_mut(self) -> ClassRefMut<'static, Self> {
        ClassRefMut::new(self)
    }

    /// `T`, `ClassRef<'_, T>`, `ClassRefMut<'_, T>`, `ClassPtr<'_, T>`, `ClassMutPtr<'_,
    /// T>`作为返回值实质是等价.
    fn into_ptr(self) -> ClassPtr<'static, Self> {
        ClassPtr::new(self)
    }

    /// `T`, `ClassRef<'_, T>`, `ClassRefMut<'_, T>`, `ClassPtr<'_, T>`, `ClassMutPtr<'_,
    /// T>`作为返回值实质是等价.
    fn into_mut_ptr(self) -> ClassMutPtr<'static, Self> {
        ClassMutPtr::new(self)
    }

    /// 仅内部使用.指针携带附加信息，不能转换为`c++`类指针.
    fn get_raw_obj(&self) -> *const ();
    /// # Safety
    ///
    /// 仅内部使用. 指针需确保obj符合内部管理规则. 返回对象不负责释放`c++`资源.
    unsafe fn make_ref(&self, obj: *const ()) -> Self;
    /// 一般内部使用. 返回值等同于`c++`侧的指针。
    fn get_obj(&self) -> *const () {
        let obj = self.get_raw_obj() as usize;
        let obj = obj & !(size_of::<*const ()>() - 1);
        obj as *const ()
    }
}

/// 类型标识，用于需要从`rust`的trait创建`c++`类对象的场景.
#[repr(transparent)]
pub struct Interface<T>(T);

unsafe impl<T: AbiClass + Sync> Send for Interface<T> {}
unsafe impl<T: AbiClass + Sync> Sync for Interface<T> {}

impl<T: AbiClass> Interface<T> {
    pub fn new(val: T) -> Self {
        Self(val)
    }
}

/// 对应`&T`.
///
/// 可用于`c++`接口返回值类型或者`std::unique_ptr<T>`类型. 强调其生命周期依赖某个参数.
#[repr(C)]
#[derive(Debug)]
pub struct ClassRef<'a, T: AbiClass> {
    inner: T,
    _mark: PhantomData<&'a T>,
}

unsafe impl<T: AbiClass + Sync> Send for ClassRef<'_, T> {}
unsafe impl<T: AbiClass + Sync> Sync for ClassRef<'_, T> {}

impl<'a, T: AbiClass> ClassRef<'a, T> {
    fn new(obj: T) -> Self {
        Self {
            inner: obj,
            _mark: PhantomData,
        }
    }

    /// `AbiType::InputPtr<'_, T>`参数类型实际是`&ClassPtr<'_, T>`,
    /// 当需要传递这类参数时可调用此接口.
    pub fn as_ptr(&self) -> ClassPtr<'a, T> {
        ClassPtr::new(unsafe { self.inner.make_ref(self.inner.get_obj()) })
    }

    /// # Safety
    ///
    /// 接口依赖`'a`生命周期时调用，调用者保证在资源实际生命周期长于`'a`.
    ///
    pub unsafe fn as_deref(&self) -> &'a T {
        &*(&self.inner as *const T)
    }

    /// # Safety
    ///
    /// 对应`static_const`应用场景, 用户保证不会出现读写冲突.
    pub unsafe fn into_mut(self) -> ClassRefMut<'a, T> {
        ClassRefMut::new(self.inner)
    }

    /// # Safety
    ///
    /// `T`, `ClassRef<'_, T>`, `ClassRefMut<'_, T>`, `ClassPtr<'_, T>`, `ClassMutPtr<'_,
    /// T>`作为返回值实质是等价.
    ///
    /// 调用此接口后，和其他资源的生命周期关联关系丢失，调用者负责生命周期管理.
    /// 一般应仅用于参数传递场景.
    ///
    pub unsafe fn into_value(self) -> T {
        self.inner
    }

    /// `T`, `ClassRef<'_, T>`, `ClassRefMut<'_, T>`, `ClassPtr<'_, T>`, `ClassMutPtr<'_,
    /// T>`作为返回值实质是等价.
    pub fn into_ptr(self) -> ClassPtr<'a, T> {
        ClassPtr::new(self.inner)
    }

}

impl<T: AbiClass> Deref for ClassRef<'_, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T: AbiClass + PartialEq> PartialEq for ClassRef<'_, T> {
    fn eq(&self, other: &Self) -> bool {
        PartialEq::eq(&self.inner, &other.inner)
    }
}

impl<T: AbiClass + Eq> Eq for ClassRef<'_, T> {}

impl<T: AbiClass + PartialOrd> PartialOrd for ClassRef<'_, T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        PartialOrd::partial_cmp(&self.inner, &other.inner)
    }
}

impl<T: AbiClass + Ord> Ord for ClassRef<'_, T> {
    fn cmp(&self, other: &Self) -> Ordering {
        Ord::cmp(&self.inner, &other.inner)
    }
}

/// 对应`&mut T`.
///
/// 用于`c++`接口返回值类型或者`std::unique_ptr<T>`类型. 强调其生命周期依赖某个参数.
#[repr(C)]
#[derive(Debug)]
pub struct ClassRefMut<'a, T: AbiClass> {
    inner: T,
    _mark: PhantomData<&'a T>,
}

unsafe impl<T: AbiClass + Sync> Send for ClassRefMut<'_, T> {}
unsafe impl<T: AbiClass + Sync> Sync for ClassRefMut<'_, T> {}

impl<'a, T: AbiClass> ClassRefMut<'a, T> {
    fn new(obj: T) -> Self {
        Self {
            inner: obj,
            _mark: PhantomData,
        }
    }
    
    /// `AbiType::InputPtr<'_, T>`参数类型实际是`&ClassPtr<'_, T>`,
    /// 当需要传递这类参数时可调用此接口.
    pub fn as_ptr(&self) -> ClassPtr<'a, T> {
        ClassPtr::new(unsafe { self.inner.make_ref(self.inner.get_obj()) })
    }

    /// `AbiType::InputMutPtr<'_, T>`参数类型实际是`&ClassMutPtr<'_, T>`,
    /// 当需要传递这类参数时可调用此接口.
    pub fn as_mut_ptr(&self) -> ClassMutPtr<'a, T> {
        ClassMutPtr::new(unsafe { self.inner.make_ref(self.inner.get_obj()) })
    }

    /// # Safety
    ///
    /// 接口依赖`'a`生命周期时调用，调用者保证在资源实际生命周期长于`'a`.
    ///
    pub unsafe fn as_deref(&self) -> &'a T {
        &*(&self.inner as *const T)
    }

    /// # Safety
    ///
    /// 接口依赖`'a`生命周期时调用，调用者保证在资源实际生命周期长于`'a`.
    ///
    pub unsafe fn as_deref_mut(&mut self) -> &'a mut T {
        &mut *(&mut self.inner as *mut T)
    }

    /// 对应`static_cast`应用场景
    pub fn into_ref(self) -> ClassRef<'a, T> {
        ClassRef::new(self.inner)
    }
    /// # Safety
    ///
    /// `T`, `ClassRef<'_, T>`, `ClassRefMut<'_, T>`, `ClassPtr<'_, T>`, `ClassMutPtr<'_,
    /// T>`作为返回值实质是等价.
    ///
    /// 调用此接口后，和其他资源的生命周期关联关系丢失，调用者负责生命周期管理.
    /// 一般应仅用于参数传递场景.
    ///
    pub unsafe fn into_value(self) -> T {
        self.inner
    }

    /// `T`, `ClassRef<'_, T>`, `ClassRefMut<'_, T>`, `ClassPtr<'_, T>`, `ClassMutPtr<'_,
    /// T>`作为返回值实质是等价.
    pub fn into_ptr(self) -> ClassPtr<'a, T> {
        ClassPtr::new(self.inner)
    }

    /// `T`, `ClassRef<'_, T>`, `ClassRefMut<'_, T>`, `ClassPtr<'_, T>`, `ClassMutPtr<'_,
    /// T>`作为返回值实质是等价.
    pub fn into_mut_ptr(self) -> ClassMutPtr<'a, T> {
        ClassMutPtr::new(self.inner)
    }
}

impl<T: AbiClass> Deref for ClassRefMut<'_, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T: AbiClass> DerefMut for ClassRefMut<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl<T: AbiClass + PartialEq> PartialEq for ClassRefMut<'_, T> {
    fn eq(&self, other: &Self) -> bool {
        PartialEq::eq(&self.inner, &other.inner)
    }
}

impl<T: AbiClass + Eq> Eq for ClassRefMut<'_, T> {}

impl<T: AbiClass + PartialOrd> PartialOrd for ClassRefMut<'_, T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        PartialOrd::partial_cmp(&self.inner, &other.inner)
    }
}

impl<T: AbiClass + Ord> Ord for ClassRefMut<'_, T> {
    fn cmp(&self, other: &Self) -> Ordering {
        Ord::cmp(&self.inner, &other.inner)
    }
}

///
/// 对应`c++`类对象只读数组
///
/// `c++`接口常见用指针和数组大小来表示一个数组.
///
#[derive(Debug)]
pub struct ClassArray<'a, T: AbiClass> {
    size: usize,
    inner: T,
    _mark: PhantomData<&'a T>,
}

unsafe impl<T: AbiClass + Sync> Send for ClassArray<'_, T> {}
unsafe impl<T: AbiClass + Sync> Sync for ClassArray<'_, T> {}

impl<T: AbiClass> ClassArray<'_, T> {
    unsafe fn new(inner: T, size: usize) -> Self {
        Self {
            size,
            inner,
            _mark: PhantomData,
        }
    }
}

impl<'a, T: AbiClass> ClassArray<'a, T> {
    pub fn size(&self) -> usize {
        self.size
    }
    pub fn is_empty(&self) -> bool {
        self.size == 0_usize
    }
    pub fn get(&self, pos: usize) -> Option<ClassRef<'a, T>> {
        if pos >= self.size {
            return None;
        }
        let ptr = self.inner.get_obj() as usize + self.inner.size_of() * pos;
        unsafe { Some(ClassRef::new(self.inner.make_ref(ptr as *const ()))) }
    }
    pub fn iter(&self) -> impl Iterator<Item = ClassRef<'a, T>> + '_ {
        ClassArrayIter {
            array: self,
            pos: 0,
        }
    }
    pub fn rev_iter(&self) -> impl Iterator<Item = ClassRef<'a, T>> + '_ {
        ClassArrayRevIter {
            array: self,
            pos: self.size,
        }
    }
}

struct ClassArrayIter<'a, T: AbiClass> {
    array: *const ClassArray<'a, T>,
    pos: usize,
}

impl<'a, T: AbiClass> Iterator for ClassArrayIter<'a, T> {
    type Item = ClassRef<'a, T>;
    fn next(&mut self) -> Option<Self::Item> {
        let array = unsafe { &*self.array };
        if self.pos < array.size {
            self.pos += 1;
            array.get(self.pos - 1)
        } else {
            None
        }
    }
    fn count(self) -> usize {
        let array = unsafe { &*self.array };
        array.size() - self.pos
    }
}

struct ClassArrayRevIter<'a, T: AbiClass> {
    array: *const ClassArray<'a, T>,
    pos: usize,
}

impl<'a, T: AbiClass> Iterator for ClassArrayRevIter<'a, T> {
    type Item = ClassRef<'a, T>;
    fn next(&mut self) -> Option<Self::Item> {
        let array = unsafe { &*self.array };
        if self.pos > 0 {
            self.pos -= 1;
            array.get(self.pos)
        } else {
            None
        }
    }
    fn count(self) -> usize {
        self.pos
    }
}

impl<T: AbiClass + PartialEq> PartialEq for ClassArray<'_, T> {
    fn eq(&self, other: &Self) -> bool {
        for n in 0..self.size.min(other.size) {
            if self.get(n) != other.get(n) {
                return false;
            }
        }
        self.size == other.size
    }
}

impl<T: AbiClass + Eq> Eq for ClassArray<'_, T> {}

impl<T: AbiClass + PartialOrd> PartialOrd for ClassArray<'_, T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        for n in 0..self.size.min(other.size) {
            let ord = PartialOrd::partial_cmp(&self.get(n), &other.get(n));
            if !matches!(ord, Some(Ordering::Equal)) {
                return ord;
            }
        }
        self.size.partial_cmp(&other.size)
    }
}

impl<T: AbiClass + Ord> Ord for ClassArray<'_, T> {
    fn cmp(&self, other: &Self) -> Ordering {
        for n in 0..self.size.min(other.size) {
            let ord = Ord::cmp(&self.get(n), &other.get(n));
            if ord != Ordering::Equal {
                return ord;
            }
        }
        self.size.cmp(&other.size)
    }
}

///
/// 对应`c++`类对象可写数组
///
/// `c++`接口常见用指针和数组大小来表示一个数组.
///
#[derive(Debug)]
pub struct ClassMutArray<'a, T: AbiClass> {
    size: usize,
    inner: T,
    _mark: PhantomData<&'a mut T>,
}

unsafe impl<T: AbiClass + Sync> Send for ClassMutArray<'_, T> {}
unsafe impl<T: AbiClass + Sync> Sync for ClassMutArray<'_, T> {}

impl<T: AbiClass> ClassMutArray<'_, T> {
    unsafe fn new(inner: T, size: usize) -> Self {
        Self {
            size,
            inner,
            _mark: PhantomData,
        }
    }
}

impl<'a, T: AbiClass> ClassMutArray<'a, T> {
    pub fn size(&self) -> usize {
        self.size
    }
    pub fn is_empty(&self) -> bool {
        self.size == 0
    }
    pub fn get(&self, pos: usize) -> Option<ClassRef<'a, T>> {
        if pos >= self.size {
            return None;
        }
        let ptr = self.inner.get_obj() as usize + self.inner.size_of() * pos;
        unsafe { Some(ClassRef::new(self.inner.make_ref(ptr as *const ()))) }
    }
    pub fn get_mut(&mut self, pos: usize) -> Option<ClassRefMut<'a, T>> {
        if pos >= self.size {
            return None;
        }
        let ptr = self.inner.get_obj() as usize + self.inner.size_of() * pos;
        unsafe { Some(ClassRefMut::new(self.inner.make_ref(ptr as *const ()))) }
    }

    pub fn iter(&self) -> impl Iterator<Item = ClassRef<'a, T>> + '_ {
        ClassMutArrayIter {
            array: self,
            pos: 0,
        }
    }
    pub fn rev_iter(&self) -> impl Iterator<Item = ClassRef<'a, T>> + '_ {
        ClassMutArrayRevIter {
            array: self,
            pos: self.size,
        }
    }
    pub fn iter_mut(&mut self) -> impl Iterator<Item = ClassRefMut<'a, T>> + '_ {
        ClassMutArrayIterMut {
            array: self,
            pos: 0,
        }
    }
    pub fn rev_iter_mut(&mut self) -> impl Iterator<Item = ClassRefMut<'a, T>> + '_ {
        ClassMutArrayRevIterMut {
            array: self,
            pos: self.size,
        }
    }
}

struct ClassMutArrayIter<'a, T: AbiClass> {
    array: *const ClassMutArray<'a, T>,
    pos: usize,
}

impl<'a, T: AbiClass> Iterator for ClassMutArrayIter<'a, T> {
    type Item = ClassRef<'a, T>;
    fn next(&mut self) -> Option<Self::Item> {
        let array = unsafe { &*self.array };
        if self.pos < array.size {
            self.pos += 1;
            array.get(self.pos - 1)
        } else {
            None
        }
    }
    fn count(self) -> usize {
        let array = unsafe { &*self.array };
        array.size() - self.pos
    }
}

struct ClassMutArrayRevIter<'a, T: AbiClass> {
    array: *const ClassMutArray<'a, T>,
    pos: usize,
}

impl<'a, T: AbiClass> Iterator for ClassMutArrayRevIter<'a, T> {
    type Item = ClassRef<'a, T>;
    fn next(&mut self) -> Option<Self::Item> {
        let array = unsafe { &*self.array };
        if self.pos > 0 {
            self.pos -= 1;
            array.get(self.pos)
        } else {
            None
        }
    }
    fn count(self) -> usize {
        self.pos
    }
}

struct ClassMutArrayIterMut<'a, T: AbiClass> {
    array: *mut ClassMutArray<'a, T>,
    pos: usize,
}

impl<'a, T: AbiClass> Iterator for ClassMutArrayIterMut<'a, T> {
    type Item = ClassRefMut<'a, T>;
    fn next(&mut self) -> Option<Self::Item> {
        let array = unsafe { &mut *self.array };
        if self.pos < array.size {
            self.pos += 1;
            array.get_mut(self.pos - 1)
        } else {
            None
        }
    }
}

struct ClassMutArrayRevIterMut<'a, T: AbiClass> {
    array: *mut ClassMutArray<'a, T>,
    pos: usize,
}

impl<'a, T: AbiClass> Iterator for ClassMutArrayRevIterMut<'a, T> {
    type Item = ClassRefMut<'a, T>;
    fn next(&mut self) -> Option<Self::Item> {
        let array = unsafe { &mut *self.array };
        if self.pos > 0 {
            self.pos -= 1;
            array.get_mut(self.pos)
        } else {
            None
        }
    }
}

impl<T: AbiClass + PartialEq> PartialEq for ClassMutArray<'_, T> {
    fn eq(&self, other: &Self) -> bool {
        for n in 0..self.size.min(other.size) {
            if self.get(n) != other.get(n) {
                return false;
            }
        }
        self.size == other.size
    }
}

impl<T: AbiClass + Eq> Eq for ClassMutArray<'_, T> {}

impl<T: AbiClass + PartialOrd> PartialOrd for ClassMutArray<'_, T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        for n in 0..self.size.min(other.size) {
            let ord = PartialOrd::partial_cmp(&self.get(n), &other.get(n));
            if !matches!(ord, Some(Ordering::Equal)) {
                return ord;
            }
        }
        self.size.partial_cmp(&other.size)
    }
}

impl<T: AbiClass + Ord> Ord for ClassMutArray<'_, T> {
    fn cmp(&self, other: &Self) -> Ordering {
        for n in 0..self.size.min(other.size) {
            let ord = Ord::cmp(&self.get(n), &other.get(n));
            if ord != Ordering::Equal {
                return ord;
            }
        }
        self.size.cmp(&other.size)
    }
}

///
/// 对应`const T*/const T**/...`
///
#[repr(transparent)]
#[derive(Debug)]
pub struct ClassPtr<'a, T: AbiClass, const N: usize = 1> {
    inner: T,
    _mark: PhantomData<&'a T>,
}

unsafe impl<T: AbiClass + Sync, const N: usize> Send for ClassPtr<'_, T, N> {}
unsafe impl<T: AbiClass + Sync, const N: usize> Sync for ClassPtr<'_, T, N> {}

impl<'a, T: AbiClass, const N: usize> ClassPtr<'a, T, N> {
    pub fn is_null(&self) -> bool {
        self.inner.is_null()
    }

    /// # Safety
    ///
    /// `static_cast`应用场景. 调用者保证不出现读写冲突.
    ///
    pub unsafe fn into_mut_ptr(self) -> ClassMutPtr<'a, T, N> {
        ClassMutPtr::new(self.inner)
    }

    fn new(val: T) -> Self {
        Self {
            inner: val,
            _mark: PhantomData,
        }
    }
    unsafe fn unsafe_read<const M: usize>(&self) -> ClassPtr<'a, T, M> {
        assert!(check_ptr::<N>(self.inner.get_raw_obj()));
        assert!(!self.is_null());
        assert!(M + 1 == N);
        assert!(N > 1);
        let pobj = self.inner.get_obj() as *const *const ();
        let obj = ((pobj.read() as usize) | (M - 1)) as *const ();
        ClassPtr::<'a, T, M>::new(self.inner.make_ref(obj))
    }
}

impl<'a, T: AbiClass> ClassPtr<'a, T, 1> {
    /*
    /// # panic
    /// 如果为空指针会panic.
    pub fn as_ref(&self) -> ClassRef<'a, T> {
        assert!(check_ptr::<1>(self.inner.get_raw_obj()));
        assert!(!self.inner.is_null());
        unsafe { ClassRef::new(self.inner.make_ref(self.inner.get_obj())) }
    }
    */

    /// # Safety
    ///
    /// 接口依赖`'a`生命周期时调用，调用者保证在资源实际生命周期长于`'a`.
    ///
    /// # panic
    /// 如果为空指针会panic.
    pub unsafe fn as_deref(&self) -> &'a T {
        assert!(check_ptr::<1>(self.inner.get_raw_obj()));
        assert!(!self.inner.is_null());
        &*(&self.inner as *const T)
    }

    /// # Safety
    /// 调用者保证输入参数`count`的正确性.
    /// # panic
    /// 如果为空指针且count大于0会panic.
    pub unsafe fn into_array(self, count: usize) -> ClassArray<'a, T> {
        assert!(check_ptr::<1>(self.inner.get_raw_obj()));
        assert!(!self.inner.is_null());
        assert!(!self.inner.is_null() && count > 0);
        ClassArray::new(self.inner, count)
    }

    /// # Safety
    ///
    /// `T`, `ClassRef<'_, T>`, `ClassRefMut<'_, T>`, `ClassPtr<'_, T>`, `ClassMutPtr<'_,
    /// T>`作为返回值实质是等价.
    ///
    /// 调用此接口后，和其他资源的生命周期关联关系丢失，调用者负责生命周期管理.
    /// 一般应仅用于参数传递场景.
    ///
    /// # panic
    /// 如果为空指针会panic.
    pub unsafe fn into_value(self) -> T {
        assert!(check_ptr::<1>(self.inner.get_raw_obj()));
        assert!(!self.inner.is_null());
        self.inner
    }

    /// `T`, `ClassRef<'_, T>`, `ClassRefMut<'_, T>`, `ClassPtr<'_, T>`, `ClassMutPtr<'_,
    /// T>`作为返回值实质是等价.
    ///
    /// # panic
    /// 如果为空指针会panic.
    pub fn as_ref(&self) -> ClassRef<'a, T> {
        assert!(check_ptr::<1>(self.inner.get_raw_obj()));
        unsafe { self.as_deref().as_ref() }
    }


    /// `T`, `ClassRef<'_, T>`, `ClassRefMut<'_, T>`, `ClassPtr<'_, T>`, `ClassMutPtr<'_,
    /// T>`作为返回值实质是等价.
    pub fn into_ref(self) -> ClassRef<'a, T> {
        assert!(check_ptr::<1>(self.inner.get_raw_obj()));
        assert!(!self.inner.is_null());
        ClassRef::new(self.inner)
    }
}

impl<T: AbiClass> Deref for ClassPtr<'_, T, 1> {
    type Target = T;
    /// # panic
    /// 空指针会导致panic
    fn deref(&self) -> &Self::Target {
        assert!(check_ptr::<1>(self.inner.get_raw_obj()));
        assert!(!self.inner.is_null());
        &self.inner
    }
}

impl<T: AbiClass + PartialEq> PartialEq for ClassPtr<'_, T> {
    fn eq(&self, other: &Self) -> bool {
        PartialEq::eq(&self.inner, &other.inner)
    }
}

impl<T: AbiClass + Eq> Eq for ClassPtr<'_, T> {}

impl<T: AbiClass + PartialOrd> PartialOrd for ClassPtr<'_, T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        PartialOrd::partial_cmp(&self.inner, &other.inner)
    }
}

impl<T: AbiClass + Ord> Ord for ClassPtr<'_, T> {
    fn cmp(&self, other: &Self) -> Ordering {
        Ord::cmp(&self.inner, &other.inner)
    }
}

/// const T**: 2重指针
impl<'a, T: AbiClass> ClassPtr<'a, T, 2> {
    /// # Safety
    /// 如果为空指针，则会panic
    pub unsafe fn read(&self) -> ClassPtr<'a, T, 1> {
        self.unsafe_read::<1>()
    }
}

/// const T***: 3重指针
impl<'a, T: AbiClass> ClassPtr<'a, T, 3> {
    /// # Safety
    /// 如果为空指针，则会panic
    pub unsafe fn read(&self) -> ClassPtr<'a, T, 2> {
        self.unsafe_read::<2>()
    }
}

/// const T****: 4重指针
impl<'a, T: AbiClass> ClassPtr<'a, T, 4> {
    /// # Safety
    /// 如果为空指针，则会panic
    pub unsafe fn read(&self) -> ClassPtr<'a, T, 3> {
        self.unsafe_read::<3>()
    }
}

/// T*****: 5重指针
#[cfg(target_pointer_width = "64")]
impl<'a, T: AbiClass> ClassPtr<'a, T, 5> {
    /// # Safety
    /// 如果为空指针，则会panic
    pub unsafe fn read(&self) -> ClassPtr<'a, T, 4> {
        self.unsafe_read::<4>()
    }
}

/// T******: 6重指针
#[cfg(target_pointer_width = "64")]
impl<'a, T: AbiClass> ClassPtr<'a, T, 6> {
    /// # Safety
    /// 如果为空指针，则会panic
    pub unsafe fn read(&self) -> ClassPtr<'a, T, 5> {
        self.unsafe_read::<5>()
    }
}

///
/// 对应`T*/T**/...`
///
#[repr(transparent)]
#[derive(Debug)]
pub struct ClassMutPtr<'a, T: AbiClass, const N: usize = 1> {
    inner: T,
    _mark: PhantomData<&'a T>,
}

unsafe impl<T: AbiClass + Sync, const N: usize> Send for ClassMutPtr<'_, T, N> {}
unsafe impl<T: AbiClass + Sync, const N: usize> Sync for ClassMutPtr<'_, T, N> {}

/// 对应`T*/T**/...`
///
/// 如果是返回类型，应该是ClassMutPtr<'_, T, N>.
///
/// 如果是参数类型, 应该是`&ClassMutPtr<'_, T, N>`.
///
impl<'a, T: AbiClass, const N: usize> ClassMutPtr<'a, T, N> {
    pub fn is_null(&self) -> bool {
        self.inner.is_null()
    }

    /// `static_cast`应用场景.
    pub fn into_ptr(self) -> ClassPtr<'a, T, N> {
        ClassPtr::new(self.inner)
    }

    fn new(val: T) -> Self {
        Self {
            inner: val,
            _mark: PhantomData,
        }
    }

    unsafe fn unsafe_read<const M: usize>(&self) -> ClassMutPtr<'a, T, M> {
        assert!(check_ptr::<N>(self.inner.get_raw_obj()));
        assert!(!self.is_null());
        assert!(M + 1 == N);
        assert!(N > 1);
        let pobj = self.inner.get_obj() as *const *const ();
        let obj = ((pobj.read() as usize) | (M - 1)) as *const ();
        unsafe { ClassMutPtr::<'a, T, M>::new(self.inner.make_ref(obj)) }
    }
}

impl<'a, T: AbiClass> ClassMutPtr<'a, T, 1> {
    /// # Safety
    ///
    /// 接口依赖`'a`生命周期时调用，调用者保证在资源实际生命周期长于`'a`.
    ///
    /// # panic
    /// 如果为空指针会panic.
    pub unsafe fn as_deref(&self) -> &'a T {
        assert!(check_ptr::<1>(self.inner.get_raw_obj()));
        assert!(!self.inner.is_null());
        &*(&self.inner as *const T)
    }

    /// # Safety
    ///
    /// 接口依赖`'a`生命周期时调用，调用者保证在资源实际生命周期长于`'a`.
    ///
    /// # panic
    /// 如果为空指针会panic.
    pub unsafe fn as_deref_mut(&mut self) -> &'a mut T {
        assert!(check_ptr::<1>(self.inner.get_raw_obj()));
        assert!(!self.inner.is_null());
        &mut *(&mut self.inner as *mut T)
    }
    /// # Safety
    ///
    /// 调用者保证输入参数`count`的正确性.
    /// # panic
    /// 如果为空指针且count大于0会panic.
    pub unsafe fn into_array(self, count: usize) -> ClassArray<'a, T> {
        assert!(check_ptr::<1>(self.inner.get_raw_obj()));
        assert!(!self.inner.is_null() && count > 0);
        ClassArray::new(self.inner, count)
    }

    /// # Safety
    ///
    /// 调用者保证输入参数`count`的正确性.
    /// # panic
    /// 如果为空指针且count大于0会panic.
    pub unsafe fn into_mut_array(self, count: usize) -> ClassMutArray<'a, T> {
        assert!(check_ptr::<1>(self.inner.get_raw_obj()));
        assert!(!self.inner.is_null() && count > 0);
        ClassMutArray::new(self.inner, count)
    }

    /// # Safety
    ///
    /// `T`, `ClassRef<'_, T>`, `ClassRefMut<'_, T>`, `ClassPtr<'_, T>`, `ClassMutPtr<'_,
    /// T>`作为返回值实质是等价.
    ///
    /// 调用此接口后，和其他资源的生命周期关联关系丢失，调用者负责生命周期管理.
    /// 一般应仅用于参数传递场景.
    ///
    /// # panic
    /// 如果为空指针会panic.
    pub unsafe fn into_value(self) -> T {
        assert!(check_ptr::<1>(self.inner.get_raw_obj()));
        assert!(!self.inner.is_null());
        self.inner
    }

    /// `T`, `ClassRef<'_, T>`, `ClassRefMut<'_, T>`, `ClassPtr<'_, T>`, `ClassMutPtr<'_,
    /// T>`作为返回值实质是等价.
    ///
    /// # panic
    /// 如果为空指针会panic.
    pub fn as_ref(&self) -> ClassRef<'a, T> {
        assert!(check_ptr::<1>(self.inner.get_raw_obj()));
        unsafe { self.as_deref().as_ref() }
    }

    /// `T`, `ClassRef<'_, T>`, `ClassRefMut<'_, T>`, `ClassPtr<'_, T>`, `ClassMutPtr<'_,
    /// T>`作为返回值实质是等价.
    ///
    /// # panic
    /// 如果为空指针会panic.
    pub fn as_mut(&mut self) -> ClassRefMut<'a, T> {
        assert!(check_ptr::<1>(self.inner.get_raw_obj()));
        unsafe { self.as_deref_mut().as_mut() }
    }

    /// `T`, `ClassRef<'_, T>`, `ClassRefMut<'_, T>`, `ClassPtr<'_, T>`, `ClassMutPtr<'_,
    /// T>`作为返回值实质是等价.
    ///
    /// # panic
    /// 如果为空指针会panic.
    pub fn into_ref(self) -> ClassRef<'a, T> {
        assert!(check_ptr::<1>(self.inner.get_raw_obj()));
        assert!(!self.inner.is_null());
        ClassRef::new(self.inner)
    }

    /// `T`, `ClassRef<'_, T>`, `ClassRefMut<'_, T>`, `ClassPtr<'_, T>`, `ClassMutPtr<'_,
    /// T>`作为返回值实质是等价.
    ///
    /// # panic
    /// 如果为空指针会panic.
    pub fn into_mut(self) -> ClassRefMut<'a, T> {
        assert!(check_ptr::<1>(self.inner.get_raw_obj()));
        assert!(!self.inner.is_null());
        ClassRefMut::new(self.inner)
    }
}

impl<T: AbiClass> Deref for ClassMutPtr<'_, T, 1> {
    type Target = T;
    /// # panic
    /// 空指针会导致panic
    fn deref(&self) -> &Self::Target {
        assert!(check_ptr::<1>(self.inner.get_raw_obj()));
        assert!(!self.inner.is_null());
        &self.inner
    }
}

impl<T: AbiClass> DerefMut for ClassMutPtr<'_, T, 1> {
    /// # panic
    /// 空指针会导致panic
    fn deref_mut(&mut self) -> &mut Self::Target {
        assert!(check_ptr::<1>(self.inner.get_raw_obj()));
        assert!(!self.inner.is_null());
        &mut self.inner
    }
}

impl<T: AbiClass + PartialEq> PartialEq for ClassMutPtr<'_, T> {
    fn eq(&self, other: &Self) -> bool {
        PartialEq::eq(&self.inner, &other.inner)
    }
}

impl<T: AbiClass + Eq> Eq for ClassMutPtr<'_, T> {}

impl<T: AbiClass + PartialOrd> PartialOrd for ClassMutPtr<'_, T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        PartialOrd::partial_cmp(&self.inner, &other.inner)
    }
}

impl<T: AbiClass + Ord> Ord for ClassMutPtr<'_, T> {
    fn cmp(&self, other: &Self) -> Ordering {
        Ord::cmp(&self.inner, &other.inner)
    }
}

/// T**: 2重指针
impl<'a, T: AbiClass> ClassMutPtr<'a, T, 2> {
    /// # Safety
    /// 如果为空指针，则会panic
    pub unsafe fn read(&self) -> ClassMutPtr<'a, T, 1> {
        self.unsafe_read::<1>()
    }
}

/// T***: 3重指针
impl<'a, T: AbiClass> ClassMutPtr<'a, T, 3> {
    /// # Safety
    /// 如果为空指针，则会panic
    pub unsafe fn read(&self) -> ClassMutPtr<'a, T, 2> {
        self.unsafe_read::<2>()
    }
}

/// T****: 4重指针
impl<'a, T: AbiClass> ClassMutPtr<'a, T, 4> {
    /// # Safety
    /// 如果为空指针，则会panic
    pub unsafe fn read(&self) -> ClassMutPtr<'a, T, 3> {
        self.unsafe_read::<3>()
    }
}

/// T*****: 5重指针
#[cfg(target_pointer_width = "64")]
impl<'a, T: AbiClass> ClassMutPtr<'a, T, 5> {
    /// # Safety
    ///
    /// 如果为空指针，则会panic
    pub unsafe fn read(&self) -> ClassMutPtr<'a, T, 4> {
        self.unsafe_read::<4>()
    }
}

/// T******: 6重指针
#[cfg(target_pointer_width = "64")]
impl<'a, T: AbiClass> ClassMutPtr<'a, T, 6> {
    /// # Safety
    ///
    /// 如果为空指针，则会panic
    pub unsafe fn read(&self) -> ClassMutPtr<'a, T, 5> {
        self.unsafe_read::<5>()
    }
}

fn check_ptr<const N: usize>(obj: *const ()) -> bool {
    let n = ((obj as usize) & (size_of_val(&obj) - 1)) + 1;
    n == N
}
