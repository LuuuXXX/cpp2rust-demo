use hicc::{AbiType, ClassMutPtr};
use std::iter::Iterator;

hicc::cpp! {
    #include <list>
}

hicc::import_class! {
    #[cpp(class = "template<class T, class Allocator> std::list<T, Allocator>")]
    pub class list<T> {
        hicc::cpp! {
            typedef typename Self::iterator iterator;
            typedef typename Self::const_iterator const_iterator;
            typedef typename Self::reverse_iterator reverse_iterator;
            typedef typename Self::const_reverse_iterator const_reverse_iterator;
        }
        /// ```
        /// use hicc_std::ListInt;
        /// let list = ListInt::new();
        /// assert!(list.is_empty());
        /// ```
        #[cpp(method = "bool empty() const")]
        pub fn is_empty(&self) -> bool;

        /// ```
        /// use hicc_std::ListInt;
        /// let list = ListInt::new();
        /// assert_eq!(list.size(), 0);
        /// ```
        #[cpp(method = "size_t size() const")]
        pub fn size(&self) -> usize;

        /// ```
        /// use hicc_std::ListInt;
        /// let list = ListInt::new();
        /// assert!(list.max_size() >= list.size());
        /// ```
        #[cpp(method = "size_t max_size() const")]
        pub fn max_size(&self) -> usize;

        /// ```
        /// use hicc_std::ListInt;
        /// let mut list = ListInt::new();
        /// list.push_back(&1);
        /// list.clear();
        /// assert!(list.is_empty());
        /// ```
        #[cpp(method = "void clear()")]
        pub fn clear(&mut self);

        /// ```
        /// use hicc_std::ListInt;
        /// let mut list = ListInt::new();
        /// list.resize(10, &1);
        /// assert_eq!(list.size(), 10);
        /// list.iter().for_each(|v| { assert_eq!(v, &1); });
        /// ```
        #[cpp(method = "void resize(size_t, const T&)")]
        pub fn resize(&mut self, n: usize, val: &T);

        /// 如果为空则忽略.
        /// ```
        /// use hicc_std::ListInt;
        /// let mut list = ListInt::new();
        /// list.push_back(&1);
        /// list.pop_back();
        /// assert!(list.is_empty());
        /// ```
        pub fn pop_back(&mut self) {
            if !self.is_empty() {
                self._pop_back();
            }
        }
        #[cpp(method = "void pop_back()")]
        fn _pop_back(&mut self);

        /// ```
        /// use hicc_std::ListInt;
        /// let mut list = ListInt::new();
        /// list.push_back(&1);
        /// assert_eq!(list.front(), Some(&1));
        /// assert_eq!(list.back(), Some(&1));
        /// ```
        #[cpp(method = "void push_back(const T&)")]
        pub fn push_back(&mut self, val: &T);

        /// 如果为空则忽略.
        /// ```
        /// use hicc_std::ListInt;
        /// let mut list = ListInt::new();
        /// list.push_back(&1);
        /// list.pop_front();
        /// assert!(list.is_empty());
        /// ```
        pub fn pop_front(&mut self) {
            if !self.is_empty() {
                self._pop_front();
            }
        }
        #[cpp(method = "void pop_front()")]
        fn _pop_front(&mut self);

        /// ```
        /// use hicc_std::ListInt;
        /// let mut list = ListInt::new();
        /// list.push_front(&1);
        /// assert_eq!(list.front(), Some(&1));
        /// assert_eq!(list.back(), Some(&1));
        /// ```
        #[cpp(method = "void push_front(const T&)")]
        pub fn push_front(&mut self, val: &T);

        /// ```
        /// use hicc_std::ListInt;
        /// let mut list = ListInt::new();
        /// list.push_back(&1);
        /// list.swap(&mut ListInt::new());
        /// assert!(list.is_empty());
        /// ```
        #[cpp(method = "void swap(Self&)")]
        pub fn swap(&mut self, other: &mut Self);

        /// ```
        /// use hicc_std::ListInt;
        /// let mut list = ListInt::new();
        /// list.assign(10, &1);
        /// assert_eq!(list.size(), 10);
        /// list.iter().for_each(|v| assert_eq!(v, &1));
        /// ```
        #[cpp(method = "void assign(size_t, const T&)")]
        pub fn assign(&mut self, ncount: usize, val: &T);

        /// ```
        /// use hicc_std::ListInt;
        /// let mut list = ListInt::new();
        /// list.push_back(&1);
        /// list.push_back(&2);
        /// assert_eq!(list.front(), Some(&1));
        /// assert_eq!(list.back(), Some(&2));
        /// list.reverse();
        /// assert_eq!(list.front(), Some(&2));
        /// assert_eq!(list.back(), Some(&1));
        /// ```
        #[cpp(method = "void reverse()")]
        pub fn reverse(&mut self);

        /// ```
        /// use hicc_std::ListInt;
        /// let mut list = ListInt::new();
        /// assert_eq!(list.back(), None);
        /// list.push_back(&1);
        /// assert_eq!(list.back(), Some(&1));
        /// ```
        pub fn back(&self) -> Option<T::OutputRef<'_>> {
            if !self.is_empty() {
                return Some(unsafe { self._back() });
            }
            None
        }
        #[cpp(method = "const T& back() const")]
        unsafe fn _back(&self) -> &T;

        /// ```
        /// use hicc_std::ListInt;
        /// let mut list = ListInt::new();
        /// assert_eq!(list.back_mut(), None);
        /// list.push_back(&1);
        /// *list.back_mut().unwrap() = 2;
        /// assert_eq!(list.back(), Some(&2));
        ///
        /// use hicc_std::{string, ListString};
        /// use hicc::AbiClass;
        /// let mut list = ListString::new();
        /// list.push_back(&string::from(c"hello"));
        /// list.back_mut().unwrap().write(string::from(c"world"));
        /// assert_eq!(list.back(), Some(string::from(c"world").into_ref()));
        /// ```
        pub fn back_mut(&mut self) -> Option<T::OutputRefMut<'_>> {
            if !self.is_empty() {
                return Some(unsafe { self._back_mut() });
            }
            None
        }
        #[cpp(method = "T& back()")]
        unsafe fn _back_mut(&mut self) -> &mut T;

        /// ```
        /// use hicc_std::ListInt;
        /// let mut list = ListInt::new();
        /// list.push_back(&1);
        /// assert_eq!(list.back(), Some(&1));
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
        /// use hicc_std::ListInt;
        /// let mut list = ListInt::new();
        /// list.push_back(&1);
        /// *list.front_mut().unwrap() = 2;
        /// assert_eq!(list.front(), Some(&2));
        ///
        /// use hicc_std::{string, ListString};
        /// use hicc::AbiClass;
        /// let mut list = ListString::new();
        /// list.push_back(&string::from(c"hello"));
        /// list.front_mut().unwrap().write(string::from(c"world"));
        /// assert_eq!(list.front(), Some(string::from(c"world").into_ref()));
        /// ```
        pub fn front_mut(&mut self) -> Option<T::OutputRefMut<'_>> {
            if !self.is_empty() {
                return Some(unsafe { self._front_mut() });
            }
            None
        }
        #[cpp(method = "T& front()")]
        unsafe fn _front_mut(&mut self) -> &mut T;

        /// ```
        /// use hicc_std::ListInt;
        /// let mut list = ListInt::new();
        /// list.push_back(&1);
        /// list.push_back(&2);
        /// list.push_back(&1);
        /// list.remove(&1);
        /// assert_eq!(list.size(), 1);
        /// assert_eq!(list.front(), Some(&2));
        /// ```
        #[cpp(method = "void remove(const T&)")]
        pub fn remove(&mut self, val: &T);

        hicc::cpp! {
            static void remove_if(Self& self, std::function<bool(const T&)> comp) {
                self.remove_if(comp);
            }
        }
        /// ```
        /// use hicc_std::ListInt;
        /// let mut list = ListInt::new();
        /// list.push_back(&1);
        /// list.push_back(&2);
        /// list.push_back(&3);
        /// list.remove_if(|v: &i32| -> bool {
        ///     (v & 1) == 1
        ///     }.into());
        /// assert_eq!(list.size(), 1);
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
        /// use hicc_std::ListInt;
        /// let mut list = ListInt::new();
        /// list.push_back(&3);
        /// list.push_back(&2);
        /// list.push_back(&1);
        /// list.sort(|v1: &i32, v2: &i32| -> bool {
        ///     v1 < v2
        ///     }.into());
        /// let mut it = list.iter();
        /// assert_eq!(it.next(), Some(&1));
        /// assert_eq!(it.next(), Some(&2));
        /// assert_eq!(it.next(), Some(&3));
        /// assert_eq!(it.next(), None);
        /// ```
        //#[cpp(method = "void sort<std::function<bool(const T&, const T&)>>(std::function<bool(const T&, const T&)>)")]
        #[cpp(func = "void SelfMethods::sort(Self&, std::function<bool(const T&, const T&)>)")]
        pub fn sort<'a>(&'a mut self, comp: hicc::Function<fn(&'a T::InputType, &'a T::InputType) -> bool>);

        hicc::cpp! {
            static void merge(Self& self, Self& other, std::function<bool(const T&, const T&)> comp) {
                if (self.get_allocator() == other.get_allocator()) {
                    self.merge(other, comp);
                }
            }
        }
        /// 如果不满足如下条件，则忽略
        /// 1. `self.get_allocator() == other.get_allocator()`.
        /// ```
        /// use hicc_std::ListInt;
        /// let mut list = ListInt::new();
        /// list.push_back(&3);
        /// list.push_back(&1);
        /// let cmp = |v1: &i32, v2: &i32| -> bool {
        ///     v1 < v2
        ///     };
        /// list.sort(cmp.clone().into());
        /// let mut other = ListInt::new();
        /// other.push_back(&4);
        /// other.push_back(&2);
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
        /// use hicc_std::ListInt;
        /// let mut list = ListInt::new();
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
        /// use hicc_std::ListInt;
        /// let mut list = ListInt::new();
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
        unsafe fn begin(&self) -> *mut CppListIter<T>;
        #[cpp(method = "const_iterator end() const")]
        unsafe fn end(&self) -> *mut CppListIter<T>;

        // 需要同时调用begin_mut和end_mut, 只能返回指针，否则破坏引用规则.
        #[cpp(method = "iterator begin()")]
        unsafe fn begin_mut(&mut self) -> *mut CppListIterMut<T>;
        #[cpp(method = "iterator end()")]
        unsafe fn end_mut(&mut self) -> *mut CppListIterMut<T>;

        #[cpp(method = "const_reverse_iterator rbegin() const")]
        unsafe fn rbegin(&self) -> *mut CppListRevIter<T>;
        #[cpp(method = "const_reverse_iterator rend() const")]
        unsafe fn rend(&self) -> *mut CppListRevIter<T>;

        // 需要同时调用rbegin_mut和rend_mut, 只能返回指针，否则破坏引用规则.
        #[cpp(method = "reverse_iterator rbegin()")]
        unsafe fn rbegin_mut(&mut self) -> *mut CppListRevIterMut<T>;
        #[cpp(method = "reverse_iterator rend()")]
        unsafe fn rend_mut(&mut self) -> *mut CppListRevIterMut<T>;

        hicc::cpp! {
            static void insert(Self& self, iterator& pos, size_t ncount, const T& val, iterator& end) {
                pos = self.insert(pos, ncount, val);
                end = self.end();
            }
        }
        #[cpp(func = "void SelfMethods::insert(Self&, iterator&, size_t, const T&, iterator&)")]
        unsafe fn insert(&mut self, pos: &mut CppListIterMut<T>, ncount: usize, val: &T, end: &mut CppListIterMut<T>);

        hicc::cpp! {
            static void splice(Self& self, iterator& pos, Self& other, iterator& end) {
                if (&self != &other && self.get_allocator() == other.get_allocator()) {
                    self.splice(pos, other);
                    end = self.end();
                }
            }
        }
        #[cpp(func = "void SelfMethods::splice(Self&, iterator&, Self&, iterator&)")]
        unsafe fn splice(&mut self, pos: &mut CppListIterMut<T>, other: &mut Self, end: &mut CppListIterMut<T>);

        hicc::cpp! {
            static void erase(Self& self, iterator& pos, iterator& end) {
                if (pos != self.end()) {
                    pos = self.erase(pos);
                    end = self.end();
                }
            }
        }
        #[cpp(func = "void SelfMethods::erase(Self&, iterator&, iterator&)")]
        unsafe fn erase(&mut self, pos: &mut CppListIterMut<T>, end: &mut CppListIterMut<T>);
    }

    unsafe impl<T: AbiType + Sync> Send for list<T> {}
    unsafe impl<T: AbiType + Sync> Sync for list<T> {}

    #[cpp(class = "template<class T, class Allocator> std::list<T, Allocator>::const_iterator")]
    class CppListIter<T> {
        hicc::cpp! {
            static const T& next(Self& self) {
                return *self++;
            }
        }
        #[cpp(func = "const T& SelfMethods::next(Self&)")]
        unsafe fn next(&mut self) -> &T;
        #[cpp(func = "bool hicc::make_eq<Self, Self>(const Self&, const Self&)")]
        fn equal(&self, other: &Self) -> bool;
    }

    #[cpp(class = "template<class T, class Allocator> std::list<T, Allocator>::iterator")]
    class CppListIterMut<T> {
        hicc::cpp! {
            static T& next(Self& self) {
                return *self++;
            }
        }
        #[cpp(func = "T& SelfMethods::next(Self&)")]
        unsafe fn next(&mut self) -> &mut T;
        #[cpp(func = "bool hicc::make_eq(const Self&, const Self&)")]
        fn equal(&self, other: &Self) -> bool;
    }

    #[cpp(class = "template<class T, class Allocator> std::list<T, Allocator>::const_reverse_iterator")]
    class CppListRevIter<T> {
        hicc::cpp! {
            static const T& next(Self& self) {
                return *self++;
            }
        }
        #[cpp(func = "const T& SelfMethods::next(Self&)")]
        unsafe fn next(&mut self) -> &T;
        #[cpp(func = "bool hicc::make_eq(const Self&, const Self&)")]
        fn equal(&self, other: &Self) -> bool;
    }

    #[cpp(class = "template<class T, class Allocator> std::list<T, Allocator>::reverse_iterator")]
    class CppListRevIterMut<T> {
        hicc::cpp! {
            static T& next(Self& self) {
                return *self++;
            }
        }
        #[cpp(func = "T& SelfMethods::next(Self&)")]
        unsafe fn next(&mut self) -> &mut T;
        #[cpp(func = "bool hicc::make_eq<Self, Self>(const Self&, const Self&)")]
        fn equal(&self, other: &Self) -> bool;
    }
}

