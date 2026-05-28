use hicc::{AbiType, ClassMutPtr};
use std::iter::Iterator;
use std::marker::PhantomData;

hicc::cpp! {
    #include <unordered_map>
}

hicc::import_class! {
    #[cpp(class = "template <class K, class V, class Hash, class Pred, class Allocator> std::unordered_map<K, V, Hash, Pred, Allocator>")]
    pub class unordered_map<K, V> {
        hicc::cpp! {
            typedef typename Self::iterator iterator;
            typedef typename Self::const_iterator const_iterator;
        }
        /// ```
        /// use hicc_std::UnorderedMapIntInt;
        /// let map = UnorderedMapIntInt::new();
        /// assert!(map.is_empty());
        /// ```
        #[cpp(method = "bool empty() const")]
        pub fn is_empty(&self) -> bool;

        /// ```
        /// use hicc_std::UnorderedMapIntInt;
        /// let map = UnorderedMapIntInt::new();
        /// assert_eq!(map.size(), 0_usize);
        /// ```
        #[cpp(method = "size_t size() const")]
        pub fn size(&self) -> usize;

        /// ```
        /// use hicc_std::UnorderedMapIntInt;
        /// let map = UnorderedMapIntInt::new();
        /// println!("map.max_size() = {}", map.max_size());
        /// ```
        #[cpp(method = "size_t max_size() const")]
        pub fn max_size(&self) -> usize;

        /// ```
        /// use hicc_std::UnorderedMapIntInt;
        /// let mut map = UnorderedMapIntInt::new();
        /// map.insert(&1, &2);
        /// assert_eq!(map.size(), 1_usize);
        /// map.clear();
        /// assert_eq!(map.size(), 0_usize);
        /// ```
        #[cpp(method = "void clear()")]
        pub fn clear(&mut self);

        /// ```
        /// use hicc_std::UnorderedMapIntInt;
        /// let mut map1 = UnorderedMapIntInt::new();
        /// map1.insert(&1, &2);
        /// let mut map2 = UnorderedMapIntInt::new();
        /// map2.swap(&mut map1);
        /// assert_eq!(map1.size(), 0_usize);
        /// assert_eq!(map2.size(), 1_usize);
        /// ```
        #[cpp(method = "void swap(Self&)")]
        pub fn swap(&mut self, other: &mut Self);

        /// ```
        /// use hicc_std::UnorderedMapIntInt;
        /// let mut map = UnorderedMapIntInt::new();
        /// map.insert(&1, &2);
        /// assert_eq!(map.count(&1), 1);
        /// assert_eq!(map.count(&2), 0);
        /// ```
        #[cpp(method = "size_t count(const K&) const")]
        pub fn count(&self, key: &K) -> usize;

        hicc::cpp! {
            static bool contains(const Self& self, const K& key) {
                return self.find(key) != self.end();
            }
        }
        /// ```
        /// use hicc_std::UnorderedMapIntInt;
        /// let mut map = UnorderedMapIntInt::new();
        /// map.insert(&1, &2);
        /// assert!(map.contains(&1));
        /// assert!(!map.contains(&2));
        /// ```
        #[cpp(func = "bool SelfMethods::contains(const Self&, const K&)")]
        pub fn contains(&self, key: &K) -> bool;

        /// ```
        /// use hicc_std::UnorderedMapIntInt;
        /// let mut map = UnorderedMapIntInt::new();
        /// map.insert(&1, &2);
        /// map.assign(&mut UnorderedMapIntInt::new());
        /// assert!(map.is_empty());
        /// ```
        #[cpp(func = "void hicc::make_assign<Self, Self>(Self&, const Self&)")]
        pub fn assign(&mut self, other: &Self);

        hicc::cpp! {
            static bool insert(Self& self, const K& key, const V& val) {
                return self.insert(std::make_pair(key, val)).second;
            }
        }
        /// ```
        /// use hicc_std::UnorderedMapIntInt;
        /// let mut map = UnorderedMapIntInt::new();
        /// assert!(map.insert(&1, &2));
        /// assert!(!map.insert(&1, &2));
        /// assert_eq!(map.get(&1), Some(&2));
        /// ```
        #[cpp(func = "bool SelfMethods::insert(Self&, const K&, const V&)")]
        pub fn insert(&mut self, key: &K, val: &V) -> bool;

        /// ```
        /// use hicc_std::UnorderedMapIntInt;
        /// let mut map = UnorderedMapIntInt::new();
        /// assert_eq!(map.erase(&1), 0_usize);
        /// assert!(map.insert(&1, &2));
        /// assert_eq!(map.erase(&1), 1_usize);
        /// ```
        #[cpp(method = "size_t erase(const K&)")]
        pub fn erase(&mut self, key: &K) -> usize;

        #[cpp(method = "const_iterator find(const K&) const")]
        unsafe fn find(&self, key: &K) -> *mut CppUnorderedMapIter<K, V>;
        #[cpp(method = "iterator find(const K&)")]
        unsafe fn find_mut(&mut self, key: &K) -> *mut CppUnorderedMapIterMut<K, V>;
        #[cpp(method = "const_iterator begin() const")]
        unsafe fn begin(&self) -> *mut CppUnorderedMapIter<K, V>;
        #[cpp(method = "iterator begin()")]
        unsafe fn begin_mut(&mut self) -> *mut CppUnorderedMapIterMut<K, V>;
        #[cpp(method = "const_iterator end() const")]
        unsafe fn end(&self) -> *mut CppUnorderedMapIter<K, V>;
        #[cpp(method = "iterator end()")]
        unsafe fn end_mut(&mut self) -> *mut CppUnorderedMapIterMut<K, V>;
    }

    unsafe impl<K: AbiType + Sync, V: AbiType + Sync> Send for unordered_map<K, V> {}
    unsafe impl<K: AbiType + Sync, V: AbiType + Sync> Sync for unordered_map<K, V> {}

    #[cpp(class = "template <class K, class V, class Hash, class Pred, class Allocator> std::unordered_map<K, V, Hash, Pred, Allocator>::const_iterator")]
    class CppUnorderedMapIter<K, V> {
        hicc::cpp! {
            static void next(Self& self) {
                ++self;
            }
            static const K& key(const Self& self) {
                return self->first;
            }
            static const V& value(const Self& self) {
                return self->second;
            }
        }
        #[cpp(func = "void SelfMethods::next(Self&)")]
        fn next(&mut self);
        #[cpp(func = "const K& SelfMethods::key(const Self&)")]
        fn as_key(&self) -> &K;
        #[cpp(func = "const V& SelfMethods::value(const Self&)")]
        fn as_value(&self) -> &V;
        #[cpp(func = "bool hicc::make_eq<Self, Self>(const Self&, const Self&)")]
        fn equal(&self, other: &Self) -> bool;
    }

    #[cpp(class = "template <class K, class V, class Hash, class Pred, class Allocator> std::unordered_map<K, V, Hash, Pred, Allocator>::iterator")]
    class CppUnorderedMapIterMut<K, V> {
        hicc::cpp! {
            static void next(Self& self) {
                ++self;
            }
            static const K& key(const Self& self) {
                return self->first;
            }
            static V& value(Self& self) {
                return self->second;
            }
        }
        #[cpp(func = "void SelfMethods::next(Self&)")]
        fn next(&mut self);
        #[cpp(func = "const K& SelfMethods::key(const Self&)")]
        fn as_key(&self) -> &K;
        #[cpp(func = "V& SelfMethods::value(Self&)")]
        fn as_value(&mut self) -> &mut V;
        #[cpp(func = "bool hicc::make_eq<Self, Self>(const Self&, const Self&)")]
        fn equal(&self, other: &Self) -> bool;
    }
}

