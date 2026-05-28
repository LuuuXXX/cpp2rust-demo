use hicc::{AbiType, ClassMutPtr};
use std::iter::Iterator;
use std::marker::PhantomData;

hicc::cpp! {
    #include <map>
}

hicc::import_class! {
    #[cpp(class = "template<class K, class V, class Compare, class Allocator> std::map<K, V, Compare, Allocator>")]
    pub class map<K, V> {
        hicc::cpp! {
            typedef typename Self::iterator iterator;
            typedef typename Self::reverse_iterator reverse_iterator;
            typedef typename Self::const_iterator const_iterator;
            typedef typename Self::const_reverse_iterator const_reverse_iterator;
        }
        /// ```
        /// use hicc_std::MapIntInt;
        /// let map = MapIntInt::new();
        /// assert!(map.is_empty());
        /// ```
        #[cpp(method = "bool empty() const")]
        pub fn is_empty(&self) -> bool;

        /// ```
        /// use hicc_std::MapIntInt;
        /// let map = MapIntInt::new();
        /// assert_eq!(map.size(), 0_usize);
        /// ```
        #[cpp(method = "size_t size() const")]
        pub fn size(&self) -> usize;

        /// ```
        /// use hicc_std::MapIntInt;
        /// let map = MapIntInt::new();
        /// println!("map.max_size() = {}", map.max_size());
        /// ```
        #[cpp(method = "size_t max_size() const")]
        pub fn max_size(&self) -> usize;

        /// ```
        /// use hicc_std::MapIntInt;
        /// let mut map = MapIntInt::new();
        /// map.insert(&1, &2);
        /// assert_eq!(map.size(), 1_usize);
        /// map.clear();
        /// assert_eq!(map.size(), 0_usize);
        /// ```
        #[cpp(method = "void clear()")]
        pub fn clear(&mut self);

        /// ```
        /// use hicc_std::MapIntInt;
        /// let mut map1 = MapIntInt::new();
        /// map1.insert(&1, &2);
        /// let mut map2 = MapIntInt::new();
        /// map2.swap(&mut map1);
        /// assert_eq!(map1.size(), 0_usize);
        /// assert_eq!(map2.size(), 1_usize);
        /// ```
        #[cpp(method = "void swap(Self&)")]
        pub fn swap(&mut self, other: &mut Self);

        /// ```
        /// use hicc_std::MapIntInt;
        /// let mut map = MapIntInt::new();
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
        /// use hicc_std::MapIntInt;
        /// let mut map = MapIntInt::new();
        /// map.insert(&1, &2);
        /// assert!(map.contains(&1));
        /// assert!(!map.contains(&2));
        /// ```
        #[cpp(func = "bool SelfMethods::contains(const Self&, const K&)")]
        pub fn contains(&self, key: &K) -> bool;

        /// ```
        /// use hicc_std::MapIntInt;
        /// let mut map = MapIntInt::new();
        /// map.insert(&1, &2);
        /// map.assign(&mut MapIntInt::new());
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
        /// use hicc_std::MapIntInt;
        /// let mut map = MapIntInt::new();
        /// assert!(map.insert(&1, &2));
        /// assert_eq!(map.get(&1), Some(&2));
        /// assert!(!map.insert(&1, &2));
        /// ```
        #[cpp(func = "bool SelfMethods::insert(Self&, const K&, const V&)")]
        pub fn insert(&mut self, key: &K, val: &V) -> bool;

        /// ```
        /// use hicc_std::MapIntInt;
        /// let mut map = MapIntInt::new();
        /// assert_eq!(map.erase(&1), 0_usize);
        /// assert!(map.insert(&1, &2));
        /// assert_eq!(map.erase(&1), 1_usize);
        /// ```
        #[cpp(method = "size_t erase(const K&)")]
        pub fn erase(&mut self, key: &K) -> usize;

        #[cpp(method = "const_iterator find(const K&) const")]
        unsafe fn find(&self, key: &K) -> *mut CppMapIter<K, V>;
        #[cpp(method = "iterator find(const K&)")]
        unsafe fn find_mut(&mut self, key: &K) -> *mut CppMapIterMut<K, V>;
        #[cpp(method = "const_iterator lower_bound(const K&) const")]
        unsafe fn lower_bound(&self, key: &K) -> *mut CppMapIter<K, V>;
        #[cpp(method = "iterator lower_bound(const K&)")]
        unsafe fn lower_bound_mut(&mut self, key: &K) -> *mut CppMapIterMut<K, V>;
        #[cpp(method = "const_iterator upper_bound(const K&) const")]
        unsafe fn upper_bound(&self, key: &K) -> *mut CppMapIter<K, V>;
        #[cpp(method = "iterator upper_bound(const K&)")]
        unsafe fn upper_bound_mut(&mut self, key: &K) -> *mut CppMapIterMut<K, V>;

        #[cpp(method = "const_iterator begin() const")]
        unsafe fn begin(&self) -> *mut CppMapIter<K, V>;
        #[cpp(method = "iterator begin()")]
        unsafe fn begin_mut(&mut self) -> *mut CppMapIterMut<K, V>;
        #[cpp(method = "const_iterator end() const")]
        unsafe fn end(&self) -> *mut CppMapIter<K, V>;
        #[cpp(method = "iterator end()")]
        unsafe fn end_mut(&mut self) -> *mut CppMapIterMut<K, V>;
        #[cpp(method = "const_reverse_iterator rbegin() const")]
        unsafe fn rbegin(&self) -> *mut CppMapRevIter<K, V>;
        #[cpp(method = "reverse_iterator rbegin()")]
        unsafe fn rbegin_mut(&mut self) -> *mut CppMapRevIterMut<K, V>;
        #[cpp(method = "const_reverse_iterator rend() const")]
        unsafe fn rend(&self) -> *mut CppMapRevIter<K, V>;
        #[cpp(method = "reverse_iterator rend()")]
        unsafe fn rend_mut(&mut self) -> *mut CppMapRevIterMut<K, V>;
    }

    unsafe impl<K: AbiType + Sync, V: AbiType + Sync> Send for map<K, V> {}
    unsafe impl<K: AbiType + Sync, V: AbiType + Sync> Sync for map<K, V> {}

    #[cpp(class = "template<class K, class V, class Compare, class Allocator> std::map<K, V, Compare, Allocator>::const_iterator")]
    class CppMapIter<K, V> {
        hicc::cpp! {
            typedef typename SelfContainer::const_reverse_iterator const_reverse_iterator;
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
        unsafe fn next(&mut self);
        #[cpp(func = "const K& SelfMethods::key(const Self&)")]
        unsafe fn as_key(&self) -> &K;
        #[cpp(func = "const V& SelfMethods::value(const Self&)")]
        unsafe fn as_value(&self) -> &V;
        #[cpp(func = "bool hicc::make_eq<Self, Self>(const Self&, const Self&)")]
        fn equal(&self, other: &Self) -> bool;
        #[cpp(func = "const_reverse_iterator hicc::make_constructor<const_reverse_iterator, Self>(Self&&)")]
        fn into_reverse(self) -> *mut CppMapRevIter<K, V>;
    }

    #[cpp(class = "template<class K, class V, class Compare, class Allocator> std::map<K, V, Compare, Allocator>::iterator")]
    class CppMapIterMut<K, V> {
        hicc::cpp! {
            typedef typename SelfContainer::reverse_iterator reverse_iterator;
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
        unsafe fn next(&mut self);
        #[cpp(func = "const K& SelfMethods::key(const Self&)")]
        unsafe fn as_key(&self) -> &K;
        #[cpp(func = "V& SelfMethods::value(Self&)")]
        unsafe fn as_value(&mut self) -> &mut V;
        #[cpp(func = "bool hicc::make_eq<Self, Self>(const Self&, const Self&)")]
        fn equal(&self, other: &Self) -> bool;
        #[cpp(func = "reverse_iterator hicc::make_constructor<reverse_iterator, Self>(Self&&)")]
        fn into_reverse(self) -> *mut CppMapRevIterMut<K, V>;
    }

    #[cpp(class = "template<class K, class V, class Compare, class Allocator> std::map<K, V, Compare, Allocator>::const_reverse_iterator")]
    class CppMapRevIter<K, V> {
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
        unsafe fn next(&mut self);
        #[cpp(func = "const K& SelfMethods::key(const Self&)")]
        unsafe fn as_key(&self) -> &K;
        #[cpp(func = "const V& SelfMethods::value(const Self&)")]
        unsafe fn as_value(&self) -> &V;
        #[cpp(func = "bool hicc::make_eq<Self, Self>(const Self&, const Self&)")]
        fn equal(&self, other: &Self) -> bool;
    }

    #[cpp(class = "template<class K, class V, class Compare, class Allocator> std::map<K, V, Compare, Allocator>::reverse_iterator")]
    class CppMapRevIterMut<K, V> {
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
        unsafe fn next(&mut self);
        #[cpp(func = "const K& SelfMethods::key(const Self&)")]
        unsafe fn as_key(&self) -> &K;
        #[cpp(func = "V& SelfMethods::value(Self&)")]
        unsafe fn as_value(&mut self) -> &mut V;
        #[cpp(func = "bool hicc::make_eq<Self, Self>(const Self&, const Self&)")]
        fn equal(&self, other: &Self) -> bool;
    }
}

impl<K: AbiType, V: AbiType> map<K, V> {
    /// ```
    /// use hicc_std::MapIntInt;
    /// let mut map = MapIntInt::new();
    /// map.insert(&1, &2);
    /// assert!(map.get(&1).is_some());
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
    /// use hicc_std::MapIntInt;
    /// let mut map = MapIntInt::new();
    /// map.insert(&1, &2);
    /// assert!(map.get(&2).is_none());
    /// assert_eq!(map.get(&1), Some(&2));
    ///
    /// use hicc_std::{string, MapIntString};
    /// use hicc::AbiClass;
    /// let mut map = MapIntString::new();
    /// map.insert(&1, &string::from(c"hello"));
    /// assert!(*map.get(&1).unwrap() == string::from(c"hello"));
    /// map.get_mut(&1).unwrap().write(string::from(c"world"));
    /// assert!(*map.get(&1).unwrap() == string::from(c"world"));
    /// ```
    pub fn get_mut(&mut self, key: &K::InputType) -> Option<V::OutputRefMut<'_>> {
        unsafe {
            let end = self.end_mut().into_value();
            let mut it = self.find_mut(key);
            if !it.equal(&end) {
                return Some(it.as_deref_mut().as_value());
            }
        }
        None
    }

    /// ```
    /// use hicc_std::MapIntInt;
    /// let mut map = MapIntInt::new();
    /// map.insert(&1, &2);
    /// map.insert(&2, &3);
    /// let mut it = map.iter();
    /// assert_eq!(it.next(), Some((&1, &2)));
    /// assert_eq!(it.next(), Some((&2, &3)));
    /// assert!(it.next().is_none());
    /// ```
    pub fn iter(&self) -> impl Iterator<Item = (K::OutputRef<'_>, V::OutputRef<'_>)> {
        MapIter {
            beg: unsafe { self.begin() },
            end: unsafe { self.end() },
            mark: PhantomData,
        }
    }

    /// ```
    /// use hicc_std::MapIntInt;
    /// let mut map = MapIntInt::new();
    /// map.insert(&1, &2);
    /// map.insert(&2, &3);
    /// map.iter_mut().for_each(|(_, v)| *v += 1);
    /// let mut it = map.iter();
    /// assert_eq!(it.next(), Some((&1, &3)));
    /// assert_eq!(it.next(), Some((&2, &4)));
    /// assert!(it.next().is_none());
    /// ```
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (K::OutputRef<'_>, V::OutputRefMut<'_>)> {
        let beg = unsafe { self.begin_mut() };
        let end = unsafe { self.end_mut() };
        MapIterMut {
            beg,
            end,
            mark: PhantomData,
        }
    }

    /// ```
    /// use hicc_std::MapIntInt;
    /// let mut map = MapIntInt::new();
    /// map.insert(&1, &2);
    /// map.insert(&2, &3);
    /// let mut it = map.rev_iter();
    /// assert_eq!(it.next(), Some((&2, &3)));
    /// assert_eq!(it.next(), Some((&1, &2)));
    /// assert!(it.next().is_none());
    /// ```
    pub fn rev_iter(&self) -> impl Iterator<Item = (K::OutputRef<'_>, V::OutputRef<'_>)> {
        MapRevIter {
            beg: unsafe { self.rbegin() },
            end: unsafe { self.rend() },
            mark: PhantomData,
        }
    }

    /// ```
    /// use hicc_std::MapIntInt;
    /// let mut map = MapIntInt::new();
    /// map.insert(&1, &2);
    /// map.insert(&2, &3);
    /// map.rev_iter_mut().for_each(|(_, v)| *v += 1);
    /// let mut it = map.rev_iter();
    /// assert_eq!(it.next(), Some((&2, &4)));
    /// assert_eq!(it.next(), Some((&1, &3)));
    /// assert!(it.next().is_none());
    /// ```
    pub fn rev_iter_mut(
        &mut self,
    ) -> impl Iterator<Item = (K::OutputRef<'_>, V::OutputRefMut<'_>)> {
        let beg = unsafe { self.rbegin_mut() };
        let end = unsafe { self.rend_mut() };
        MapRevIterMut {
            beg,
            end,
            mark: PhantomData,
        }
    }

    /// ```
    /// use hicc_std::MapIntInt;
    /// let mut map = MapIntInt::new();
    /// map.insert(&1, &2);
    /// map.insert(&2, &3);
    /// map.insert(&3, &4);
    /// let mut it = map.iter_lower_upper_bound(Some(&1), Some(&2));
    /// assert_eq!(it.next(), Some((&1, &2)));
    /// assert_eq!(it.next(), Some((&2, &3)));
    /// assert_eq!(it.next(), None);
    /// ```
    pub fn iter_lower_upper_bound(
        &self,
        lower_key: Option<&K::InputType>,
        upper_key: Option<&K::InputType>,
    ) -> impl Iterator<Item = (K::OutputRef<'_>, V::OutputRef<'_>)> {
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
        MapIter {
            beg,
            end,
            mark: PhantomData,
        }
    }

    /// ```
    /// use hicc_std::MapIntInt;
    /// let mut map = MapIntInt::new();
    /// map.insert(&1, &2);
    /// map.insert(&2, &3);
    /// map.insert(&3, &4);
    /// map.iter_lower_upper_bound_mut(Some(&1), Some(&2)).for_each(|(_, v)| *v -= 1);
    /// let mut it = map.iter_lower_upper_bound(Some(&1), Some(&2));
    /// assert_eq!(it.next(), Some((&1, &1)));
    /// assert_eq!(it.next(), Some((&2, &2)));
    /// assert_eq!(it.next(), None);
    /// ```
    pub fn iter_lower_upper_bound_mut(
        &mut self,
        lower_key: Option<&K::InputType>,
        upper_key: Option<&K::InputType>,
    ) -> impl Iterator<Item = (K::OutputRef<'_>, V::OutputRefMut<'_>)> {
        let beg = if let Some(key) = lower_key {
            unsafe { self.lower_bound_mut(key) }
        } else {
            unsafe { self.begin_mut() }
        };
        let end = if let Some(key) = upper_key {
            unsafe { self.upper_bound_mut(key) }
        } else {
            unsafe { self.end_mut() }
        };
        MapIterMut {
            beg,
            end,
            mark: PhantomData,
        }
    }

    /// ```
    /// use hicc_std::MapIntInt;
    /// let mut map = MapIntInt::new();
    /// map.insert(&1, &0);
    /// map.insert(&2, &1);
    /// map.insert(&3, &2);
    /// let mut it = map.rev_iter_lower_upper_bound(Some(&1), Some(&2));
    /// assert_eq!(it.next(), Some((&2, &1)));
    /// assert_eq!(it.next(), Some((&1, &0)));
    /// assert_eq!(it.next(), None);
    /// ```
    pub fn rev_iter_lower_upper_bound(
        &self,
        lower_key: Option<&K::InputType>,
        upper_key: Option<&K::InputType>,
    ) -> impl Iterator<Item = (K::OutputRef<'_>, V::OutputRef<'_>)> {
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
        MapRevIter {
            beg,
            end,
            mark: PhantomData,
        }
    }

    /// ```
    /// use hicc_std::MapIntInt;
    /// let mut map = MapIntInt::new();
    /// map.insert(&1, &0);
    /// map.insert(&2, &1);
    /// map.insert(&3, &2);
    /// map.rev_iter_lower_upper_bound_mut(Some(&1), Some(&2)).for_each(|(_, v)| *v -= 1);
    /// let mut it = map.rev_iter_lower_upper_bound(Some(&1), Some(&2));
    /// assert_eq!(it.next(), Some((&2, &0)));
    /// assert_eq!(it.next(), Some((&1, &-1)));
    /// assert_eq!(it.next(), None);
    /// ```
    pub fn rev_iter_lower_upper_bound_mut(
        &mut self,
        lower_key: Option<&K::InputType>,
        upper_key: Option<&K::InputType>,
    ) -> impl Iterator<Item = (K::OutputRef<'_>, V::OutputRefMut<'_>)> {
        let beg = if let Some(key) = upper_key {
            unsafe { self.upper_bound_mut(key).into_value() }.into_reverse()
        } else {
            unsafe { self.rbegin_mut() }
        };
        let end = if let Some(key) = lower_key {
            unsafe { self.lower_bound_mut(key).into_value() }.into_reverse()
        } else {
            unsafe { self.rend_mut() }
        };
        MapRevIterMut {
            beg,
            end,
            mark: PhantomData,
        }
    }
}

/// 对应`std::map<K, V>::const_iterator`
struct MapIter<'a, K: AbiType + 'static, V: AbiType + 'static> {
    beg: ClassMutPtr<'static, CppMapIter<K, V>>,
    end: ClassMutPtr<'static, CppMapIter<K, V>>,
    mark: PhantomData<&'a map<K, V>>,
}

