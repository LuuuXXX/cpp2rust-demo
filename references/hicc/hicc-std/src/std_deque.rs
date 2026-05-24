use hicc::AbiType;
use std::iter::Iterator;
use std::marker::PhantomData;

hicc::cpp! {
    #include <deque>
    #include <algorithm>
}

hicc::import_class! {
    #[cpp(class = "template<class T, class Allocator> std::deque<T, Allocator>")]
    pub class deque<T> {
        /// ```
        /// use hicc_std::DequeInt;
        /// let deque = DequeInt::new();
        /// assert!(deque.is_empty());
        /// ```
        #[cpp(method = "bool empty() const")]
        pub fn is_empty(&self) -> bool;

        /// ```
        /// use hicc_std::DequeInt;
        /// let deque = DequeInt::new();
        /// assert_eq!(deque.size(), 0);
        /// ```
        #[cpp(method = "size_t size() const")]
        pub fn size(&self) -> usize;

        /// ```
        /// use hicc_std::DequeInt;
        /// let deque = DequeInt::new();
        /// println!("deque.max_size = {}", deque.max_size());
        /// ```
        #[cpp(method = "size_t max_size() const")]
        pub fn max_size(&self) -> usize;

        /// ```
        /// use hicc_std::DequeInt;
        /// let mut deque = DequeInt::new();
        /// deque.push_back(&1);
        /// deque.clear();
        /// assert!(deque.is_empty());
        /// ```
        #[cpp(method = "void clear()")]
        pub fn clear(&mut self);

        /// ```
        /// use hicc_std::DequeInt;
        /// let mut deque = DequeInt::new();
        /// deque.resize(10, &1);
        /// deque.push_front(&2);
        /// // ...
        /// deque.shrink_to_fit();
        /// ```
        #[cpp(method = "void shrink_to_fit()")]
        pub fn shrink_to_fit(&mut self);

        /// ```
        /// use hicc_std::DequeInt;
        /// let mut deque = DequeInt::new();
        /// deque.resize(10, &1);
        /// assert_eq!(deque.size(), 10);
        /// assert_eq!(deque.back(), Some(&1));
        /// assert_eq!(deque.front(), Some(&1));
        /// assert_eq!(deque.get(5), Some(&1));
        /// ```
        #[cpp(method = "void resize(size_t, const T&)")]
        pub fn resize(&mut self, n: usize, val: &T);

        /// ```
        /// use hicc_std::DequeInt;
        /// let mut deque = DequeInt::new();
        /// assert_eq!(deque.get(0), None);
        /// deque.push_front(&1);
        /// assert_eq!(deque.get(0), Some(&1));
        /// ```
        pub fn get(&self, pos: usize) -> Option<T::OutputRef<'_>> {
            if pos < self.size() {
                return unsafe { Some(self._at(pos)) };
            }
            None
        }
        #[cpp(method = "const T& at(size_t) const")]
        unsafe fn _at(&self, pos: usize) -> &T;

        /// ```
        /// use hicc_std::DequeInt;
        /// let mut deque = DequeInt::new();
        /// assert_eq!(deque.get(0), None);
        /// deque.push_front(&1);
        /// *deque.get_mut(0).unwrap() += 1;
        /// assert_eq!(deque.get(0), Some(&2));
        /// ```
        pub fn get_mut(&mut self, pos: usize) -> Option<T::OutputRefMut<'_>> {
            if pos < self.size() {
                return unsafe { Some(self._at_mut(pos)) };
            }
            None
        }
        #[cpp(method = "T& at(size_t)")]
        unsafe fn _at_mut(&mut self, pos: usize) -> &mut T;

        /// ```
        /// use hicc_std::DequeInt;
        /// let mut deque = DequeInt::new();
        /// assert_eq!(deque.back(), None);
        /// deque.push_back(&1);
        /// assert_eq!(deque.back(), Some(&1));
        /// ```
        pub fn back(&self) -> Option<T::OutputRef<'_>> {
            if !self.is_empty() {
                return unsafe { Some(self._back()) };
            }
            None
        }
        #[cpp(method = "const T& back() const")]
        unsafe fn _back(&self) -> &T;

        /// 1. 如果是`hicc::Pod<T>`，可通过`*back_mut() = <new value>`改为.
        /// 1. 如果是`c++ class`, 则必须通过`hicc::AbiClass::write`改写.
        /// ```
        /// use hicc_std::DequeInt;
        /// let mut deque = DequeInt::new();
        /// assert_eq!(deque.back_mut(), None);
        /// deque.push_back(&1);
        /// *deque.back_mut().unwrap() += 1;
        /// assert_eq!(deque.back(), Some(&2));
        ///
        /// use hicc_std::{string, DequeString};
        /// use hicc::AbiClass;
        /// let mut deque = DequeString::new();
        /// deque.push_back(&string::from(c"hello"));
        /// assert_eq!(deque.back(), Some(string::from(c"hello").into_ref()));
        /// deque.back_mut().unwrap().write(string::from(c"world"));
        /// assert_eq!(deque.back(), Some(string::from(c"world").into_ref()));
        /// ```
        pub fn back_mut(&mut self) -> Option<T::OutputRefMut<'_>> {
            if !self.is_empty() {
                return unsafe { Some(self._back_mut()) };
            }
            None
        }
        #[cpp(method = "T& back()")]
        unsafe fn _back_mut(&mut self) -> &mut T;

        /// ```
        /// use hicc_std::DequeInt;
        /// let mut deque = DequeInt::new();
        /// assert_eq!(deque.front(), None);
        /// deque.push_back(&1);
        /// assert_eq!(deque.front(), Some(&1));
        /// ```
        pub fn front(&self) -> Option<T::OutputRef<'_>> {
            if !self.is_empty() {
                return unsafe { Some(self._front()) };
            }
            None
        }
        #[cpp(method = "const T& front() const")]
        unsafe fn _front(&self) -> &T;

        /// 1. 如果是`hicc::Pod<T>`，可通过`*back_mut() = <new value>`改为.
        /// 1. 如果是`c++ class`, 则必须通过`hicc::AbiClass::write`改写.
        /// ```
        /// use hicc_std::DequeInt;
        /// let mut deque = DequeInt::new();
        /// assert_eq!(deque.front_mut(), None);
        /// deque.push_back(&1);
        /// *deque.front_mut().unwrap() += 1;
        /// assert_eq!(deque.front_mut(), Some(&mut 2));
        ///
        /// use hicc_std::{string, DequeString};
        /// use hicc::AbiClass;
        /// let mut deque = DequeString::new();
        /// deque.push_back(&string::from(c"hello"));
        /// assert_eq!(deque.front(), Some(string::from(c"hello").into_ref()));
        /// deque.front_mut().unwrap().write(string::from(c"world"));
        /// assert_eq!(deque.front(), Some(string::from(c"world").into_ref()));
        /// ```
        pub fn front_mut(&mut self) -> Option<T::OutputRefMut<'_>> {
            if !self.is_empty() {
                return unsafe { Some(self._front_mut()) };
            }
            None
        }
        #[cpp(method = "T& front()")]
        unsafe fn _front_mut(&mut self) -> &mut T;

        /// ```
        /// use hicc_std::DequeInt;
        /// let mut deque = DequeInt::new();
        /// deque.assign(10, &1);
        /// assert_eq!(deque.size(), 10);
        /// ```
        #[cpp(method = "void assign(size_t, const T&)")]
        pub fn assign(&mut self, n: usize, val: &T);

        /// 如果为空不做任何改变.
        /// ```
        /// use hicc_std::DequeInt;
        /// let mut deque = DequeInt::new();
        /// deque.pop_back();
        /// deque.push_back(&1);
        /// deque.pop_back();
        /// assert!(deque.is_empty());
        /// ```
        pub fn pop_back(&mut self) {
            if !self.is_empty() {
                unsafe { self._pop_back(); }
            }
        }
        #[cpp(method = "void pop_back()")]
        unsafe fn _pop_back(&mut self);

        /// ```
        /// use hicc_std::DequeInt;
        /// let mut deque = DequeInt::new();
        /// assert_eq!(deque.front(), None);
        /// deque.push_back(&1);
        /// assert_eq!(deque.front(), Some(&1));
        /// ```
        #[cpp(method = "void push_back(const T&)")]
        pub fn push_back(&mut self, val: &T);

        /// 如果为空不做任何改变.
        /// ```
        /// use hicc_std::DequeInt;
        /// let mut deque = DequeInt::new();
        /// deque.pop_front();
        /// deque.push_front(&1);
        /// deque.pop_front();
        /// assert!(deque.is_empty());
        /// ```
        pub fn pop_front(&mut self) {
            if !self.is_empty() {
                unsafe { self._pop_front(); }
            }
        }
        #[cpp(method = "void pop_front()")]
        unsafe fn _pop_front(&mut self);

        /// ```
        /// use hicc_std::DequeInt;
        /// let mut deque = DequeInt::new();
        /// deque.push_front(&1);
        /// assert_eq!(deque.front(), Some(&1));
        /// ```
        #[cpp(method = "void push_front(const T&)")]
        pub fn push_front(&mut self, val: &T);

        /// ```
        /// use hicc_std::DequeInt;
        /// let mut deque = DequeInt::new();
        /// deque.push_back(&1);
        /// deque.swap(&mut DequeInt::new());
        /// assert!(deque.is_empty());
        /// ```
        #[cpp(method = "void swap(Self&)")]
        pub fn swap(&mut self, other: &mut Self);

        /// ```
        /// use hicc_std::DequeInt;
        /// let mut deque = DequeInt::new();
        /// deque.insert(2, 10, &2);
        /// assert_eq!(deque.size(), 10);
        /// ```
        hicc::cpp! {
            static void insert(Self& self, size_t pos, size_t count, const T& val) {
                self.insert(self.begin() + std::min(pos, self.size()), count, val);
            }
        }
        #[cpp(func = "void SelfMethods::insert(Self&, size_t, size_t, const T&)")]
        pub fn insert(&mut self, pos: usize, count: usize, val: &T);

        /// ```
        /// use hicc_std::DequeInt;
        /// let mut deque = DequeInt::new();
        /// deque.erase(1, 9);
        /// deque.insert(0, 10, &2);
        /// deque.erase(1, 9);
        /// assert_eq!(deque.size(), 1);
        /// ```
        hicc::cpp! {
            static void erase(Self& self, size_t pos, size_t count) {
                pos = std::min(pos, self.size());
                count = std::min(count, self.size() - pos);
                self.erase(self.begin() + pos, self.begin() + pos + count);
            }
        }
        #[cpp(func = "void SelfMethods::erase(Self&, size_t, size_t)")]
        pub fn erase(&mut self, pos: usize, count: usize);
    }
}