impl<K: AbiType + 'static, V: AbiType + 'static> unordered_map<K, V> {
    /// ```
    /// use hicc_std::UnorderedMapIntInt;
    /// let mut map = UnorderedMapIntInt::new();
    /// map.insert(&1, &2);
    /// assert!(map.get(&2).is_none());
    /// assert_eq!(map.get(&1), Some(&2));
    /// ```
    pub fn get(&self, key: &K::InputType) -> Option<V::OutputRef<'_>> {
        unsafe {
            let it = self.find(key);
            if !it.equal(&self.end()) {
                return Some(it.as_deref().as_value());
            }
        }
        None
    }
    /// ```
    /// use hicc_std::UnorderedMapIntInt;
    /// let mut map = UnorderedMapIntInt::new();
    /// map.insert(&1, &2);
    /// *map.get_mut(&1).unwrap() = 0;
    /// assert_eq!(map.get(&1), Some(&0));
    ///
    /// use hicc_std::{string, UnorderedMapIntString};
    /// use hicc::AbiClass;
    /// let mut map = UnorderedMapIntString::new();
    /// map.insert(&1, &string::from(c"hello"));
    /// map.get_mut(&1).unwrap().write(string::from(c"world"));
    /// assert!(*map.get(&1).unwrap() == string::from(c"world"));
    /// ```
    pub fn get_mut(&mut self, key: &K::InputType) -> Option<V::OutputRefMut<'_>> {
        unsafe {
            let mut it = self.find_mut(key);
            if !it.equal(&self.end_mut()) {
                return Some(it.as_deref_mut().as_value());
            }
        }
        None
    }
    /// ```
    /// use hicc_std::UnorderedMapIntInt;
    /// let mut map = UnorderedMapIntInt::new();
    /// map.insert(&1, &2);
    /// map.insert(&2, &1);
    /// map.iter().for_each(|(k, v)| println!("key = {k}, value = {v}"));
    /// ```
    pub fn iter(&self) -> impl Iterator<Item = (K::OutputRef<'_>, V::OutputRef<'_>)> {
        UnorderedMapIter {
            beg: unsafe { self.begin() },
            end: unsafe { self.end() },
            mark: PhantomData,
        }
    }

    /// ```
    /// use hicc_std::UnorderedMapIntInt;
    /// let mut map = UnorderedMapIntInt::new();
    /// map.insert(&1, &2);
    /// map.insert(&2, &1);
    /// map.iter_mut().for_each(|(_, v)| *v += 1);
    /// assert_eq!(map.get(&1), Some(&3));
    /// assert_eq!(map.get(&2), Some(&2));
    /// ```
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (K::OutputRef<'_>, V::OutputRefMut<'_>)> {
        let beg = unsafe { self.begin_mut() };
        let end = unsafe { self.end_mut() };
        UnorderedMapIterMut {
            beg,
            end,
            mark: PhantomData,
        }
    }
}