impl<T: AbiType> list<T> {
    /// ```
    /// use hicc_std::ListInt;
    /// let mut list = ListInt::new();
    /// list.push_back(&1);
    /// list.push_back(&1);
    /// assert_eq!(list.iter().count(), 2);
    /// list.iter().for_each(|v| {assert_eq!(v, &1);});
    /// ```
    pub fn iter(&self) -> impl Iterator<Item = T::OutputRef<'_>> {
        ListIter {
            beg: unsafe { self.begin() },
            end: unsafe { self.end() },
        }
    }
    /// ```
    /// use hicc_std::ListInt;
    /// let mut list = ListInt::new();
    /// list.push_back(&1);
    /// list.push_back(&2);
    /// list.iter_mut().for_each(|v| *v -= 1);
    /// assert_eq!(list.front(), Some(&0));
    /// assert_eq!(list.back(), Some(&1));
    /// ```
    pub fn iter_mut(&mut self) -> ListIterMut<'_, T> {
        let beg = unsafe { self.begin_mut() };
        let end = unsafe { self.end_mut() };
        ListIterMut {
            list: self,
            beg,
            end,
        }
    }
    /// ```
    /// use hicc_std::ListInt;
    /// let mut list = ListInt::new();
    /// list.push_back(&1);
    /// list.push_back(&1);
    /// assert_eq!(list.rev_iter().count(), 2);
    /// list.rev_iter().for_each(|v| {assert_eq!(v, &1);});
    /// ```
    pub fn rev_iter(&self) -> impl Iterator<Item = T::OutputRef<'_>> {
        ListRevIter {
            beg: unsafe { self.rbegin() },
            end: unsafe { self.rend() },
        }
    }
    /// ```
    /// use hicc_std::ListInt;
    /// let mut list = ListInt::new();
    /// list.push_back(&1);
    /// list.push_back(&2);
    /// list.rev_iter_mut().for_each(|v| *v += 1);
    /// assert_eq!(list.front(), Some(&2));
    /// assert_eq!(list.back(), Some(&3));
    /// ```
    pub fn rev_iter_mut(&mut self) -> impl Iterator<Item = T::OutputRefMut<'_>> {
        let beg = unsafe { self.rbegin_mut() };
        let end = unsafe { self.rend_mut() };
        ListRevIterMut { beg, end }
    }
}