unsafe impl<T: AbiType + Sync> Send for deque<T> {}
unsafe impl<T: AbiType + Sync> Sync for deque<T> {}

impl<T: AbiType> deque<T> {
    /// ```
    /// use hicc_std::DequeInt;
    /// let mut deque = DequeInt::new();
    /// deque.push_front(&1);
    /// deque.push_front(&2);
    /// let mut it = deque.iter();
    /// assert_eq!(it.next(), Some(&2));
    /// assert_eq!(it.next(), Some(&1));
    /// assert_eq!(it.next(), None);
    /// ```
    pub fn iter(&self) -> impl Iterator<Item = T::OutputRef<'_>> {
        DequeIter {
            deque: self,
            pos: 0,
            _mark: PhantomData,
        }
    }

    /// ```
    /// use hicc_std::DequeInt;
    /// let mut deque = DequeInt::new();
    /// deque.push_front(&1);
    /// deque.push_front(&3);
    /// deque.iter_mut().for_each(|v| *v += 1);
    /// let mut it = deque.iter();
    /// assert_eq!(it.next(), Some(&4));
    /// assert_eq!(it.next(), Some(&2));
    /// assert_eq!(it.next(), None);
    /// ```
    pub fn iter_mut(&mut self) -> impl Iterator<Item = T::OutputRefMut<'_>> {
        DequeIterMut {
            deque: self,
            pos: 0,
            _mark: PhantomData,
        }
    }

    /// ```
    /// use hicc_std::DequeInt;
    /// let mut deque = DequeInt::new();
    /// deque.push_front(&1);
    /// deque.push_front(&2);
    /// let mut it = deque.rev_iter();
    /// assert_eq!(it.next(), Some(&1));
    /// assert_eq!(it.next(), Some(&2));
    /// assert_eq!(it.next(), None);
    /// ```
    pub fn rev_iter(&self) -> impl Iterator<Item = T::OutputRef<'_>> {
        DequeRevIter {
            deque: self,
            pos: self.size(),
            _mark: PhantomData,
        }
    }

    /// ```
    /// use hicc_std::DequeInt;
    /// let mut deque = DequeInt::new();
    /// deque.push_front(&1);
    /// deque.push_front(&3);
    /// deque.rev_iter_mut().for_each(|v| *v += 1);
    /// let mut it = deque.rev_iter();
    /// assert_eq!(it.next(), Some(&2));
    /// assert_eq!(it.next(), Some(&4));
    /// assert_eq!(it.next(), None);
    /// ```
    pub fn rev_iter_mut(&mut self) -> impl Iterator<Item = T::OutputRefMut<'_>> {
        DequeRevIterMut {
            deque: self,
            pos: self.size(),
            _mark: PhantomData,
        }
    }
}

