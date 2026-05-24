use hicc::{AbiType, ClassMutPtr};
use std::iter::Iterator;

hicc::cpp! {
    #include <forward_list>
}

hicc::import_class! {
    #[cpp(class = "template<class T, class Allocator> std::forward_list<T, Allocator>")]
    pub class forward_list<T> {
        hicc::cpp! {
            typedef typename Self::iterator iterator;
            typedef typename Self::const_iterator const_iterator;
        }
        /// ```
        /// use hicc_std::ForwardListInt;
        /// let list = ForwardListInt::new();
        /// assert!(list.is_empty());
        /// ```
        #[cpp(method = "bool empty() const")]
        pub fn is_empty(&self) -> bool;

        /// ```
        /// use hicc_std::ForwardListInt;
        /// let mut list = ForwardListInt::new();
        /// list.push_front(&1);
        /// assert!(list.max_size() >= 1);
        /// ```
        #[cpp(method = "size_t max_size() const")]
        pub fn max_size(&self) -> usize;

        /// ```
        /// use hicc_std::ForwardListInt;
        /// let mut list = ForwardListInt::new();
        /// list.push_front(&1);
        /// list.clear();
        /// assert!(list.is_empty());
        /// ```
        #[cpp(method = "void clear()")]
        pub fn clear(&mut self);

        /// ```
        /// use hicc_std::ForwardListInt;
        /// let mut list = ForwardListInt::new();
        /// list.resize(10, &1);
        /// assert_eq!(list.iter().count(), 10);
        /// list.iter().for_each(|v| { println!("{v}"); });
        /// ```
        #[cpp(method = "void resize(size_t, const T&)")]
        pub fn resize(&mut self, n: usize, val: &T);

        /// 如果为空则忽略.
        /// ```
        /// use hicc_std::ForwardListInt;
        /// let mut list = ForwardListInt::new();
        /// list.push_front(&1);
        /// list.pop_front();
        /// assert!(list.is_empty());
        /// ```
        pub fn pop_front(&mut self) {
            if !self.is_empty() {
                unsafe { self._pop_front() };
            }
        }
        #[cpp(method = "void pop_front()")]
        unsafe fn _pop_front(&mut self);

        /// ```
        /// use hicc_std::ForwardListInt;
        /// let mut list = ForwardListInt::new();
        /// list.push_front(&1);
        /// assert_eq!(list.front(), Some(&1));
        /// ```
        #[cpp(method = "void push_front(const T&)")]
        pub fn push_front(&mut self, val: &T);

        /// ```
        /// use hicc_std::ForwardListInt;
        /// let mut list = ForwardListInt::new();
        /// list.push_front(&1);
        /// list.swap(&mut ForwardListInt::new());
        /// assert!(list.is_empty());
        /// ```
        #[cpp(method = "void swap(Self&)")]
        pub fn swap(&mut self, other: &mut Self);

        /// ```
        /// use hicc_std::ForwardListInt;
        /// let mut list = ForwardListInt::new();
        /// list.assign(10, &1);
        /// assert_eq!(list.iter().count(), 10);
        /// assert_eq!(list.front(), Some(&1));
        /// list.iter().for_each(|v| { assert_eq!(v, &1); });
        /// ```
        #[cpp(method = "void assign(size_t, const T&)")]
        pub fn assign(&mut self, ncount: usize, val: &T);

        /// ```
        /// use hicc_std::ForwardListInt;
        /// let mut list = ForwardListInt::new();
        /// list.push_front(&2);
        /// list.push_front(&1);
        /// assert_eq!(list.front(), Some(&1));
        /// list.reverse();
        /// assert_eq!(list.front(), Some(&2));
        /// list.iter().for_each(|v| { println!("{v}"); });
        /// ```
        #[cpp(method = "void reverse()")]
        pub fn reverse(&mut self);

        /// ```
        /// use hicc_std::ForwardListInt;
        /// let mut list = ForwardListInt::new();
        /// list.push_front(&1);
        /// assert_eq!(list.front(), Some(&1));
        /// ```
        pub fn front(&self) -> Option<T::OutputRef<'_>> {
            if !self.is_empty() {
                return Some(unsafe { self._front() });
            }
            None
        }

        #[cpp(method = "const T& front() const")]
        unsafe fn _front(&self) -> &T;

        /// ```
        /// use hicc_std::ForwardListInt;
        /// let mut list = ForwardListInt::new();
        /// list.push_front(&1);
        /// *list.front_mut().unwrap() = 2;
        /// assert_eq!(list.front(), Some(&2));
        ///
        /// use hicc_std::{string, ForwardListString};
        /// use hicc::AbiClass;
        /// let mut list = ForwardListString::new();
        /// list.push_front(&string::from(c"hello"));
        /// list.front_mut().unwrap().write(string::from(c"world"));
        /// assert_eq!(list.front() , Some(string::from(c"world").into_ref()));
        /// ```
        pub fn front_mut(&mut self) -> Option<T::OutputRefMut<'_>> {
            if !self.is_empty() {
                return Some(unsafe { self._front_mut() });
            }
            None
        }
        #[cpp(method = "const T& front() const")]
        unsafe fn _front_mut(&mut self) -> &mut T;


        /// ```
        /// use hicc_std::ForwardListInt;
        /// let mut list = ForwardListInt::new();
        /// list.push_front(&1);
        /// list.push_front(&2);
        /// list.push_front(&1);
        /// list.remove(&1);
        /// assert_eq!(list.iter().count(), 1);
        /// assert_eq!(list.front(), Some(&2));
        /// ```
        #[cpp(method = "void remove(const T&)")]
        pub fn remove(&mut self, val: &T);


        hicc::cpp! {
            static void remove_if(Self& self, std::function<bool(const T&)> pred) {
                self.remove_if(pred);
            }
        }
        /// ```
        /// use hicc_std::ForwardListInt;
        /// let mut list = ForwardListInt::new();
        /// list.push_front(&1);
        /// list.push_front(&2);
        /// list.push_front(&3);
        /// list.remove_if(|v: &i32| -> bool {
        ///     (v & 1) == 1
        ///     }.into());
        /// assert_eq!(list.iter().count(), 1);
        /// assert_eq!(list.front(), Some(&2));
        /// ```
        #[cpp(func = "void SelfMethods::remove_if(Self&, std::function<bool(const T&)>)")]
        pub fn remove_if<'a>(&'a mut self, pred: hicc::Function<fn(&'a T::InputType) -> bool>);

        hicc::cpp! {
            static void sort(Self& self, std::function<bool(const T&, const T&)> comp) {
                self.sort(comp);
            }
        }
        /// ```
        /// use hicc_std::ForwardListInt;
        /// let mut list = ForwardListInt::new();
        /// list.push_front(&1);
        /// list.push_front(&2);
        /// list.push_front(&3);
        /// assert_eq!(list.front(), Some(&3));
        /// list.sort(|v1: &i32, v2: &i32| -> bool {
        ///     v1 < v2
        ///     }.into());
        /// let mut it = list.iter();
        /// assert_eq!(it.next(), Some(&1));
        /// assert_eq!(it.next(), Some(&2));
        /// assert_eq!(it.next(), Some(&3));
        /// assert_eq!(it.next(), None);
        /// ```
        #[cpp(func = "void SelfMethods::sort(Self&, std::function<bool(const T&, const T&)>)")]
        pub fn sort<'a>(&'a mut self, comp: hicc::Function<fn(&'a T::InputType, &'a T::InputType) -> bool>);

        hicc::cpp! {
            static void merge(Self& self, Self& other, std::function<bool(const T&, const T&)> comp) {
                if (self.get_allocator() == other.get_allocator()) {
                    self.merge(other, comp);
                }
            }
        }
        /// 如果不满足如下条件则忽略.
        /// 1. `self.get_allocator() == other.get_allocator()`.
        /// ```
        /// use hicc_std::ForwardListInt;
        /// let mut list = ForwardListInt::new();
        /// list.push_front(&1);
        /// list.push_front(&3);
        /// let cmp = |v1: &i32, v2: &i32| -> bool {
        ///     v1 < v2
        ///     };
        /// list.sort(cmp.clone().into());
        /// let mut other = ForwardListInt::new();
        /// other.push_front(&2);
        /// other.push_front(&4);
        /// other.sort(cmp.clone().into());
        /// list.merge(&mut other, cmp.into());
        /// let mut it = list.iter();
        /// assert!(other.is_empty());
        /// assert_eq!(it.next(), Some(&1));
        /// assert_eq!(it.next(), Some(&2));
        /// assert_eq!(it.next(), Some(&3));
        /// assert_eq!(it.next(), Some(&4));
        /// assert_eq!(it.next(), None);
        /// ```
        #[cpp(func = "void SelfMethods::merge(Self&, Self&, std::function<bool(const T&, const T&)>)")]
        pub fn merge<'a>(&'a mut self, other: &mut Self, comp: hicc::Function<fn(&'a T::InputType, &'a T::InputType) -> bool>);


        hicc::cpp! {
            static void unique(Self& self, std::function<bool(const T&, const T&)> comp) {
                self.unique(comp);
            }
        }
        /// ```
        /// use hicc_std::ForwardListInt;
        /// let mut list = ForwardListInt::new();
        /// list.push_front(&3);
        /// list.push_front(&3);
        /// list.push_front(&1);
        /// list.push_front(&1);
        /// let mut n = 0;
        /// list.unique_with(|v1: &i32, v2: &i32| -> bool {
        ///     v1 == v2
        /// }.into());
        /// let mut it = list.iter();
        /// assert_eq!(it.next(), Some(&1));
        /// assert_eq!(it.next(), Some(&3));
        /// assert_eq!(it.next(), None);
        /// ```
        #[cpp(func = "void SelfMethods::unique(Self&, std::function<bool(const T&, const T&)>)")]
        pub fn unique_with<'a>(&'a mut self, comp: hicc::Function<fn(&'a T::InputType, &'a T::InputType) -> bool>);

        /// ```
        /// use hicc_std::ForwardListInt;
        /// let mut list = ForwardListInt::new();
        /// list.push_front(&3);
        /// list.push_front(&3);
        /// list.push_front(&1);
        /// list.push_front(&1);
        /// list.unique();
        /// let mut it = list.iter();
        /// assert_eq!(it.next(), Some(&1));
        /// assert_eq!(it.next(), Some(&3));
        /// assert_eq!(it.next(), None);
        /// ```
        #[cpp(method = "void unique()")]
        pub fn unique(&mut self);

        #[cpp(method = "const_iterator begin() const")]
        unsafe fn begin(&self) -> *mut CppForwardListIter<T>;
        #[cpp(method = "const_iterator end() const")]
        unsafe fn end(&self) -> *mut CppForwardListIter<T>;
        #[cpp(method = "iterator begin()")]
        unsafe fn begin_mut(&mut self) -> *mut CppForwardListIterMut<T>;
        #[cpp(method = "iterator end()")]
        unsafe fn end_mut(&mut self) -> *mut CppForwardListIterMut<T>;

        hicc::cpp! {
            static void insert_after(Self& self, iterator& pos, size_t count, const T& val) {
                if (pos != self.end()) {
                    self.insert_after(pos, count, val);
                }
            }
            static void insert_after(Self& self, size_t count, const T& val) {
                    self.insert_after(self.before_begin(), count, val);
            }
        }
        #[cpp(func = "void insert_after(Self&, iterator&, size_t, const T&)")]
        unsafe fn insert_after(&mut self, pos: &mut CppForwardListIterMut<T>, ncount: usize, val: &T);
        /// ```
        /// use hicc_std::ForwardListInt;
        /// let mut list = ForwardListInt::new();
        /// list.insert_front(2, &1);
        /// let mut it = list.iter();
        /// assert_eq!(it.next(), Some(&1));
        /// assert_eq!(it.next(), Some(&1));
        /// assert_eq!(it.next(), None);
        /// ```
        #[cpp(func = "void insert_after(Self&, size_t, const T&)")]
        pub fn insert_front(&mut self, ncount: usize, val: &T);


        hicc::cpp! {
            static void splice_after(Self& self, iterator& pos, Self& other) {
                if (pos != self.end() && self.get_allocator() == other.get_allocator() && &self != &other) {
                    self.splice_after(pos, other);
                }
            }
            static void splice_after(Self& self, Self& other) {
                if (self.get_allocator() == other.get_allocator() && &self != &other) {
                    self.splice_after(self.before_begin(), other);
                }
            }
        }
        #[cpp(func = "void splice_after(Self&, iterator&, Self&)")]
        unsafe fn splice_after(&mut self, pos: &mut CppForwardListIterMut<T>, other: &mut Self);
        /// 不满足下面条件则不做任何修改:
        /// 1. `self.get_allocator() == other.get_allocator()`
        /// 2. `&self != &other`
        /// ```
        /// use hicc_std::ForwardListInt;
        /// let mut src = ForwardListInt::new();
        /// src.push_front(&1);
        /// src.push_front(&2);
        /// let mut dst = ForwardListInt::new();
        /// dst.splice_front(&mut src);
        /// assert!(src.is_empty());
        /// let mut it = dst.iter();
        /// assert_eq!(it.next(), Some(&2));
        /// assert_eq!(it.next(), Some(&1));
        /// assert_eq!(it.next(), None);
        /// ```
        #[cpp(func = "void splice_after(Self&, Self&)")]
        pub fn splice_front(&mut self, other: &mut Self);

        hicc::cpp! {
            static void erase_after(Self& self, iterator& pos) {
                const_iterator it = pos;
                if (it != self.end() && ++it != self.end()) {
                    self.erase_after(pos);
                }
            }
        }
        #[cpp(func = "void erase_after(Self&, iterator&)")]
        unsafe fn erase_after(&mut self, pos: &mut CppForwardListIterMut<T>);
    }

    unsafe impl<T: AbiType + Sync> Send for forward_list<T> {}
    unsafe impl<T: AbiType + Sync> Sync for forward_list<T> {}

    #[cpp(class = "template<class T, class Allocator> std::forward_list<T, Allocator>::const_iterator")]
    class CppForwardListIter<T> {
        hicc::cpp! {
            static const T& next(Self& self) {
                return *self++;
            }
            static const T& get(const Self& self) {
                return *self;
            }
        }
        #[cpp(func = "const T& SelfMethods::next(Self&)")]
        unsafe fn next(&mut self) -> &T;
        #[cpp(func = "const T& SelfMethods::get(const Self&)")]
        unsafe fn get(&self) -> &T;
        #[cpp(func = "bool hicc::make_eq<Self, Self>(const Self&, const Self&)")]
        fn equal(&self, other: &Self) -> bool;
    }

    #[cpp(class = "template<class T, class Allocator> std::forward_list<T, Allocator>::iterator")]
    class CppForwardListIterMut<T> {
        hicc::cpp! {
            static T& next(Self& self) {
                return *self++;
            }
            static const T& get(const Self& self) {
                return *self;
            }
            static bool has_next(const Self& self, const Self& end) {
                return self != end && ++Self(self) != end;
            }
        }
        #[cpp(func = "T& SelfMethods::next(Self&)")]
        unsafe fn next(&mut self) -> &mut T;
        #[cpp(func = "const T& SelfMethods::get(const Self&)")]
        unsafe fn get(&self) -> &T;
        #[cpp(func = "const T& SelfMethods::get(const Self&)")]
        unsafe fn get_mut(&mut self) -> &mut T;
        #[cpp(func = "bool hicc::make_eq<Self, Self>(const Self&, const Self&)")]
        fn equal(&self, other: &Self) -> bool;
        #[cpp(func = "bool SelfMethods::has_next(const Self&, const Self&)")]
        fn has_next(&self, other: &Self) -> bool;
    }
}