/// 对应`std::list<T>::const_iterator`
struct ListIter<'a, T: AbiType + 'static> {
    beg: ClassMutPtr<'a, CppListIter<T>>,
    end: ClassMutPtr<'a, CppListIter<T>>,
}

impl<'a, T: AbiType + 'static> Iterator for ListIter<'a, T> {
    type Item = T::OutputRef<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        if !self.beg.equal(&self.end) {
            return Some(unsafe { self.beg.as_deref_mut().next() });
        }
        None
    }
}

/// 对应`std::list<T>::const_reverse_iterator`
struct ListRevIter<'a, T: AbiType + 'static> {
    beg: ClassMutPtr<'a, CppListRevIter<T>>,
    end: ClassMutPtr<'a, CppListRevIter<T>>,
}

impl<'a, T: AbiType + 'static> Iterator for ListRevIter<'a, T> {
    type Item = T::OutputRef<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        if !self.beg.equal(&self.end) {
            return Some(unsafe { self.beg.as_deref_mut().next() });
        }
        None
    }
}

/// 对应`std::list<T>::iterator`
pub struct ListIterMut<'a, T: AbiType + 'static> {
    list: &'a mut list<T>,
    beg: ClassMutPtr<'a, CppListIterMut<T>>,
    end: ClassMutPtr<'a, CppListIterMut<T>>,
}

