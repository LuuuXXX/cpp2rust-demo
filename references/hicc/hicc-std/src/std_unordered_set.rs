use hicc::{AbiType, ClassMutPtr};
use std::iter::Iterator;

hicc::cpp! {
    #include <unordered_set>
}

hicc::import_class! {
    #[cpp(class = "template <class T, class Hash, class Pred, class Allocator> std::unordered_set<T, Hash, Pred, Allocator>")]
    pub class unordered_set<T> {
        hicc::cpp! {
            typedef typename Self::iterator iterator;
            typedef typename Self::const_iterator const_iterator;
        }
        /// ```
        /// use hicc_std::UnorderedSetInt;
        /// let set = UnorderedSetInt::new();
        /// assert!(set.is_empty());
        /// ```
        #[cpp(method = "bool empty() const")]
        pub fn is_empty(&self) -> bool;

        /// ```
        /// use hicc_std::UnorderedSetInt;
        /// let set = UnorderedSetInt::new();
        /// assert_eq!(set.size(), 0);
        /// ```
        #[cpp(method = "size_t size() const")]
        pub fn size(&self) -> usize;

        /// ```
        /// use hicc_std::UnorderedSetInt;
        /// let set = UnorderedSetInt::new();
        /// println!("set.max_size = {}", set.max_size());
        /// ```
        #[cpp(method = "size_t max_size() const")]
        pub fn max_size(&self) -> usize;

        /// ```
        /// use hicc_std::UnorderedSetInt;
        /// let mut set = UnorderedSetInt::new();
        /// set.insert(&1);
        /// set.clear();
        /// assert!(set.is_empty());
        /// ```
        #[cpp(method = "void clear()")]
        pub fn clear(&mut self);

        /// ```
        /// use hicc_std::UnorderedSetInt;
        /// let mut set = UnorderedSetInt::new();
        /// set.insert(&1);
        /// set.swap(&mut UnorderedSetInt::new());
        /// assert!(set.is_empty());
        /// ```
        #[cpp(method = "void swap(Self&)")]
        pub fn swap(&mut self, other: &mut Self);

        /// ```
        /// use hicc_std::UnorderedSetInt;
        /// let mut set = UnorderedSetInt::new();
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
        /// use hicc_std::UnorderedSetInt;
        /// let mut set = UnorderedSetInt::new();
        /// set.insert(&1);
        /// assert!(set.contains(&1));
        /// ```
        #[cpp(func = "bool SelfMethods::contains(const Self&, const T&)")]
        pub fn contains(&self, val: &T) -> bool;

        /// ```
        /// use hicc_std::UnorderedSetInt;
        /// let mut set = UnorderedSetInt::new();
        /// set.insert(&1);
        /// set.assign(&UnorderedSetInt::new());
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
        /// use hicc_std::UnorderedSetInt;
        /// let mut set = UnorderedSetInt::new();
        /// assert!(set.insert(&1));
        /// assert!(!set.insert(&1));
        /// ```
        #[cpp(func = "bool SelfMethods::insert(Self&, const T&)")]
        pub fn insert(&mut self, val: &T) -> bool;

        /// ```
        /// use hicc_std::UnorderedSetInt;
        /// let mut set = UnorderedSetInt::new();
        /// set.insert(&1);
        /// assert_eq!(set.erase(&1), 1);
        /// ```
        #[cpp(method = "size_t erase(const T&)")]
        pub fn erase(&mut self, val: &T) -> usize;

        #[cpp(method = "const_iterator begin() const")]
        unsafe fn begin(&self) -> *mut CppUnorderedSetIter<T>;
        #[cpp(method = "const_iterator end() const")]
        unsafe fn end(&self) -> *mut CppUnorderedSetIter<T>;

    }

    unsafe impl<T: AbiType + Sync> Send for unordered_set<T> {}
    unsafe impl<T: AbiType + Sync> Sync for unordered_set<T> {}

    #[cpp(class = "template <class T, class Hash, class Pred, class Allocator> std::unordered_set<T, Hash, Pred, Allocator>::const_iterator")]
    class CppUnorderedSetIter<T> {
        hicc::cpp! {
            static const T& next(Self& self) {
                return *self++;
            }
        }
        #[cpp(func = "const T& next(Self&)")]
        unsafe fn next(&mut self) -> &T;
        #[cpp(func = "bool hicc::make_eq<Self, Self>(const Self&, const Self&)")]
        fn equal(&self, other: &Self) -> bool;
    }
}

impl<T: AbiType> unordered_set<T> {
    /// ```
    /// use hicc_std::UnorderedSetInt;
    /// let mut set = UnorderedSetInt::new();
    /// set.insert(&1);
    /// set.insert(&2);
    /// set.iter().for_each(|v| {println!("{v}");});
    /// ```
    pub fn iter(&self) -> impl Iterator<Item = T::OutputRef<'_>> {
        UnorderedSetIter {
            beg: unsafe { self.begin() },
            end: unsafe { self.end() },
        }
    }
}

/// 对应`std::unordered_set<T>::const_iterator`
struct UnorderedSetIter<'a, T: AbiType + 'static> {
    beg: ClassMutPtr<'a, CppUnorderedSetIter<T>>,
    end: ClassMutPtr<'a, CppUnorderedSetIter<T>>,
}

impl<'a, T: AbiType + 'static> Iterator for UnorderedSetIter<'a, T> {
    type Item = T::OutputRef<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        if !self.beg.equal(&self.end) {
            return unsafe { Some(self.beg.as_deref_mut().next()) };
        }
        None
    }
}