impl<'a, K: AbiType + 'static, V: AbiType + 'static> Iterator for MapIter<'a, K, V> {
    type Item = (K::OutputRef<'a>, V::OutputRef<'a>);
    fn next(&mut self) -> Option<Self::Item> {
        if self.beg.equal(&self.end) {
            return None;
        }
        let key = unsafe { self.beg.as_deref().as_key() };
        let val = unsafe { self.beg.as_deref().as_value() };
        unsafe { self.beg.next() };
        Some((key, val))
    }
}

/// 对应`std::map<K, V>::iterator`
struct MapIterMut<'a, K: AbiType + 'static, V: AbiType + 'static> {
    beg: ClassMutPtr<'static, CppMapIterMut<K, V>>,
    end: ClassMutPtr<'static, CppMapIterMut<K, V>>,
    mark: PhantomData<&'a mut map<K, V>>,
}

impl<'a, K: AbiType + 'static, V: AbiType + 'static> Iterator for MapIterMut<'a, K, V> {
    type Item = (K::OutputRef<'a>, V::OutputRefMut<'a>);
    fn next(&mut self) -> Option<Self::Item> {
        if self.beg.equal(&self.end) {
            return None;
        }
        let key = unsafe { self.beg.as_deref().as_key() };
        let val = unsafe { self.beg.as_deref_mut().as_value() };
        unsafe { self.beg.next() };
        Some((key, val))
    }
}