impl<'a, T: AbiType + 'static> Iterator for ListIterMut<'a, T> {
    type Item = T::OutputRefMut<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        if !self.beg.equal(&self.end) {
            return Some(unsafe { self.beg.as_deref_mut().next() });
        }
        None
    }
}

/// 对应`std::list<T>::reverse_iterator`
struct ListRevIterMut<'a, T: AbiType + 'static> {
    beg: ClassMutPtr<'a, CppListRevIterMut<T>>,
    end: ClassMutPtr<'a, CppListRevIterMut<T>>,
}

impl<'a, T: AbiType + 'static> Iterator for ListRevIterMut<'a, T> {
    type Item = T::OutputRefMut<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        if !self.beg.equal(&self.end) {
            return Some(unsafe { self.beg.as_deref_mut().next() });
        }
        None
    }
}

impl<T: AbiType + 'static> ListIterMut<'_, T> {
    /// 调用`std::list::insert`并将当前节点更新为其返回值.
    /// ```
    /// use hicc_std::ListInt;
    /// let mut list = ListInt::new();
    /// let mut it = list.iter_mut();
    /// it.insert(2, &1);
    /// assert_eq!(it.next(), Some(&mut 1));
    /// assert_eq!(it.next(), Some(&mut 1));
    /// assert_eq!(it.next(), None);
    /// it.insert(2, &2);
    /// assert_eq!(it.next(), Some(&mut 2));
    /// assert_eq!(it.next(), Some(&mut 2));
    /// assert_eq!(it.next(), None);
    /// ```
    pub fn insert(&mut self, ncount: usize, val: &T::InputType) {
        unsafe {
            self.list.insert(&mut self.beg, ncount, val, &mut self.end);
        }
    }