hicc::import_class! {
    #[cpp(class = "template <class T, class Hash, class Pred, class Allocator> std::unordered_multiset<T, Hash, Pred, Allocator>")]
    pub class unordered_multiset<T> {
        hicc::cpp! {
            typedef typename Self::iterator iterator;
            typedef typename Self::const_iterator const_iterator;
        }
        /// ```
        /// use hicc_std::UnorderedMultiSetInt;
        /// let set = UnorderedMultiSetInt::new();
        /// assert!(set.is_empty());
        /// ```
        #[cpp(method = "bool empty() const")]
        pub fn is_empty(&self) -> bool;

        /// ```
        /// use hicc_std::UnorderedMultiSetInt;
        /// let set = UnorderedMultiSetInt::new();
        /// assert_eq!(set.size(), 0);
        /// ```
        #[cpp(method = "size_t size() const")]
        pub fn size(&self) -> usize;

        /// ```
        /// use hicc_std::UnorderedMultiSetInt;
        /// let set = UnorderedMultiSetInt::new();
        /// println!("set.max_size = {}", set.max_size());
        /// ```
        #[cpp(method = "size_t max_size() const")]
        pub fn max_size(&self) -> usize;

        /// ```
        /// use hicc_std::UnorderedMultiSetInt;
        /// let mut set = UnorderedMultiSetInt::new();
        /// set.insert(&1);
        /// set.clear();
        /// assert!(set.is_empty());
        /// ```
        #[cpp(method = "void clear()")]
        pub fn clear(&mut self);

        /// ```
        /// use hicc_std::UnorderedMultiSetInt;
        /// let mut set = UnorderedMultiSetInt::new();
        /// set.insert(&1);
        /// set.swap(&mut UnorderedMultiSetInt::new());
        /// assert!(set.is_empty());
        /// ```
        #[cpp(method = "void swap(Self&)")]
        pub fn swap(&mut self, other: &mut Self);

        /// ```
        /// use hicc_std::UnorderedMultiSetInt;
        /// let mut set = UnorderedMultiSetInt::new();
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
        /// use hicc_std::UnorderedMultiSetInt;
        /// let mut set = UnorderedMultiSetInt::new();
        /// set.contains(&1);
        /// assert!(set.is_empty());
        /// ```
        #[cpp(func = "bool SelfMethods::contains(const Self&, const T&)")]
        pub fn contains(&self, val: &T) -> bool;

        /// ```
        /// use hicc_std::UnorderedMultiSetInt;
        /// let mut set = UnorderedMultiSetInt::new();
        /// set.insert(&1);
        /// set.assign(&UnorderedMultiSetInt::new());
        /// assert!(set.is_empty());
        /// ```
        #[cpp(func = "void hicc::make_assign<Self, Self>(Self&, const Self&)")]
        pub fn assign(&mut self, other: &Self);

        /// ```
        /// use hicc_std::UnorderedMultiSetInt;
        /// let mut set = UnorderedMultiSetInt::new();
        /// set.insert(&1);
        /// set.insert(&1);
        /// assert_eq!(set.size(), 2);
        /// ```
        #[cpp(method = "iterator insert(const T&)")]
        pub fn insert(&mut self, val: &T);

        /// ```
        /// use hicc_std::UnorderedMultiSetInt;
        /// let mut set = UnorderedMultiSetInt::new();
        /// set.insert(&1);
        /// set.insert(&1);
        /// assert_eq!(set.erase(&1), 2);
        /// ```
        #[cpp(method = "size_t erase(const T&)")]
        pub fn erase(&mut self, val: &T) -> usize;

        #[cpp(method = "const_iterator begin() const")]
        unsafe fn begin(&self) -> *mut CppUnorderedMultiSetIter<T>;
        #[cpp(method = "const_iterator end() const")]
        unsafe fn end(&self) -> *mut CppUnorderedMultiSetIter<T>;

    }

    unsafe impl<T: AbiType + Sync> Send for unordered_multiset<T> {}
    unsafe impl<T: AbiType + Sync> Sync for unordered_multiset<T> {}

    #[cpp(class = "template <class T, class Hash, class Pred, class Allocator> std::unordered_multiset<T, Hash, Pred, Allocator>::const_iterator")]
    class CppUnorderedMultiSetIter<T> {
        hicc::cpp! {
            static const T& next(Self& self) {
                return *self++;
            }
        }
        #[cpp(func = "const T& SelfMethods::next(Self&)")]
        fn next(&mut self) -> &T;
        #[cpp(func = "bool hicc::make_eq<Self, Self>(const Self&, const Self&)")]
        fn equal(&self, other: &Self) -> bool;
    }
}

impl<T: AbiType> unordered_multiset<T> {
    /// ```
    /// use hicc_std::UnorderedMultiSetInt;
    /// let mut set = UnorderedMultiSetInt::new();
    /// set.insert(&1);
    /// set.insert(&2);
    /// set.insert(&1);
    /// set.iter().for_each(|v| { println!("{v}"); });
    /// ```
    pub fn iter(&self) -> impl Iterator<Item = T::OutputRef<'_>> {
        UnorderedMultiSetIter {
            beg: unsafe { self.begin() },
            end: unsafe { self.end() },
        }
    }
}

/// 对应`std::unordered_multiset<T>::const_iterator`
struct UnorderedMultiSetIter<'a, T: AbiType + 'static> {
    beg: ClassMutPtr<'a, CppUnorderedMultiSetIter<T>>,
    end: ClassMutPtr<'a, CppUnorderedMultiSetIter<T>>,
}

impl<'a, T: AbiType + 'static> Iterator for UnorderedMultiSetIter<'a, T> {
    type Item = T::OutputRef<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.beg.equal(&self.end) {
            return unsafe { Some(self.beg.as_deref_mut().next()) };
        }
        None
    }
}
