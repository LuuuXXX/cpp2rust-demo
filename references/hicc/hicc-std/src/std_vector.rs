use hicc::{AbiClass, AbiType, ClassArray, ClassMutArray};
use std::slice;

hicc::cpp! {
    #include <vector>
    #include <algorithm>
}

hicc::import_class! {
    #[cpp(class = "template<class T, class Allocator> std::vector<T, Allocator>")]
    pub class vector<T> {
        #[cpp(method = "const T* data() const")]
        fn data(&self) -> *const T;
        /// ```
        /// use hicc_std::VecInt;
        /// let vec = VecInt::new();
        /// assert!(vec.is_empty());
        /// ```
        #[cpp(method = "bool empty() const")]
        pub fn is_empty(&self) -> bool;

        /// ```
        /// use hicc_std::VecInt;
        /// let vec = VecInt::new();
        /// assert_eq!(vec.size(), 0);
        /// ```
        #[cpp(method = "size_t size() const")]
        pub fn size(&self) -> usize;

        /// ```
        /// use hicc_std::VecInt;
        /// let vec = VecInt::new();
        /// println!("vec.max_size = {}", vec.max_size());
        /// ```
        #[cpp(method = "size_t max_size() const")]
        pub fn max_size(&self) -> usize;

        /// ```
        /// use hicc_std::VecInt;
        /// let vec = VecInt::new();
        /// println!("vec.capacity = {}", vec.capacity());
        /// ```
        #[cpp(method = "size_t capacity() const")]
        pub fn capacity(&self) -> usize;

        /// ```
        /// use hicc_std::VecInt;
        /// let mut vec = VecInt::new();
        /// vec.push_back(&1);
        /// vec.clear();
        /// assert!(vec.is_empty());
        /// ```
        #[cpp(method = "void clear()")]
        pub fn clear(&mut self);

        /// ```
        /// use hicc_std::VecInt;
        /// let mut vec = VecInt::new();
        /// vec.reserve(10_usize);
        /// assert!(vec.is_empty());
        /// println!("vec.capacity = {}", vec.capacity());
        /// ```
        #[cpp(method = "void reserve(size_t)")]
        pub fn reserve(&mut self, n: usize);

        /// ```
        /// use hicc_std::VecInt;
        /// let mut vec = VecInt::new();
        /// vec.reserve(10_usize);
        /// vec.shrink_to_fit();
        /// println!("vec.capacity = {}", vec.capacity());
        /// ```
        #[cpp(method = "void shrink_to_fit()")]
        pub fn shrink_to_fit(&mut self);

        /// ```
        /// use hicc_std::VecInt;
        /// let mut vec = VecInt::new();
        /// vec.resize(10, &1);
        /// assert_eq!(vec.size(), 10);
        /// assert_eq!(vec.back(), Some(&1));
        /// vec.as_slice().iter().for_each(|v| println!("{v}"));
        /// ```
        #[cpp(method = "void resize(size_t, const T&)")]
        pub fn resize(&mut self, n: usize, val: &T);

        /// ```
        /// use hicc_std::VecInt;
        /// let mut vec = VecInt::new();
        /// assert_eq!(vec.get(0), None);
        /// vec.push_back(&1);
        /// assert_eq!(vec.get(0), Some(&1));
        /// ```
        pub fn get(&self, pos: usize) -> Option<T::OutputRef<'_>> {
            if pos < self.size() {
                return unsafe { Some(self.at(pos)) };
            }
            None
        }
        #[cpp(method = "const T& at(size_t) const")]
        unsafe fn at(&self, pos: usize) -> &T;

        /// ```
        /// use hicc_std::VecInt;
        /// let mut vec = VecInt::new();
        /// assert_eq!(vec.get(0), None);
        /// vec.push_back(&1);
        /// *vec.get_mut(0).unwrap() += 1;
        /// assert_eq!(vec.get(0), Some(&2));
        /// ```
        pub fn get_mut(&mut self, pos: usize) -> Option<T::OutputRefMut<'_>> {
            if pos < self.size() {
                return unsafe { Some(self.at_mut(pos)) };
            }
            None
        }
        #[cpp(method = "T& at(size_t)")]
        unsafe fn at_mut(&mut self, pos: usize) -> &mut T;

        /// ```
        /// use hicc_std::VecInt;
        /// let mut vec = VecInt::new();
        /// assert_eq!(vec.back(), None);
        /// vec.push_back(&1);
        /// assert_eq!(vec.back(), Some(&1));
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
        /// use hicc_std::VecInt;
        /// let mut vec = VecInt::new();
        /// assert_eq!(vec.back_mut(), None);
        /// vec.push_back(&1);
        /// *vec.back_mut().unwrap() += 1;
        /// assert_eq!(vec.back(), Some(&2));
        ///
        /// use hicc_std::{string, VecString};
        /// use hicc::AbiClass;
        /// let mut vec = VecString::new();
        /// vec.push_back(&string::from(c"hello"));
        /// assert_eq!(vec.back(), Some(string::from(c"hello").into_ref()));
        /// vec.back_mut().unwrap().write(string::from(c"world"));
        /// assert_eq!(vec.back(), Some(string::from(c"world").into_ref()));
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
        /// use hicc_std::VecInt;
        /// let mut vec = VecInt::new();
        /// assert_eq!(vec.front(), None);
        /// vec.push_back(&1);
        /// assert_eq!(vec.front(), Some(&1));
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
        /// use hicc_std::VecInt;
        /// let mut vec = VecInt::new();
        /// assert_eq!(vec.front_mut(), None);
        /// vec.push_back(&1);
        /// *vec.front_mut().unwrap() += 1;
        /// assert_eq!(vec.front(), Some(&2));
        ///
        /// use hicc_std::{string, VecString};
        /// use hicc::AbiClass;
        /// let mut vec = VecString::new();
        /// vec.push_back(&string::from(c"hello"));
        /// assert_eq!(vec.front(), Some(string::from(c"hello").into_ref()));
        /// vec.front_mut().unwrap().write(string::from(c"world"));
        /// assert_eq!(vec.front(), Some(string::from(c"world").into_ref()));
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
        /// use hicc_std::VecInt;
        /// let mut vec = VecInt::new();
        /// vec.assign(10, &1);
        /// assert_eq!(vec.size(), 10);
        /// ```
        #[cpp(method = "void assign(size_t, const T&)")]
        pub fn assign(&mut self, n: usize, val: &T);

        /// 如果为空不做任何改变.
        /// ```
        /// use hicc_std::VecInt;
        /// let mut vec = VecInt::new();
        /// vec.pop_back();
        /// vec.push_back(&1);
        /// assert!(!vec.is_empty());
        /// vec.pop_back();
        /// assert!(vec.is_empty());
        /// ```
        pub fn pop_back(&mut self) {
            if !self.is_empty() {
                unsafe { self._pop_back() };
            }
        }
        #[cpp(method = "void pop_back()")]
        unsafe fn _pop_back(&mut self);

        /// ```
        /// use hicc_std::VecInt;
        /// let mut vec = VecInt::new();
        /// vec.push_back(&1);
        /// assert_eq!(vec.front(), Some(&1));
        /// ```
        #[cpp(method = "void push_back(const T&)")]
        pub fn push_back(&mut self, val: &T);

        /// ```
        /// use hicc_std::VecInt;
        /// let mut vec = VecInt::new();
        /// vec.push_back(&1);
        /// vec.swap(&mut VecInt::new());
        /// assert!(vec.is_empty());
        /// ```
        #[cpp(method = "void swap(Self&)")]
        pub fn swap(&mut self, other: &mut Self);

        hicc::cpp! {
            static void insert(Self& self, size_t pos, size_t count, const T& val) {
                self.insert(self.begin() + std::min(pos, self.size()), count, val);
            }
        }
        /// 如果pos超出范围则追加到最后.
        /// ```
        /// use hicc_std::VecInt;
        /// let mut vec = VecInt::new();
        /// vec.insert(0, 2, &1);
        /// vec.insert(vec.size(), 1, &2);
        /// assert_eq!(vec.as_slice(), &[1, 1, 2]);
        /// ```
        #[cpp(func = "void SelfMethods::insert(Self&, size_t, size_t, const T&)")]
        pub fn insert(&mut self, pos: usize, count: usize, val: &T);

        hicc::cpp! {
            static void erase(Self& self, size_t pos, size_t count) {
                pos = std::min(pos, self.size());
                count = std::min(count, self.size() - pos);
                self.erase(self.begin() + pos, self.begin() + pos + count);
            }
        }
        /// 如果pos超出范围则不做任何改变.
        /// ```
        /// use hicc_std::VecInt;
        /// let mut vec = VecInt::new();
        /// vec.insert(0, 10, &2);
        /// vec.erase(1, 9);
        /// assert_eq!(vec.size(), 1);
        /// ```
        #[cpp(func = "void SelfMethods::erase(Self&, size_t, size_t)")]
        pub fn erase(&mut self, pos: usize, count: usize);
    }
}