/// 对应`std::unordered_map<K, V>::const_iterator`
struct UnorderedMapIter<'a, K: AbiType + 'static, V: AbiType + 'static> {
    beg: ClassMutPtr<'static, CppUnorderedMapIter<K, V>>,
    end: ClassMutPtr<'static, CppUnorderedMapIter<K, V>>,
    mark: PhantomData<&'a unordered_map<K, V>>,
}

impl<'a, K: AbiType + 'static, V: AbiType + 'static> Iterator for UnorderedMapIter<'a, K, V> {
    type Item = (K::OutputRef<'a>, V::OutputRef<'a>);
    fn next(&mut self) -> Option<Self::Item> {
        if !self.beg.equal(&self.end) {
            unsafe {
                let key = self.beg.as_deref().as_key();
                let val = self.beg.as_deref().as_value();
                self.beg.next();
                return Some((key, val));
            }
        }
        None
    }
}

/// 对应`std::unordered_map<K, V>::iterator`
struct UnorderedMapIterMut<'a, K: AbiType + 'static, V: AbiType + 'static> {
    beg: ClassMutPtr<'static, CppUnorderedMapIterMut<K, V>>,
    end: ClassMutPtr<'static, CppUnorderedMapIterMut<K, V>>,
    mark: PhantomData<&'a mut unordered_map<K, V>>,
}

impl<'a, K: AbiType + 'static, V: AbiType + 'static> Iterator for UnorderedMapIterMut<'a, K, V> {
    type Item = (K::OutputRef<'a>, V::OutputRefMut<'a>);
    fn next(&mut self) -> Option<Self::Item> {
        if !self.beg.equal(&self.end) {
            unsafe {
                let key = self.beg.as_deref().as_key();
                let val = self.beg.as_deref_mut().as_value();
                self.beg.next();
                return Some((key, val));
            }
        }
        None
    }
}

