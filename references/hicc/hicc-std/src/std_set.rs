use hicc::{AbiType, ClassMutPtr};
use std::iter::Iterator;

hicc::cpp! {
    #include <set>
}

hicc::import_class! {
    #[cpp(class = "template<class T, class Compare, class Allocator> std::set<T, Compare, Allocator>")]
    pub class set<T> {
        hicc::cpp! {
            typedef typename Self::const_iterator const_iterator;
            typedef typename Self::const_reverse_iterator const_reverse_iterator;
        }
        /// ```
        /// use hicc_std::SetInt;
        /// let set = SetInt::new();
        /// assert!(set.is_empty());
        /// ```
        #[cpp(method = "bool empty() const")]
        pub fn is_empty(&self) -> bool;

        /// ```
        /// use hicc_std::SetInt;
        /// let set = SetInt::new();
        /// assert_eq!(set.size(), 0_usize);
        /// ```
        #[cpp(method = "size_t size() const")]
        pub fn size(&self) -> usize;

        /// ```
        /// use hicc_std::SetInt;
        /// let set = SetInt::new();
        /// println!("set.max_size = {}", set.max_size());
        /// ```
        #[cpp(method = "size_t max_size() const")]
        pub fn max_size(&self) -> usize;

        /// ```
        /// use hicc_std::SetInt;
        /// let mut set = SetInt::new();
        /// set.insert(&1);
        /// set.clear();
        /// assert!(set.is_empty());
        /// ```
        #[cpp(method = "void clear()")]
        pub fn clear(&mut self);

        /// ```
        /// use hicc_std::SetInt;
        /// let mut set = SetInt::new();
        /// set.insert(&1);
        /// set.swap(&mut SetInt::new());
        /// assert!(set.is_empty());
        /// ```
        #[cpp(method = "void swap(Self&)")]
        pub fn swap(&mut self, other: &mut Self);

        /// ```
        /// use hicc_std::SetInt;
        /// let mut set = SetInt::new();
        /// set.insert(&1);
        /// assert_eq!(set.count(&1), 1);
        /// ```
        #[cpp(method = "size_t count(const T&) const")]
        pub fn count(&self, val: &T) -> usize;

        hicc::cpp! {
            static bool contains(const Self& self, const T& val) {
                return self.find(val) != self.end();
            }
        }
        /// ```
        /// use hicc_std::SetInt;
        /// let mut set = SetInt::new();
        /// set.insert(&1);
        /// assert!(set.contains(&1));
        /// ```
        #[cpp(func = "bool SelfMethods::contains(const Self&, const T&)")]
        pub fn contains(&self, val: &T) -> bool;

        /// ```
        /// use hicc_std::SetInt;
        /// let mut set = SetInt::new();
        /// set.insert(&1);
        /// set.assign(&SetInt::new());
        /// assert!(set.is_empty());
        /// ```
        #[cpp(func = "void hicc::make_assign<Self, Self>(Self&, const Self&)")]
        pub fn assign(&mut self, other: &Self);

        hicc::cpp! {
            static bool insert(Self& self, const T& val) {
                return self.insert(val).second;
            }
        }
        /// ```
        /// use hicc_std::SetInt;
        /// let mut set = SetInt::new();
        /// assert!(set.insert(&1));
        /// assert!(!set.insert(&1));
        /// ```
        #[cpp(func = "bool SelfMethods::insert(Self&, const T&)")]
        pub fn insert(&mut self, val: &T) -> bool;

        /// ```
        /// use hicc_std::SetInt;
        /// let mut set = SetInt::new();
        /// assert!(set.insert(&1));
        /// set.erase(&1);
        /// assert!(set.insert(&1));
        /// ```
        #[cpp(method = "size_t erase(const T&)")]
        pub fn erase(&mut self, val: &T) -> usize;

        #[cpp(method = "const_iterator find(const T&) const")]
        unsafe fn find(&self, val: &T) -> *mut CppSetIter<T>;
        #[cpp(method = "const_iterator lower_bound(const T&) const")]
        unsafe fn lower_bound(&self, val: &T) -> *mut CppSetIter<T>;
        #[cpp(method = "const_iterator upper_bound(const T&) const")]
        unsafe fn upper_bound(&self, val: &T) -> *mut CppSetIter<T>;
        #[cpp(method = "const_iterator begin() const")]
        unsafe fn begin(&self) -> *mut CppSetIter<T>;
        #[cpp(method = "const_iterator end() const")]
        unsafe fn end(&self) -> *mut CppSetIter<T>;
        #[cpp(method = "const_reverse_iterator rbegin() const")]
        unsafe fn rbegin(&self) -> *mut CppSetRevIter<T>;
        #[cpp(method = "const_reverse_iterator rend() const")]
        unsafe fn rend(&self) -> *mut CppSetRevIter<T>;
    }

    unsafe impl<T: AbiType + Sync> Send for set<T> {}
    unsafe impl<T: AbiType + Sync> Sync for set<T> {}

    #[cpp(class = "template<class T, class Compare, class Allocator> std::set<T, Compare, Allocator>::const_iterator")]
    class CppSetIter<T> {
        hicc::cpp! {
            typedef typename SelfContainer::const_reverse_iterator const_reverse_iterator;
            static const T& next(Self& self) {
                return *self++;
            }
        }
        #[cpp(func = "const T& SelfMethods::next(Self&)")]
        unsafe fn next(&mut self) -> &T;
        #[cpp(func = "bool hicc::make_eq<Self, Self>(const Self&, const Self&)")]
        fn equal(&self, other: &Self) -> bool;
        #[cpp(func = "const_reverse_iterator hicc::make_constructor<const_reverse_iterator, Self>(Self&&)")]
        fn into_reverse(self) -> *mut CppSetRevIter<T>;
    }

    #[cpp(class = "template<class T, class Compare, class Allocator> std::set<T, Compare, Allocator>::const_reverse_iterator")]
    class CppSetRevIter<T> {
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
}

impl<T: AbiType> set<T> {
    /// ```
    /// use hicc_std::SetInt;
    /// let mut set = SetInt::new();
    /// set.insert(&1);
    /// set.insert(&2);
    /// set.iter().for_each(|v| println!("value = {v}"));
    /// ```
    pub fn iter(&self) -> impl Iterator<Item = T::OutputRef<'_>> {
        SetIter {
            beg: unsafe { self.begin() },
            end: unsafe { self.end() },
        }
    }

    /// ```
    /// use hicc_std::SetInt;
    /// let mut set = SetInt::new();
    /// set.insert(&1);
    /// set.insert(&2);
    /// set.rev_iter().for_each(|v| println!("value = {v}"));
    /// ```
    pub fn rev_iter(&self) -> impl Iterator<Item = T::OutputRef<'_>> {
        SetRevIter {
            beg: unsafe { self.rbegin() },
            end: unsafe { self.rend() },
        }
    }

    /// ```
    /// use hicc_std::SetInt;
    /// let mut set = SetInt::new();
    /// set.insert(&1);
    /// set.insert(&2);
    /// set.insert(&3);
    /// let mut it = set.iter_lower_upper_bound(Some(&1), Some(&2));
    /// assert_eq!(it.next(), Some(&1));
    /// assert_eq!(it.next(), Some(&2));
    /// assert_eq!(it.next(), None);
    /// ```
    pub fn iter_lower_upper_bound(
        &self,
        lower_key: Option<&T::InputType>,
        upper_key: Option<&T::InputType>,
    ) -> impl Iterator<Item = T::OutputRef<'_>> {
        let beg = if let Some(key) = lower_key {
            unsafe { self.lower_bound(key) }
        } else {
            unsafe { self.begin() }
        };
        let end = if let Some(key) = upper_key {
            unsafe { self.upper_bound(key) }
        } else {
            unsafe { self.end() }
        };
        SetIter { beg, end }
    }

    /// ```
    /// use hicc_std::SetInt;
    /// let mut set = SetInt::new();
    /// set.insert(&1);
    /// set.insert(&2);
    /// set.insert(&3);
    /// let mut it = set.rev_iter_lower_upper_bound(Some(&1), Some(&2));
    /// assert_eq!(it.next(), Some(&2));
    /// assert_eq!(it.next(), Some(&1));
    /// assert_eq!(it.next(), None);
    /// ```
    pub fn rev_iter_lower_upper_bound(
        &self,
        lower_key: Option<&T::InputType>,
        upper_key: Option<&T::InputType>,
    ) -> impl Iterator<Item = T::OutputRef<'_>> {
        let beg = if let Some(key) = upper_key {
            unsafe { self.upper_bound(key).into_value().into_reverse() }
        } else {
            unsafe { self.rbegin() }
        };
        let end = if let Some(key) = lower_key {
            unsafe { self.lower_bound(key).into_value().into_reverse() }
        } else {
            unsafe { self.rend() }
        };
        SetRevIter { beg, end }
    }
}