impl<T: AbiType> forward_list<T> {
    /// ```
    /// use hicc_std::ForwardListInt;
    /// let mut list = ForwardListInt::new();
    /// list.push_front(&1);
    /// list.push_front(&2);
    /// let mut it = list.iter();
    /// assert_eq!(it.next(), Some(&2));
    /// assert_eq!(it.next(), Some(&1));
    /// assert_eq!(it.next(), None);
    /// ```
    pub fn iter(&self) -> ForwardListIter<'_, T> {
        ForwardListIter {
            beg: unsafe { self.begin() },
            end: unsafe { self.end() },
        }
    }
    /// ```
    /// use hicc_std::ForwardListInt;
    /// let mut list = ForwardListInt::new();
    /// list.push_front(&1);
    /// list.iter_mut().for_each(|v| *v -= 1);
    /// assert_eq!(list.front(), Some(&0));
    /// ```
    pub fn iter_mut(&mut self) -> ForwardListIterMut<'_, T> {
        let beg = unsafe { self.begin_mut() };
        let end = unsafe { self.end_mut() };
        ForwardListIterMut {
            list: self,
            beg,
            end,
        }
    }
}

/// 对应`std::forward_list<T>::const_iterator`
pub struct ForwardListIter<'a, T: AbiType + 'static> {
    beg: ClassMutPtr<'a, CppForwardListIter<T>>,
    end: ClassMutPtr<'a, CppForwardListIter<T>>,
}