hicc::import_class! {
    #[cpp(class = "template <class K, class V, class Hash, class Pred, class Allocator> std::unordered_multimap<K, V, Hash, Pred, Allocator>")]
    pub class unordered_multimap<K, V> {
        hicc::cpp! {
            typedef typename Self::iterator iterator;
            typedef typename Self::const_iterator const_iterator;
        }
        /// ```
        /// use hicc_std::UnorderedMultiMapIntInt;
        /// let map = UnorderedMultiMapIntInt::new();
        /// assert!(map.is_empty());
        /// ```
        #[cpp(method = "bool empty() const")]
        pub fn is_empty(&self) -> bool;

        /// ```
        /// use hicc_std::UnorderedMultiMapIntInt;
        /// let map = UnorderedMultiMapIntInt::new();
        /// assert_eq!(map.size(), 0_usize);
        /// ```
        #[cpp(method = "size_t size() const")]
        pub fn size(&self) -> usize;

        /// ```
        /// use hicc_std::UnorderedMultiMapIntInt;
        /// let map = UnorderedMultiMapIntInt::new();
        /// println!("map.max_size() = {}", map.max_size());
        /// ```
        #[cpp(method = "size_t max_size() const")]
        pub fn max_size(&self) -> usize;

        /// ```
        /// use hicc_std::UnorderedMultiMapIntInt;
        /// let mut map = UnorderedMultiMapIntInt::new();
        /// map.insert(&1, &2);
        /// assert_eq!(map.size(), 1_usize);
        /// map.clear();
        /// assert_eq!(map.size(), 0_usize);
        /// ```
        #[cpp(method = "void clear()")]
        pub fn clear(&mut self);

        /// ```
        /// use hicc_std::UnorderedMultiMapIntInt;
        /// let mut map1 = UnorderedMultiMapIntInt::new();
        /// map1.insert(&1, &2);
        /// let mut map2 = UnorderedMultiMapIntInt::new();
        /// map2.swap(&mut map1);
        /// assert_eq!(map1.size(), 0_usize);
        /// assert_eq!(map2.size(), 1_usize);
        /// ```
        #[cpp(method = "void swap(Self&)")]
        pub fn swap(&mut self, other: &mut Self);

        /// ```
        /// use hicc_std::UnorderedMultiMapIntInt;
        /// let mut map = UnorderedMultiMapIntInt::new();
        /// map.insert(&1, &2);
        /// map.insert(&1, &2);
        /// assert_eq!(map.count(&1), 2_usize);
        /// assert_eq!(map.count(&2), 0);
        /// ```
        #[cpp(method = "size_t count(const K&) const")]
        pub fn count(&self, key: &K) -> usize;

        hicc::cpp! {
            static bool contains(const Self& self, const K& key) {
                return self.find(key) != self.end();
            }
        }
        /// ```
        /// use hicc_std::UnorderedMultiMapIntInt;
        /// let mut map = UnorderedMultiMapIntInt::new();
        /// map.insert(&1, &2);
        /// assert!(map.contains(&1));
        /// assert!(!map.contains(&2));
        /// ```
        #[cpp(func = "bool SelfMethods::contains(const Self&, const K&)")]
        pub fn contains(&self, key: &K) -> bool;


        /// ```
        /// use hicc_std::UnorderedMultiMapIntInt;
        /// let mut map = UnorderedMultiMapIntInt::new();
        /// map.insert(&1, &2);
        /// map.assign(&mut UnorderedMultiMapIntInt::new());
        /// assert!(map.is_empty());
        /// ```
        #[cpp(func = "void hicc::make_assign<Self, Self>(Self&, const Self&)")]
        pub fn assign(&mut self, other: &Self);

        hicc::cpp! {
            static void insert(Self& self, const K& key, const V& val) {
                self.insert(std::make_pair(key, val));
            }
        }
        /// ```
        /// use hicc_std::UnorderedMultiMapIntInt;
        /// let mut map = UnorderedMultiMapIntInt::new();
        /// map.insert(&1, &2);
        /// map.insert(&1, &3);
        /// assert_eq!(map.count(&1), 2);
        /// ```
        #[cpp(func = "void SelfMethods::insert(Self&, const K&, const V&)")]
        pub fn insert(&mut self, key: &K, val: &V);

        /// ```
        /// use hicc_std::UnorderedMultiMapIntInt;
        /// let mut map = UnorderedMultiMapIntInt::new();
        /// map.insert(&1, &2);
        /// map.insert(&1, &3);
        /// assert_eq!(map.erase(&1), 2);
        /// ```
        #[cpp(method = "size_t erase(const K&)")]
        pub fn erase(&mut self, key: &K) -> usize;

        #[cpp(method = "const_iterator find(const K&) const")]
        unsafe fn find(&self, key: &K) -> *mut CppUnorderedMultiMapIter<K, V>;
        #[cpp(method = "iterator find(const K&)")]
        unsafe fn find_mut(&mut self, key: &K) -> *mut CppUnorderedMultiMapIterMut<K, V>;
        #[cpp(method = "const_iterator begin() const")]
        unsafe fn begin(&self) -> *mut CppUnorderedMultiMapIter<K, V>;
        #[cpp(method = "iterator begin()")]
        unsafe fn begin_mut(&mut self) -> *mut CppUnorderedMultiMapIterMut<K, V>;
        #[cpp(method = "const_iterator end() const")]
        unsafe fn end(&self) -> *mut CppUnorderedMultiMapIter<K, V>;
        #[cpp(method = "iterator end()")]
        unsafe fn end_mut(&mut self) -> *mut CppUnorderedMultiMapIterMut<K, V>;
    }

    unsafe impl<K: AbiType + Sync, V: AbiType + Sync> Send for unordered_multimap<K, V> {}
    unsafe impl<K: AbiType + Sync, V: AbiType + Sync> Sync for unordered_multimap<K, V> {}

    #[cpp(class = "template <class K, class V, class Hash, class Pred, class Allocator> std::unordered_multimap<K, V, Hash, Pred, Allocator>::const_iterator")]
    class CppUnorderedMultiMapIter<K, V> {
        hicc::cpp! {
            static void next(Self& self) {
                ++self;
            }
            static const K& key(const Self& self) {
                return self->first;
            }
            static const V& value(const Self& self) {
                return self->second;
            }
        }
        #[cpp(func = "void SelfMethods::next(Self&)")]
        fn next(&mut self);
        #[cpp(func = "const K& SelfMethods::key(const Self&)")]
        fn as_key(&self) -> &K;
        #[cpp(func = "const V& SelfMethods::value(const Self&)")]
        fn as_value(&self) -> &V;
        #[cpp(func = "bool hicc::make_eq<Self, Self>(const Self&, const Self&)")]
        fn equal(&self, other: &Self) -> bool;
    }

    #[cpp(class = "template <class K, class V, class Hash, class Pred, class Allocator> std::unordered_multimap<K, V, Hash, Pred, Allocator>::iterator")]
    class CppUnorderedMultiMapIterMut<K, V> {
        hicc::cpp! {
            static void next(Self& self) {
                ++self;
            }
            static const K& key(const Self& self) {
                return self->first;
            }
            static V& value(Self& self) {
                return self->second;
            }
        }
        #[cpp(func = "void SelfMethods::next(Self&)")]
        fn next(&mut self);
        #[cpp(func = "const K& SelfMethods::key(const Self&)")]
        fn as_key(&self) -> &K;
        #[cpp(func = "V& SelfMethods::value(Self&)")]
        fn as_value(&mut self) -> &mut V;
        #[cpp(func = "bool hicc::make_eq<Self, Self>(const Self&, const Self&)")]
        fn equal(&self, other: &Self) -> bool;
    }
}