/// 对应`std::map<K, V>::const_reverse_iterator`
struct MapRevIter<'a, K: AbiType + 'static, V: AbiType + 'static> {
    beg: ClassMutPtr<'static, CppMapRevIter<K, V>>,
    end: ClassMutPtr<'static, CppMapRevIter<K, V>>,
    mark: PhantomData<&'a map<K, V>>,
}

impl<'a, K: AbiType + 'static, V: AbiType + 'static> Iterator for MapRevIter<'a, K, V> {
    type Item = (K::OutputRef<'a>, V::OutputRef<'a>);
    fn next(&mut self) -> Option<Self::Item> {
        if self.beg.equal(&self.end) {
            return None;
        }
        let key = unsafe { self.beg.as_deref().as_key() };
        let val = unsafe { self.beg.as_deref().as_value() };
        unsafe { self.beg.next() };
        Some((key, val))
    }
}

/// 对应`std::map<K, V>::reverse_iterator`
struct MapRevIterMut<'a, K: AbiType + 'static, V: AbiType + 'static> {
    beg: ClassMutPtr<'static, CppMapRevIterMut<K, V>>,
    end: ClassMutPtr<'static, CppMapRevIterMut<K, V>>,
    mark: PhantomData<&'a mut map<K, V>>,
}

impl<'a, K: AbiType + 'static, V: AbiType + 'static> Iterator for MapRevIterMut<'a, K, V> {
    type Item = (K::OutputRef<'a>, V::OutputRefMut<'a>);
    fn next(&mut self) -> Option<Self::Item> {
        if self.beg.equal(&self.end) {
            return None;
        }
        let key = unsafe { self.beg.as_deref().as_key() };
        let val = unsafe { self.beg.as_deref_mut().as_value() };
        unsafe { self.beg.next() };
        Some((key, val))
    }
}