impl<'a, T: AbiType + 'static> Iterator for ForwardListIter<'a, T> {
    type Item = T::OutputRef<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        if !self.beg.equal(&self.end) {
            return Some(unsafe { self.beg.as_deref_mut().next() });
        }
        None
    }
}

impl<T: AbiType + 'static> ForwardListIter<'_, T> {
    /// ```
    /// use hicc_std::ForwardListInt;
    /// let mut list = ForwardListInt::new();
    /// let it = list.iter();
    /// assert_eq!(it.get(), None);
    /// list.push_front(&1);
    /// let mut it = list.iter();
    /// assert_eq!(it.get(), Some(&1));
    /// assert_eq!(it.next(), Some(&1));
    /// assert_eq!(it.get(), None);
    /// assert_eq!(it.next(), None);
    /// ```
    pub fn get(&self) -> Option<T::OutputRef<'_>> {
        if !self.beg.equal(&self.end) {
            return Some(unsafe { self.beg.get() });
        }
        None
    }
}

/// 对应`std::forward_list<T>::iterator`
pub struct ForwardListIterMut<'a, T: AbiType + 'static> {
    list: &'a mut forward_list<T>,
    beg: ClassMutPtr<'a, CppForwardListIterMut<T>>,
    end: ClassMutPtr<'a, CppForwardListIterMut<T>>,
}