impl<K: AbiType, V: AbiType> unordered_multimap<K, V> {
    /// ```
    /// use hicc_std::UnorderedMultiMapIntInt;
    /// let mut map = UnorderedMultiMapIntInt::new();
    /// map.insert(&1, &2);
    /// assert!(map.get(&2).is_none());
    /// assert_eq!(map.get(&1), Some(&2));
    /// ```
    pub fn get(&self, key: &K::InputType) -> Option<V::OutputRef<'_>> {
        unsafe {
            let it = self.find(key);
            if !it.equal(&self.end()) {
                return Some(it.as_deref().as_value());
            }
        }
        None
    }
    /// ```
    /// use hicc_std::UnorderedMultiMapIntInt;
    /// let mut map = UnorderedMultiMapIntInt::new();
    /// map.insert(&1, &2);
    /// *map.get_mut(&1).unwrap() = 0;
    /// assert_eq!(map.get(&1), Some(&0));
    ///
    /// use hicc_std::{string, UnorderedMultiMapIntString};
    /// use hicc::AbiClass;
    /// let mut map = UnorderedMultiMapIntString::new();
    /// map.insert(&1, &string::from(c"hello"));
    /// map.get_mut(&1).unwrap().write(string::from(c"world"));
    /// assert!(*map.get(&1).unwrap() == string::from(c"world"));
    /// ```
    pub fn get_mut(&mut self, key: &K::InputType) -> Option<V::OutputRefMut<'_>> {
        unsafe {
            let mut it = self.find_mut(key);
            if !it.equal(&self.end_mut()) {
                return Some(it.as_deref_mut().as_value());
            }
        }
        None
    }

    /// ```
    /// use hicc_std::UnorderedMultiMapIntInt;
    /// let mut map = UnorderedMultiMapIntInt::new();
    /// map.insert(&1, &2);
    /// map.insert(&1, &1);
    /// map.iter().for_each(|(k, v)| println!("key = {k}, value = {v}"));
    /// ```
    pub fn iter(&self) -> impl Iterator<Item = (K::OutputRef<'_>, V::OutputRef<'_>)> {
        UnorderedMultiMapIter {
            beg: unsafe { self.begin() },
            end: unsafe { self.end() },
            mark: PhantomData,
        }
    }

    /// ```
    /// use hicc_std::UnorderedMultiMapIntInt;
    /// let mut map = UnorderedMultiMapIntInt::new();
    /// map.insert(&1, &2);
    /// map.insert(&2, &1);
    /// map.iter_mut().for_each(|(_, v)| *v += 1);
    /// assert_eq!(map.get(&1), Some(&3));
    /// assert_eq!(map.get(&2), Some(&2));
    /// ```
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (K::OutputRef<'_>, V::OutputRefMut<'_>)> {
        let beg = unsafe { self.begin_mut() };
        let end = unsafe { self.end_mut() };
        UnorderedMultiMapIterMut {
            beg,
            end,
            mark: PhantomData,
        }
    }
}