    /// 如果满足以下调用`std::list::splice`.
    /// 1. `self`和`other`不同.
    /// 2. `self.get_allocator() == other.get_allocator`.
    /// ```
    /// use hicc_std::ListInt;
    /// let mut list = ListInt::new();
    /// let mut other = ListInt::new();
    /// other.push_back(&1);
    /// let mut it = list.iter_mut();
    /// it.splice(&mut other);
    /// assert!(other.is_empty());
    /// assert_eq!(it.next(), None);
    /// let mut it = list.iter();
    /// assert_eq!(it.next(), Some(&1));
    /// assert_eq!(it.next(), None);
    /// ```
    pub fn splice(&mut self, other: &mut list<T>) {
        unsafe {
            self.list.splice(&mut self.beg, other, &mut self.end);
        }
    }

    /// 如果当前节点有效调用`std::list::erase`并更新为其返回值.
    /// ```
    /// use hicc_std::ListInt;
    /// let mut list = ListInt::new();
    /// let mut it = list.iter_mut();
    /// it.remove();
    /// it.insert(2, &1);
    /// it.remove();
    /// it.remove();
    /// assert!(list.is_empty());
    /// ```
    pub fn remove(&mut self) {
        unsafe {
            self.list.erase(&mut self.beg, &mut self.end);
        }
    }
}