impl<'a, T: AbiType + 'static> Iterator for ForwardListIterMut<'a, T> {
    type Item = T::OutputRefMut<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        if !self.beg.equal(&self.end) {
            return Some(unsafe { self.beg.as_deref_mut().next() });
        }
        None
    }
}

impl<T: AbiType + 'static> ForwardListIterMut<'_, T> {
    /// ```
    /// use hicc_std::ForwardListInt;
    /// let mut list = ForwardListInt::new();
    /// let it = list.iter_mut();
    /// assert_eq!(it.get(), None);
    /// list.push_front(&1);
    /// let mut it = list.iter_mut();
    /// assert_eq!(it.get(), Some(&1));
    /// assert_eq!(it.next(), Some(&mut 1));
    /// assert_eq!(it.get(), None);
    /// assert_eq!(it.next(), None);
    /// ```
    pub fn get(&self) -> Option<T::OutputRef<'_>> {
        if !self.beg.equal(&self.end) {
            return Some(unsafe { self.beg.get() });
        }
        None
    }

    /// ```
    /// use hicc_std::ForwardListInt;
    /// let mut list = ForwardListInt::new();
    /// let mut it = list.iter_mut();
    /// assert_eq!(it.get_mut(), None);
    /// list.push_front(&1);
    /// let mut it = list.iter_mut();
    /// assert_eq!(it.get_mut(), Some(&mut 1));
    /// assert_eq!(it.next(), Some(&mut 1));
    /// assert_eq!(it.get(), None);
    /// assert_eq!(it.next(), None);
    /// ```
    pub fn get_mut(&mut self) -> Option<T::OutputRefMut<'_>> {
        if !self.beg.equal(&self.end) {
            return Some(unsafe { self.beg.get_mut() });
        }
        None
    }

    /// 当前是否是最后一个节点.只有`has_next`返回`true`, `erase_after`操作才会成功.
    /// ```
    /// use hicc_std::ForwardListInt;
    /// let mut list = ForwardListInt::new();
    /// let it = list.iter_mut();
    /// assert_eq!(it.has_next(), false);
    /// list.insert_front(2, &1);
    /// let mut it = list.iter_mut();
    /// assert_eq!(it.has_next(), true);
    /// assert_eq!(it.next(), Some(&mut 1));
    /// assert_eq!(it.has_next(), false);
    /// assert_eq!(it.next(), Some(&mut 1));
    /// assert_eq!(it.next(), None);
    /// ```
    pub fn has_next(&self) -> bool {
        self.beg.has_next(&self.end)
    }
    /// 如果当前节点已经是`std::forwarod_list::end()`则无效果. 可调用`insert_front`.
    /// ```
    /// use hicc_std::ForwardListInt;
    /// let mut list = ForwardListInt::new();
    /// list.push_front(&1);
    /// let mut it = list.iter_mut();
    /// it.insert_after(1, &2);
    /// assert_eq!(it.next(), Some(&mut 1));
    /// assert_eq!(it.next(), Some(&mut 2));
    /// assert_eq!(it.next(), None);
    /// assert_eq!(list.front(), Some(&1));
    /// ```
    pub fn insert_after(&mut self, ncount: usize, val: &T::InputType) {
        unsafe { self.list.insert_after(&mut self.beg, ncount, val) };
    }

    /// 不满足下面条件则不做任何修改:
    /// 1. `self.get_allocator() == other.get_allocator()`
    /// 1. `self.has_next() == true`
    /// ```
    /// use hicc_std::ForwardListInt;
    /// let mut list = ForwardListInt::new();
    /// let mut other = ForwardListInt::new();
    /// other.push_front(&2);
    /// list.push_front(&1);
    /// let mut it = list.iter_mut();
    /// it.splice_after(&mut other);
    /// assert!(other.is_empty());
    /// assert_eq!(it.next(), Some(&mut 1));
    /// assert_eq!(it.next(), Some(&mut 2));
    /// assert_eq!(it.next(), None);
    /// assert_eq!(list.front(), Some(&1));
    /// ```
    pub fn splice_after(&mut self, other: &mut forward_list<T>) {
        unsafe { self.list.splice_after(&mut self.beg, other) };
    }

    /// 无后续节点也可以安全调用.
    /// ```
    /// use hicc_std::ForwardListInt;
    /// let mut list = ForwardListInt::new();
    /// let mut it = list.iter_mut();
    /// it.erase_after();
    /// list.push_front(&1);
    /// list.push_front(&2);
    /// list.push_front(&3);
    /// let mut it = list.iter_mut();
    /// it.erase_after();
    /// it.erase_after();
    /// it.erase_after();
    /// assert_eq!(it.next(), Some(&mut 3));
    /// assert_eq!(it.next(), None);
    /// ```
    pub fn erase_after(&mut self) {
        unsafe { self.list.erase_after(&mut self.beg) };
    }
}