/// 对应`std::unordered_multimap<K, V>::const_iterator`
struct UnorderedMultiMapIter<'a, K: AbiType + 'static, V: AbiType + 'static> {
    beg: ClassMutPtr<'static, CppUnorderedMultiMapIter<K, V>>,
    end: ClassMutPtr<'static, CppUnorderedMultiMapIter<K, V>>,
    mark: PhantomData<&'a unordered_multimap<K, V>>,
}

impl<'a, K: AbiType + 'static, V: AbiType + 'static> Iterator for UnorderedMultiMapIter<'a, K, V> {
    type Item = (K::OutputRef<'a>, V::OutputRef<'a>);
    fn next(&mut self) -> Option<Self::Item> {
        if !self.beg.equal(&self.end) {
            unsafe {
                let key = self.beg.as_deref().as_key();
                let val = self.beg.as_deref().as_value();
                self.beg.next();
                return Some((key, val));
            }
        }
        None
    }
}
/// 对应`std::unordered_multimap<K, V>::iterator`
struct UnorderedMultiMapIterMut<'a, K: AbiType + 'static, V: AbiType + 'static> {
    beg: ClassMutPtr<'static, CppUnorderedMultiMapIterMut<K, V>>,
    end: ClassMutPtr<'static, CppUnorderedMultiMapIterMut<K, V>>,
    mark: PhantomData<&'a unordered_multimap<K, V>>,
}

impl<'a, K: AbiType + 'static, V: AbiType + 'static> Iterator
    for UnorderedMultiMapIterMut<'a, K, V>
{
    type Item = (K::OutputRef<'a>, V::OutputRefMut<'a>);
    fn next(&mut self) -> Option<Self::Item> {
        if !self.beg.equal(&self.end) {
            unsafe {
                let key = self.beg.as_deref().as_key();
                let val = self.beg.as_deref_mut().as_value();
                self.beg.next();
                return Some((key, val));
            }
        }
        None
    }
}