unsafe impl<T: AbiType + Sync> Send for vector<T> {}
unsafe impl<T: AbiType + Sync> Sync for vector<T> {}

impl<T: Sized + 'static> vector<hicc::Pod<T>> {
    /// ```
    /// use hicc_std::VecInt;
    /// let mut vec = VecInt::new();
    /// vec.insert(0, 2, &2);
    /// assert_eq!(vec.as_slice(), &[2, 2]);
    /// ```
    pub fn as_slice(&self) -> &[T] {
        let data = self.data();
        let size = self.size();
        unsafe { slice::from_raw_parts(data, size) }
    }

    /// ```
    /// use hicc_std::VecInt;
    /// let mut vec = VecInt::new();
    /// vec.insert(0, 2, &2);
    /// vec.as_slice_mut().iter_mut().for_each(|v| {*v += 1;});
    /// assert_eq!(vec.as_slice(), &[3, 3]);
    /// ```
    pub fn as_slice_mut(&mut self) -> &mut [T] {
        let data = self.data();
        let size = self.size();
        unsafe { slice::from_raw_parts_mut(data.cast_mut(), size) }
    }
}

impl<T: AbiClass + 'static> vector<T> {
    /// ```
    /// use hicc_std::{VecString, string};
    /// let mut vec = VecString::new();
    /// let msg = string::with_cstr(c"hello");
    /// vec.insert(0, 10, &msg);
    /// vec.as_array().iter().for_each(|msg| { println!("msg.size() = {}", msg.size()); });
    /// ```
    pub fn as_array(&self) -> ClassArray<'_, T> {
        unsafe { self.data().into_array(self.size()) }
    }
}