/// 对应`std::deque<T>::const_iterator`
struct DequeIter<'a, T: AbiType + 'static> {
    deque: *const deque<T>,
    pos: usize,
    _mark: PhantomData<&'a deque<T>>,
}

impl<'a, T: AbiType + 'static> Iterator for DequeIter<'a, T> {
    type Item = T::OutputRef<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        let deque = unsafe { &*self.deque };
        if self.pos < deque.size() {
            self.pos += 1;
            return unsafe { Some(deque._at(self.pos - 1)) };
        }
        None
    }
}

/// 对应`std::deque<T>::iterator`
struct DequeIterMut<'a, T: AbiType + 'static> {
    deque: *mut deque<T>,
    pos: usize,
    _mark: PhantomData<&'a mut deque<T>>,
}

impl<'a, T: AbiType + 'static> Iterator for DequeIterMut<'a, T> {
    type Item = T::OutputRefMut<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        let deque = unsafe { &mut *self.deque };
        if self.pos < deque.size() {
            self.pos += 1;
            return unsafe { Some(deque._at_mut(self.pos - 1)) };
        }
        None
    }
}

/// 对应`std::deque<T>::const_iterator`
struct DequeRevIter<'a, T: AbiType + 'static> {
    deque: *const deque<T>,
    pos: usize,
    _mark: PhantomData<&'a deque<T>>,
}

impl<'a, T: AbiType + 'static> Iterator for DequeRevIter<'a, T> {
    type Item = T::OutputRef<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        let deque = unsafe { &*self.deque };
        if self.pos > 0 {
            self.pos -= 1;
            return unsafe { Some(deque._at(self.pos)) };
        }
        None
    }
}

/// 对应`std::deque<T>::iterator`
struct DequeRevIterMut<'a, T: AbiType + 'static> {
    deque: *mut deque<T>,
    pos: usize,
    _mark: PhantomData<&'a mut deque<T>>,
}

impl<'a, T: AbiType + 'static> Iterator for DequeRevIterMut<'a, T> {
    type Item = T::OutputRefMut<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        let deque = unsafe { &mut *self.deque };
        if self.pos > 0 {
            self.pos -= 1;
            return unsafe { Some(deque._at_mut(self.pos)) };
        }
        None
    }
}