hicc::import_class! {
    #[cpp(class = "template<class K, class V, class Compare, class Allocator> std::multimap<K, V, Compare, Allocator>")]
    pub class multimap<K, V> {
        hicc::cpp! {
            typedef typename Self::iterator iterator;
            typedef typename Self::const_iterator const_iterator;
            typedef typename Self::reverse_iterator reverse_iterator;
            typedef typename Self::const_reverse_iterator const_reverse_iterator;
        }
        /// ```
        /// use hicc_std::MultiMapIntInt;
        /// let map = MultiMapIntInt::new();
        /// assert!(map.is_empty());
        /// ```
        #[cpp(method = "bool empty() const")]
        pub fn is_empty(&self) -> bool;

        /// ```
        /// use hicc_std::MultiMapIntInt;
        /// let map = MultiMapIntInt::new();
        /// assert_eq!(map.size(), 0_usize);
        /// ```
        #[cpp(method = "size_t size() const")]
        pub fn size(&self) -> usize;

        /// ```
        /// use hicc_std::MultiMapIntInt;
        /// let map = MultiMapIntInt::new();
        /// println!("map.max_size() = {}", map.max_size());
        /// ```
        #[cpp(method = "size_t max_size() const")]
        pub fn max_size(&self) -> usize;

        /// ```
        /// use hicc_std::MultiMapIntInt;
        /// let mut map = MultiMapIntInt::new();
        /// map.insert(&1, &2);
        /// assert_eq!(map.size(), 1_usize);
        /// map.clear();
        /// assert_eq!(map.size(), 0_usize);
        /// ```
        #[cpp(method = "void clear()")]
        pub fn clear(&mut self);

        /// ```
        /// use hicc_std::MultiMapIntInt;
        /// let mut map1 = MultiMapIntInt::new();
        /// map1.insert(&1, &2);
        /// let mut map2 = MultiMapIntInt::new();
        /// map2.swap(&mut map1);
        /// assert_eq!(map1.size(), 0_usize);
        /// assert_eq!(map2.size(), 1_usize);
        /// ```
        #[cpp(method = "void swap(Self&)")]
        pub fn swap(&mut self, other: &mut Self);

        /// ```
        /// use hicc_std::MultiMapIntInt;
        /// let mut map = MultiMapIntInt::new();
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
        /// use hicc_std::MultiMapIntInt;
        /// let mut map = MultiMapIntInt::new();
        /// map.insert(&1, &2);
        /// assert!(map.contains(&1));
        /// assert!(!map.contains(&2));
        /// ```
        #[cpp(func = "bool SelfMethods::contains(const Self&, const K&)")]
        pub fn contains(&self, key: &K) -> bool;

        /// ```
        /// use hicc_std::MultiMapIntInt;
        /// let mut map = MultiMapIntInt::new();
        /// map.insert(&1, &2);
        /// map.assign(&mut MultiMapIntInt::new());
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
        /// use hicc_std::MultiMapIntInt;
        /// let mut map = MultiMapIntInt::new();
        /// map.insert(&1, &2);
        /// map.insert(&1, &3);
        /// assert_eq!(map.count(&1), 2);
        /// ```
        #[cpp(func = "void SelfMethods::insert(Self&, const K&, const V&)")]
        pub fn insert(&mut self, key: &K, val: &V);

        /// ```
        /// use hicc_std::MultiMapIntInt;
        /// let mut map = MultiMapIntInt::new();
        /// map.insert(&1, &2);
        /// map.insert(&1, &3);
        /// assert_eq!(map.erase(&1), 2);
        /// ```
        #[cpp(method = "size_t erase(const K&)")]
        pub fn erase(&mut self, key: &K) -> usize;

        #[cpp(method = "const_iterator find(const K&) const")]
        unsafe fn find(&self, key: &K) -> *mut CppMultiMapIter<K, V>;
        #[cpp(method = "iterator find(const K&)")]
        unsafe fn find_mut(&mut self, key: &K) -> *mut CppMultiMapIterMut<K, V>;
        #[cpp(method = "const_iterator lower_bound(const K&) const")]
        unsafe fn lower_bound(&self, key: &K) -> *mut CppMultiMapIter<K, V>;
        #[cpp(method = "iterator lower_bound(const K&)")]
        unsafe fn lower_bound_mut(&mut self, key: &K) -> *mut CppMultiMapIterMut<K, V>;
        #[cpp(method = "const_iterator upper_bound(const K&) const")]
        unsafe fn upper_bound(&self, key: &K) -> *mut CppMultiMapIter<K, V>;
        #[cpp(method = "iterator upper_bound(const K&)")]
        unsafe fn upper_bound_mut(&mut self, key: &K) -> *mut CppMultiMapIterMut<K, V>;
        #[cpp(method = "const_iterator begin() const")]
        unsafe fn begin(&self) -> *mut CppMultiMapIter<K, V>;
        #[cpp(method = "iterator begin() ")]
        unsafe fn begin_mut(&mut self) -> *mut CppMultiMapIterMut<K, V>;
        #[cpp(method = "const_iterator end() const")]
        unsafe fn end(&self) -> *mut CppMultiMapIter<K, V>;
        #[cpp(method = "iterator end() ")]
        unsafe fn end_mut(&mut self) -> *mut CppMultiMapIterMut<K, V>;
        #[cpp(method = "const_reverse_iterator rbegin() const")]
        unsafe fn rbegin(&self) -> *mut CppMultiMapRevIter<K, V>;
        #[cpp(method = "reverse_iterator rbegin()")]
        unsafe fn rbegin_mut(&mut self) -> *mut CppMultiMapRevIterMut<K, V>;
        #[cpp(method = "const_reverse_iterator rend() const")]
        unsafe fn rend(&self) -> *mut CppMultiMapRevIter<K, V>;
        #[cpp(method = "reverse_iterator rend()")]
        unsafe fn rend_mut(&mut self) -> *mut CppMultiMapRevIterMut<K, V>;
    }

    unsafe impl<K: AbiType + Sync, V: AbiType + Sync> Send for multimap<K, V> {}
    unsafe impl<K: AbiType + Sync, V: AbiType + Sync> Sync for multimap<K, V> {}

    #[cpp(class = "template<class K, class V, class Compare, class Allocator> std::multimap<K, V, Compare, Allocator>::const_iterator")]
    class CppMultiMapIter<K, V> {
        hicc::cpp! {
            typedef typename SelfContainer::const_reverse_iterator const_reverse_iterator;
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
        #[cpp(func = "const_reverse_iterator hicc::make_constructor<const_reverse_iterator, Self>(Self&&)")]
        fn into_reverse(self) -> *mut CppMultiMapRevIter<K, V>;
    }

    #[cpp(class = "template<class K, class V, class Compare, class Allocator> std::multimap<K, V, Compare, Allocator>::iterator")]
    class CppMultiMapIterMut<K, V> {
        hicc::cpp! {
            typedef typename SelfContainer::reverse_iterator reverse_iterator;
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
        #[cpp(func = "reverse_iterator hicc::make_constructor<reverse_iterator, Self>(Self&&)")]
        fn into_reverse(self) -> *mut CppMultiMapRevIterMut<K, V>;
    }

    #[cpp(class = "template<class K, class V, class Compare, class Allocator> std::multimap<K, V, Compare, Allocator>::const_reverse_iterator")]
    class CppMultiMapRevIter<K, V> {
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

    #[cpp(class = "template<class K, class V, class Compare, class Allocator> std::multimap<K, V, Compare, Allocator>::reverse_iterator")]
    class CppMultiMapRevIterMut<K, V> {
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

impl<K: AbiType + 'static, V: AbiType + 'static> multimap<K, V> {
    /// ```
    /// use hicc_std::MultiMapIntInt;
    /// let mut map = MultiMapIntInt::new();
    /// map.insert(&1, &2);
    /// map.insert(&1, &1);
    /// map.iter().for_each(|(k, v)| println!("key = {k}, value = {v}"));
    /// ```
    pub fn iter(&self) -> impl Iterator<Item = (K::OutputRef<'_>, V::OutputRef<'_>)> {
        MultiMapIter {
            beg: unsafe { self.begin() },
            end: unsafe { self.end() },
            mark: PhantomData,
        }
    }

    /// ```
    /// use hicc_std::MultiMapIntInt;
    /// let mut map = MultiMapIntInt::new();
    /// map.insert(&1, &2);
    /// map.insert(&2, &3);
    /// map.iter_mut().for_each(|(_, v)| *v += 1);
    /// let mut it = map.iter();
    /// assert_eq!(it.next(), Some((&1, &3)));
    /// assert_eq!(it.next(), Some((&2, &4)));
    /// assert!(it.next().is_none());
    /// ```
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (K::OutputRef<'_>, V::OutputRefMut<'_>)> {
        let beg = unsafe { self.begin_mut() };
        let end = unsafe { self.end_mut() };
        MultiMapIterMut {
            beg,
            end,
            mark: PhantomData,
        }
    }

    /// ```
    /// use hicc_std::MultiMapIntInt;
    /// let mut map = MultiMapIntInt::new();
    /// map.insert(&1, &2);
    /// map.insert(&1, &1);
    /// map.rev_iter().for_each(|(k, v)| println!("key = {k}, value = {v}"));
    /// ```
    pub fn rev_iter(&self) -> impl Iterator<Item = (K::OutputRef<'_>, V::OutputRef<'_>)> {
        MultiMapRevIter {
            beg: unsafe { self.rbegin() },
            end: unsafe { self.rend() },
            mark: PhantomData,
        }
    }

    /// ```
    /// use hicc_std::MultiMapIntInt;
    /// let mut map = MultiMapIntInt::new();
    /// map.insert(&1, &2);
    /// map.insert(&2, &3);
    /// map.rev_iter_mut().for_each(|(_, v)| *v += 1);
    /// let mut it = map.iter();
    /// assert_eq!(it.next(), Some((&1, &3)));
    /// assert_eq!(it.next(), Some((&2, &4)));
    /// assert!(it.next().is_none());
    /// ```
    pub fn rev_iter_mut(
        &mut self,
    ) -> impl Iterator<Item = (K::OutputRef<'_>, V::OutputRefMut<'_>)> {
        let beg = unsafe { self.rbegin_mut() };
        let end = unsafe { self.rend_mut() };
        MultiMapRevIterMut {
            beg,
            end,
            mark: PhantomData,
        }
    }

    /// ```
    /// use hicc_std::MultiMapIntInt;
    /// let mut map = MultiMapIntInt::new();
    /// map.insert(&1, &2);
    /// map.insert(&2, &3);
    /// map.insert(&3, &4);
    /// let mut it = map.iter_lower_upper_bound(Some(&1), Some(&2));
    /// assert_eq!(it.next(), Some((&1, &2)));
    /// assert_eq!(it.next(), Some((&2, &3)));
    /// assert_eq!(it.next(), None);
    /// ```
    pub fn iter_lower_upper_bound(
        &self,
        lower_key: Option<&K::InputType>,
        upper_key: Option<&K::InputType>,
    ) -> impl Iterator<Item = (K::OutputRef<'_>, V::OutputRef<'_>)> {
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
        MultiMapIter {
            beg,
            end,
            mark: PhantomData,
        }
    }

    /// ```
    /// use hicc_std::MultiMapIntInt;
    /// let mut map = MultiMapIntInt::new();
    /// map.insert(&1, &2);
    /// map.insert(&2, &3);
    /// map.insert(&3, &4);
    /// map.iter_lower_upper_bound_mut(Some(&1), Some(&2)).for_each(|(_, v)| *v -= 1);
    /// let mut it = map.iter_lower_upper_bound(Some(&1), Some(&2));
    /// assert_eq!(it.next(), Some((&1, &1)));
    /// assert_eq!(it.next(), Some((&2, &2)));
    /// assert_eq!(it.next(), None);
    /// ```
    pub fn iter_lower_upper_bound_mut(
        &mut self,
        lower_key: Option<&K::InputType>,
        upper_key: Option<&K::InputType>,
    ) -> impl Iterator<Item = (K::OutputRef<'_>, V::OutputRefMut<'_>)> {
        let beg = if let Some(key) = lower_key {
            unsafe { self.lower_bound_mut(key) }
        } else {
            unsafe { self.begin_mut() }
        };
        let end = if let Some(key) = upper_key {
            unsafe { self.upper_bound_mut(key) }
        } else {
            unsafe { self.end_mut() }
        };
        MultiMapIterMut {
            beg,
            end,
            mark: PhantomData,
        }
    }

    /// ```
    /// use hicc_std::MultiMapIntInt;
    /// let mut map = MultiMapIntInt::new();
    /// map.insert(&1, &0);
    /// map.insert(&2, &1);
    /// map.insert(&3, &2);
    /// let mut it = map.rev_iter_lower_upper_bound(Some(&1), Some(&2));
    /// assert_eq!(it.next(), Some((&2, &1)));
    /// assert_eq!(it.next(), Some((&1, &0)));
    /// assert_eq!(it.next(), None);
    /// ```
    pub fn rev_iter_lower_upper_bound(
        &self,
        lower_key: Option<&K::InputType>,
        upper_key: Option<&K::InputType>,
    ) -> impl Iterator<Item = (K::OutputRef<'_>, V::OutputRef<'_>)> {
        let beg = if let Some(key) = upper_key {
            unsafe { self.upper_bound(key).into_value() }.into_reverse()
        } else {
            unsafe { self.rbegin() }
        };
        let end = if let Some(key) = lower_key {
            unsafe { self.lower_bound(key).into_value() }.into_reverse()
        } else {
            unsafe { self.rend() }
        };
        MultiMapRevIter {
            beg,
            end,
            mark: PhantomData,
        }
    }

    /// ```
    /// use hicc_std::MultiMapIntInt;
    /// let mut map = MultiMapIntInt::new();
    /// map.insert(&1, &0);
    /// map.insert(&2, &1);
    /// map.insert(&3, &2);
    /// map.rev_iter_lower_upper_bound_mut(Some(&1), Some(&2)).for_each(|(_, v)| *v -= 1);
    /// let mut it = map.rev_iter_lower_upper_bound(Some(&1), Some(&2));
    /// assert_eq!(it.next(), Some((&2, &0)));
    /// assert_eq!(it.next(), Some((&1, &-1)));
    /// assert_eq!(it.next(), None);
    /// ```
    pub fn rev_iter_lower_upper_bound_mut(
        &mut self,
        lower_key: Option<&K::InputType>,
        upper_key: Option<&K::InputType>,
    ) -> impl Iterator<Item = (K::OutputRef<'_>, V::OutputRefMut<'_>)> {
        let beg = if let Some(key) = upper_key {
            unsafe { self.upper_bound_mut(key).into_value() }.into_reverse()
        } else {
            unsafe { self.rbegin_mut() }
        };
        let end = if let Some(key) = lower_key {
            unsafe { self.lower_bound_mut(key).into_value() }.into_reverse()
        } else {
            unsafe { self.rend_mut() }
        };
        MultiMapRevIterMut {
            beg,
            end,
            mark: PhantomData,
        }
    }
}

/// 对应`std::multimap<K, V>::const_iterator`
struct MultiMapIter<'a, K: AbiType + 'static, V: AbiType + 'static> {
    beg: ClassMutPtr<'static, CppMultiMapIter<K, V>>,
    end: ClassMutPtr<'static, CppMultiMapIter<K, V>>,
    mark: PhantomData<&'a multimap<K, V>>,
}

impl<'a, K: AbiType + 'static, V: AbiType + 'static> Iterator for MultiMapIter<'a, K, V> {
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

/// 对应`std::multimap<K, V>::const_reverse_iterator`
struct MultiMapRevIter<'a, K: AbiType + 'static, V: AbiType + 'static> {
    beg: ClassMutPtr<'static, CppMultiMapRevIter<K, V>>,
    end: ClassMutPtr<'static, CppMultiMapRevIter<K, V>>,
    mark: PhantomData<&'a multimap<K, V>>,
}

impl<'a, K: AbiType + 'static, V: AbiType + 'static> Iterator for MultiMapRevIter<'a, K, V> {
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

/// 对应`std::multimap<K, V>::iterator`
struct MultiMapIterMut<'a, K: AbiType + 'static, V: AbiType + 'static> {
    beg: ClassMutPtr<'static, CppMultiMapIterMut<K, V>>,
    end: ClassMutPtr<'static, CppMultiMapIterMut<K, V>>,
    mark: PhantomData<&'a mut multimap<K, V>>,
}

impl<'a, K: AbiType + 'static, V: AbiType + 'static> Iterator for MultiMapIterMut<'a, K, V> {
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

/// 对应`std::multimap<K, V>::reverse_iterator`
struct MultiMapRevIterMut<'a, K: AbiType + 'static, V: AbiType + 'static> {
    beg: ClassMutPtr<'static, CppMultiMapRevIterMut<K, V>>,
    end: ClassMutPtr<'static, CppMultiMapRevIterMut<K, V>>,
    mark: PhantomData<&'a mut multimap<K, V>>,
}

impl<'a, K: AbiType + 'static, V: AbiType + 'static> Iterator for MultiMapRevIterMut<'a, K, V> {
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