impl<T: AbiClass + 'static> vector<T> {
    /// ```
    /// use hicc_std::{VecString, string};
    /// let mut vec = VecString::new();
    /// let msg = string::with_cstr(c"hello");
    /// vec.insert(0, 10, &msg);
    /// vec.as_mut_array().iter_mut().for_each(|mut msg| { msg.append(1, b'c' as i8); });
    /// ```
    pub fn as_mut_array(&mut self) -> ClassMutArray<'_, T> {
        unsafe { self.data().into_mut_ptr().into_mut_array(self.size()) }
    }
}

hicc::import_class! {
    #[cpp(class = "template<class Allocator> std::vector<bool, Allocator>")]
    pub class VecBool {
        /// ```
        /// use hicc_std::VecBool;
        /// let vec = VecBool::new();
        /// assert!(vec.is_empty());
        /// ```
        #[cpp(method = "bool empty() const")]
        pub fn is_empty(&self) -> bool;

        /// ```
        /// use hicc_std::VecBool;
        /// let vec = VecBool::new();
        /// assert_eq!(vec.size(), 0);
        /// ```
        #[cpp(method = "size_t size() const")]
        pub fn size(&self) -> usize;

        /// ```
        /// use hicc_std::VecBool;
        /// let vec = VecBool::new();
        /// println!("vec.max_size = {}", vec.max_size());
        /// ```
        #[cpp(method = "size_t max_size() const")]
        pub fn max_size(&self) -> usize;

        /// ```
        /// use hicc_std::VecBool;
        /// let vec = VecBool::new();
        /// println!("vec.capacity = {}", vec.capacity());
        /// ```
        #[cpp(method = "size_t capacity() const")]
        pub fn capacity(&self) -> usize;

        /// ```
        /// use hicc_std::VecBool;
        /// let mut vec = VecBool::new();
        /// vec.push_back(true);
        /// vec.clear();
        /// assert!(vec.is_empty());
        /// ```
        #[cpp(method = "void clear()")]
        pub fn clear(&mut self);

        /// ```
        /// use hicc_std::VecBool;
        /// let mut vec = VecBool::new();
        /// vec.reserve(10_usize);
        /// assert!(vec.is_empty());
        /// println!("vec.capacity = {}", vec.capacity());
        /// ```
        #[cpp(method = "void reserve(size_t)")]
        pub fn reserve(&mut self, n: usize);

        /// ```
        /// use hicc_std::VecBool;
        /// let mut vec = VecBool::new();
        /// vec.reserve(10_usize);
        /// // ...
        /// vec.shrink_to_fit();
        /// ```
        #[cpp(method = "void shrink_to_fit()")]
        pub fn shrink_to_fit(&mut self);

        /// ```
        /// use hicc_std::VecBool;
        /// let mut vec = VecBool::new();
        /// vec.resize(2, true);
        /// assert_eq!(vec.size(), 2);
        /// assert_eq!(vec.back(), Some(true));
        /// assert_eq!(vec.front(), Some(true));
        /// ```
        #[cpp(method = "void resize(size_t, bool)")]
        pub fn resize(&mut self, n: usize, val: bool);

        /// ```
        /// use hicc_std::VecBool;
        /// let mut vec = VecBool::new();
        /// assert_eq!(vec.get(0), None);
        /// vec.push_back(true);
        /// assert_eq!(vec.get(0), Some(true));
        /// ```
        pub fn get(&self, pos: usize) -> Option<bool> {
            if pos < self.size() {
                return unsafe { Some(self.at(pos)) };
            }
            None
        }
        #[cpp(method = "bool at(size_t) const")]
        unsafe fn at(&self, pos: usize) -> bool;

        hicc::cpp! {
            static void set(Self& self, size_t pos, bool val) {
                if (pos < self.size()) {
                    self[pos] = val;
                }
            }
        }
        /// 如果`pos`超出范围，则不做任何修改.
        /// ```
        /// use hicc_std::VecBool;
        /// let mut vec = VecBool::new();
        /// vec.set(0, false);
        /// assert_eq!(vec.get(0), None);
        /// vec.push_back(true);
        /// assert_eq!(vec.get(0), Some(true));
        /// vec.set(0, false);
        /// assert_eq!(vec.get(0), Some(false));
        /// ```
        ///
        #[cpp(func = "void SelfMethods::set(Self&, size_t, bool)")]
        pub fn set(&mut self, pos: usize, val: bool);

        /// ```
        /// use hicc_std::VecBool;
        /// let mut vec = VecBool::new();
        /// assert_eq!(vec.back(), None);
        /// vec.push_back(true);
        /// assert_eq!(vec.back(), Some(true));
        /// ```
        pub fn back(&self) -> Option<bool> {
            if !self.is_empty() {
                return unsafe { Some(self._back()) };
            }
            None
        }
        #[cpp(method = "bool back() const")]
        unsafe fn _back(&self) -> bool;

        /// ```
        /// use hicc_std::VecBool;
        /// let mut vec = VecBool::new();
        /// vec.push_back(true);
        /// assert_eq!(vec.front(), Some(true));
        /// ```
        pub fn front(&self) -> Option<bool> {
            if !self.is_empty() {
                return unsafe { Some(self._front()) };
            }
            None
        }
        #[cpp(method = "bool front() const")]
        unsafe fn _front(&self) -> bool;

        /// ```
        /// use hicc_std::VecBool;
        /// let mut vec = VecBool::new();
        /// vec.assign(2, true);
        /// assert_eq!(vec.size(), 2);
        /// assert_eq!(vec.front(), Some(true));
        /// assert_eq!(vec.back(), Some(true));
        /// ```
        pub fn assign(&mut self, n: usize, val: bool) {
            self._assign(n, &val);
        }
        #[cpp(method = "void assign(size_t, const bool&)")]
        fn _assign(&mut self, n: usize, val: &bool);

        /// 如果为空不做任何改变.
        /// ```
        /// use hicc_std::VecBool;
        /// let mut vec = VecBool::new();
        /// vec.pop_back();
        /// vec.push_back(true);
        /// vec.pop_back();
        /// assert!(vec.is_empty());
        /// ```
        pub fn pop_back(&mut self) {
            if !self.is_empty() {
                unsafe { self._pop_back() };
            }
        }
        #[cpp(method = "void pop_back()")]
        unsafe fn _pop_back(&mut self);

        hicc::cpp! {
            static void push_back(Self& self, bool val) {
                self.push_back(val);
            }
        }
        /// ```
        /// use hicc_std::VecBool;
        /// let mut vec = VecBool::new();
        /// vec.push_back(true);
        /// assert_eq!(vec.front(), Some(true));
        /// ```
        #[cpp(func = "void SelfMethods::push_back(Self&, bool)")]
        pub fn push_back(&mut self, val: bool);

        /// ```
        /// use hicc_std::VecBool;
        /// let mut vec = VecBool::new();
        /// vec.push_back(true);
        /// vec.swap(&mut VecBool::new());
        /// assert!(vec.is_empty());
        /// ```
        #[cpp(method = "void swap(Self&)")]
        pub fn swap(&mut self, other: &mut Self);

        hicc::cpp! {
            static void insert(Self& self, size_t pos, size_t count, bool val) {
                self.insert(self.begin() + std::min(pos, self.size()), count, val);
            }
        }
        /// 如果pos超出范围则添加到最后.
        /// ```
        /// use hicc_std::VecBool;
        /// let mut vec = VecBool::new();
        /// vec.insert(0, 2, true);
        /// vec.insert(vec.size(), 1, false);
        /// assert_eq!(vec.get(0), Some(true));
        /// assert_eq!(vec.get(1), Some(true));
        /// assert_eq!(vec.get(2), Some(false));
        /// assert_eq!(vec.get(3), None);
        /// ```
        #[cpp(func = "void SelfMethods::insert(Self&, size_t, size_t, bool)")]
        pub fn insert(&mut self, pos: usize, count: usize, val: bool);

        hicc::cpp! {
            static void erase(Self& self, size_t pos, size_t count) {
                pos = std::min(pos, self.size());
                count = std::min(count, self.size() - pos);
                self.erase(self.begin() + pos, self.begin() + pos + count);
            }
        }
        /// 如果pos超出范围则不做任何改变.
        /// ```
        /// use hicc_std::VecBool;
        /// let mut vec = VecBool::new();
        /// vec.insert(0, 10, true);
        /// vec.erase(1, 9);
        /// assert_eq!(vec.size(), 1);
        /// assert_eq!(vec.get(0), Some(true));
        /// ```
        #[cpp(func = "void SelfMethods::erase(Self&, size_t, size_t)")]
        pub fn erase(&mut self, pos: usize, count: usize);
    }
}