/// 对应`std::set<T>::const_iterator`
struct SetIter<'a, T: AbiType + 'static> {
    beg: ClassMutPtr<'a, CppSetIter<T>>,
    end: ClassMutPtr<'a, CppSetIter<T>>,
}

impl<'a, T: AbiType + 'static> Iterator for SetIter<'a, T> {
    type Item = T::OutputRef<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        if !self.beg.equal(&self.end) {
            return unsafe { Some(self.beg.as_deref_mut().next()) };
        }
        None
    }
}

/// 对应`std::set<T>::const_reverse_iterator`
struct SetRevIter<'a, T: AbiType + 'static> {
    beg: ClassMutPtr<'a, CppSetRevIter<T>>,
    end: ClassMutPtr<'a, CppSetRevIter<T>>,
}

impl<'a, T: AbiType + 'static> Iterator for SetRevIter<'a, T> {
    type Item = T::OutputRef<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        if !self.beg.equal(&self.end) {
            return unsafe { Some(self.beg.as_deref_mut().next()) };
        }
        None
    }
}

hicc::import_class! {
    #[cpp(class = "template<class T, class Compare, class Allocator> std::multiset<T, Compare, Allocator>")]
    pub class multiset<T> {
        hicc::cpp! {
            typedef typename Self::iterator iterator;
            typedef typename Self::const_iterator const_iterator;
            typedef typename Self::const_reverse_iterator const_reverse_iterator;
        }
        /// ```
        /// use hicc_std::MultiSetInt;
        /// let set = MultiSetInt::new();
        /// assert!(set.is_empty());
        /// ```
        #[cpp(method = "bool empty() const")]
        pub fn is_empty(&self) -> bool;

        /// ```
        /// use hicc_std::MultiSetInt;
        /// let set = MultiSetInt::new();
        /// assert_eq!(set.size(), 0);
        /// ```
        #[cpp(method = "size_t size() const")]
        pub fn size(&self) -> usize;

        /// ```
        /// use hicc_std::MultiSetInt;
        /// let set = MultiSetInt::new();
        /// println!("set.max_size = {}", set.max_size());
        /// ```
        #[cpp(method = "size_t max_size() const")]
        pub fn max_size(&self) -> usize;

        /// ```
        /// use hicc_std::MultiSetInt;
        /// let mut set = MultiSetInt::new();
        /// set.insert(&1);
        /// set.clear();
        /// assert_eq!(set.size(), 0);
        /// ```
        #[cpp(method = "void clear()")]
        pub fn clear(&mut self);

        /// ```
        /// use hicc_std::MultiSetInt;
        /// let mut set = MultiSetInt::new();
        /// set.insert(&1);
        /// set.swap(&mut MultiSetInt::new());
        /// assert_eq!(set.size(), 0);
        /// ```
        #[cpp(method = "void swap(Self&)")]
        pub fn swap(&mut self, other: &mut Self);

        /// ```
        /// use hicc_std::MultiSetInt;
        /// let mut set = MultiSetInt::new();
        /// set.insert(&1);
        /// assert_eq!(set.count(&1), 1);
        /// ```
        #[cpp(method = "size_t count(const T&) const")]
        pub fn count(&self, val: &T) -> usize;

        hicc::cpp! {
            static bool contains(const Self& self, const T& val) {
                return self.find(val) != self.end();
            }
        }
        /// ```
        /// use hicc_std::MultiSetInt;
        /// let mut set = MultiSetInt::new();
        /// set.insert(&1);
        /// assert!(set.contains(&1));
        /// ```
        #[cpp(func = "bool SelfMethods::contains(const Self&, const T&)")]
        pub fn contains(&self, val: &T) -> bool;

        /// ```
        /// use hicc_std::MultiSetInt;
        /// let mut set = MultiSetInt::new();
        /// set.insert(&1);
        /// set.assign(&MultiSetInt::new());
        /// assert_eq!(set.size(), 0);
        /// ```
        #[cpp(func = "void hicc::make_assign<Self, Self>(Self&, const Self&)")]
        pub fn assign(&mut self, other: &Self);

        /// ```
        /// use hicc_std::MultiSetInt;
        /// let mut set = MultiSetInt::new();
        /// set.insert(&1);
        /// set.insert(&1);
        /// assert_eq!(set.count(&1), 2);
        /// ```
        #[cpp(method = "iterator insert(const T&)")]
        pub fn insert(&mut self, val: &T);

        /// ```
        /// use hicc_std::MultiSetInt;
        /// let mut set = MultiSetInt::new();
        /// set.insert(&1);
        /// set.insert(&1);
        /// assert_eq!(set.erase(&1), 2);
        /// ```
        #[cpp(method = "size_t erase(const T&)")]
        pub fn erase(&mut self, val: &T) -> usize;

        #[cpp(method = "const_iterator find(const T&) const")]
        unsafe fn find(&self, val: &T) -> *mut CppMultiSetIter<T>;
        #[cpp(method = "const_iterator lower_bound(const T&) const")]
        unsafe fn lower_bound(&self, val: &T) -> *mut CppMultiSetIter<T>;
        #[cpp(method = "const_iterator upper_bound(const T&) const")]
        unsafe fn upper_bound(&self, val: &T) -> *mut CppMultiSetIter<T>;
        #[cpp(method = "const_iterator begin() const")]
        unsafe fn begin(&self) -> *mut CppMultiSetIter<T>;
        #[cpp(method = "const_iterator end() const")]
        unsafe fn end(&self) -> *mut CppMultiSetIter<T>;
        #[cpp(method = "const_reverse_iterator crbegin() const")]
        unsafe fn rbegin(&self) -> *mut CppMultiSetRevIter<T>;
        #[cpp(method = "const_reverse_iterator crend() const")]
        unsafe fn rend(&self) -> *mut CppMultiSetRevIter<T>;

    }

    unsafe impl<T: AbiType + Sync> Send for multiset<T> {}
    unsafe impl<T: AbiType + Sync> Sync for multiset<T> {}

    #[cpp(class = "template<class T, class Compare, class Allocator> std::multiset<T, Compare, Allocator>::const_iterator")]
    class CppMultiSetIter<T> {
        hicc::cpp! {
            typedef typename SelfContainer::const_reverse_iterator const_reverse_iterator;
            static const T& next(Self& self) {
                return *self++;
            }
        }
        #[cpp(func = "const T& SelfMethods::next(Self&)")]
        unsafe fn next(&mut self) -> &T;
        #[cpp(func = "bool hicc::make_eq<Self, Self>(const Self&, const Self&)")]
        fn equal(&self, other: &Self) -> bool;
        #[cpp(func = "const_reverse_iterator hicc::make_constructor<const_reverse_iterator, Self>(Self&&)")]
        fn into_reverse(self) -> *mut CppMultiSetRevIter<T>;
    }

    #[cpp(class = "template<class T, class Compare, class Allocator> std::multiset<T, Compare, Allocator>::const_reverse_iterator")]
    class CppMultiSetRevIter<T> {
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
}

impl<T: AbiType> multiset<T> {
    /// ```
    /// use hicc_std::MultiSetInt;
    /// let mut set = MultiSetInt::new();
    /// set.insert(&1);
    /// set.insert(&2);
    /// set.iter().for_each(|v| println!("value = {v}"));
    /// ```
    pub fn iter(&self) -> impl Iterator<Item = T::OutputRef<'_>> {
        MultiSetIter {
            beg: unsafe { self.begin() },
            end: unsafe { self.end() },
        }
    }

    /// ```
    /// use hicc_std::MultiSetInt;
    /// let mut set = MultiSetInt::new();
    /// set.insert(&1);
    /// set.insert(&2);
    /// set.rev_iter().for_each(|v| println!("value = {v}"));
    /// ```
    pub fn rev_iter(&self) -> impl Iterator<Item = T::OutputRef<'_>> {
        MultiSetRevIter {
            beg: unsafe { self.rbegin() },
            end: unsafe { self.rend() },
        }
    }

    /// ```
    /// use hicc_std::MultiSetInt;
    /// let mut set = MultiSetInt::new();
    /// set.insert(&1);
    /// set.insert(&2);
    /// set.insert(&2);
    /// set.insert(&3);
    /// let mut it = set.iter_lower_upper_bound(Some(&1), Some(&2));
    /// assert_eq!(it.next(), Some(&1));
    /// assert_eq!(it.next(), Some(&2));
    /// assert_eq!(it.next(), Some(&2));
    /// assert_eq!(it.next(), None);
    /// ```
    pub fn iter_lower_upper_bound(
        &self,
        lower_key: Option<&T::InputType>,
        upper_key: Option<&T::InputType>,
    ) -> impl Iterator<Item = T::OutputRef<'_>> {
        let beg = if let Some(key) = lower_key {
            unsafe { self.lower_bound(key) }
        } else {
            unsafe { self.begin() }
        };
        let end = if let Some(key) = upper_key {
            unsafe { self.upper_bound(key) }
        } else {
            unsafe { self.end() }
        };
        MultiSetIter { beg, end }
    }

    /// ```
    /// use hicc_std::MultiSetInt;
    /// let mut set = MultiSetInt::new();
    /// set.insert(&1);
    /// set.insert(&1);
    /// set.insert(&2);
    /// set.insert(&3);
    /// let mut it = set.rev_iter_lower_upper_bound(Some(&1), Some(&2));
    /// assert_eq!(it.next(), Some(&2));
    /// assert_eq!(it.next(), Some(&1));
    /// assert_eq!(it.next(), Some(&1));
    /// assert_eq!(it.next(), None);
    /// ```
    pub fn rev_iter_lower_upper_bound(
        &self,
        lower_key: Option<&T::InputType>,
        upper_key: Option<&T::InputType>,
    ) -> impl Iterator<Item = T::OutputRef<'_>> {
        let beg = if let Some(key) = upper_key {
            unsafe { self.upper_bound(key).into_value().into_reverse() }
        } else {
            unsafe { self.rbegin() }
        };
        let end = if let Some(key) = lower_key {
            unsafe { self.lower_bound(key).into_value().into_reverse() }
        } else {
            unsafe { self.rend() }
        };
        MultiSetRevIter { beg, end }
    }
}

/// 对应`std::multiset<T>::const_iterator`
struct MultiSetIter<'a, T: AbiType + 'static> {
    beg: ClassMutPtr<'a, CppMultiSetIter<T>>,
    end: ClassMutPtr<'a, CppMultiSetIter<T>>,
}

impl<'a, T: AbiType + 'static> Iterator for MultiSetIter<'a, T> {
    type Item = T::OutputRef<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        if !self.beg.equal(&self.end) {
            return Some(unsafe { self.beg.as_deref_mut().next() });
        }
        None
    }
}

/// 对应`std::multiset<T>::const_reverse_iterator`
struct MultiSetRevIter<'a, T: AbiType + 'static> {
    beg: ClassMutPtr<'a, CppMultiSetRevIter<T>>,
    end: ClassMutPtr<'a, CppMultiSetRevIter<T>>,
}

impl<'a, T: AbiType + 'static> Iterator for MultiSetRevIter<'a, T> {
    type Item = T::OutputRef<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        if !self.beg.equal(&self.end) {
            return Some(unsafe { self.beg.as_deref_mut().next() });
        }
        None
    }
}