unsafe impl Send for VecBool {}
unsafe impl Sync for VecBool {}

impl VecBool {

    /// ```
    /// use hicc_std::VecBool;
    /// let mut vec = VecBool::new();
    /// vec.resize(2, false);
    /// vec.set(0, true);
    /// let mut it = vec.iter();
    /// assert_eq!(it.next(), Some(true));
    /// assert_eq!(it.next(), Some(false));
    /// assert_eq!(it.next(), None);
    /// ```
    pub fn iter(&self) -> impl Iterator<Item = bool> + '_ {
        VecBoolIter { vec: self, pos: 0 }
    }

    /// ```
    /// use hicc_std::VecBool;
    /// let mut vec = VecBool::new();
    /// vec.resize(2, false);
    /// vec.set(0, true);
    /// let mut it = vec.rev_iter();
    /// assert_eq!(it.next(), Some(false));
    /// assert_eq!(it.next(), Some(true));
    /// assert_eq!(it.next(), None);
    /// ```
    pub fn rev_iter(&self) -> impl Iterator<Item = bool> + '_ {
        VecBoolRevIter {
            vec: self,
            pos: self.size(),
        }
    }
}

struct VecBoolIter<'a> {
    vec: &'a VecBool,
    pos: usize,
}

impl Iterator for VecBoolIter<'_> {
    type Item = bool;
    fn next(&mut self) -> Option<Self::Item> {
        if self.pos < self.vec.size() {
            self.pos += 1;
            return Some(unsafe { self.vec.at(self.pos - 1) });
        }
        None
    }
}

struct VecBoolRevIter<'a> {
    vec: &'a VecBool,
    pos: usize,
}

impl Iterator for VecBoolRevIter<'_> {
    type Item = bool;
    fn next(&mut self) -> Option<Self::Item> {
        if self.pos > 0 {
            self.pos -= 1;
            return Some(unsafe { self.vec.at(self.pos) });
        }
        None
    }
}
