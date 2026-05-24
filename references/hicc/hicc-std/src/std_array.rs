use hicc::{AbiClass, AbiType, ClassArray, ClassMutArray};
use std::slice;

hicc::cpp! {
    #include <array>
}

hicc::import_class! {
    #[cpp(class = "template<class T, size_t N> std::array<T, N>")]
    pub class array<T> {
        #[cpp(method = "const T* data() const")]
        fn data(&self) -> *const T;
        /// ```
        /// use hicc_std::ArrayInt;
        /// let array = ArrayInt::new_10();
        /// assert_eq!(array.size(), 10);
        /// ```
        #[cpp(method = "size_t size() const")]
        pub fn size(&self) -> usize;

        /// ```
        /// use hicc_std::ArrayInt;
        /// let array = ArrayInt::new_10();
        /// assert!(!array.is_empty());
        /// ```
        pub fn is_empty(&self) -> bool {
            self.size() == 0
        }

        /// ```
        /// use hicc_std::ArrayInt;
        /// let mut array = ArrayInt::new_10();
        /// assert_eq!(array.get(0), Some(&0));
        /// assert_eq!(array.get(10), None);
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
        /// use hicc_std::ArrayInt;
        /// let mut array = ArrayInt::new_10();
        /// assert_eq!(array.get_mut(0), Some(&mut 0));
        /// assert_eq!(array.get(10), None);
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
        /// use hicc_std::ArrayInt;
        /// let array = ArrayInt::new_10();
        /// assert_eq!(array.back(), Some(&0));
        /// ```
        pub fn back(&self) -> Option<T::OutputRef<'_>> {
            if !self.is_empty() {
                return unsafe { Some(self._back()) };
            }
            None
        }

        #[cpp(method = "const T& back() const")]
        unsafe fn _back(&self) -> &T;

        /// ```
        /// use hicc_std::ArrayInt;
        /// let mut array = ArrayInt::new_10();
        /// assert!(array.back_mut().is_some());
        /// *array.back_mut().unwrap() += 1;
        /// assert_eq!(array.back_mut(), Some(&mut 1));
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
        /// use hicc_std::ArrayInt;
        /// let array = ArrayInt::new_10();
        /// assert!(array.front().is_some());
        /// ```
        pub fn front(&self) -> Option<T::OutputRef<'_>> {
            if !self.is_empty() {
                return unsafe { Some(self._front()) };
            }
            None
        }
        #[cpp(method = "const T& front() const")]
        unsafe fn _front(&self) -> &T;

        /// ```
        /// use hicc_std::ArrayInt;
        /// let mut array = ArrayInt::new_10();
        /// assert!(array.front().is_some());
        /// *array.front_mut().unwrap() += 1;
        /// assert_eq!(array.front(), Some(&1));
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
        /// use hicc_std::ArrayInt;
        /// let mut array = ArrayInt::new_10();
        /// array.fill(&1);
        /// assert_eq!(array.front(), Some(&1));
        /// ```
        #[cpp(method = "void fill(const T&)")]
        pub fn fill(&mut self, val: &T);
    }
}

unsafe impl<T: AbiType + Sync> Send for array<T> {}
unsafe impl<T: AbiType + Sync> Sync for array<T> {}

impl<T: Sized + 'static> array<hicc::Pod<T>> {
    /// ```
    /// use hicc_std::ArrayInt;
    /// let mut array = ArrayInt::new_10();
    /// array.fill(&1);
    /// array.as_slice().iter().for_each(|v| println!("{v}"));
    /// ```
    pub fn as_slice(&self) -> &[T] {
        let data = self.data();
        let size = self.size();
        unsafe { slice::from_raw_parts(data, size) }
    }

    /// ```
    /// use hicc_std::ArrayInt;
    /// let mut array = ArrayInt::new_10();
    /// array.fill(&1);
    /// array.as_slice_mut().iter_mut().for_each(|v| {*v += 1;});
    /// ```
    pub fn as_slice_mut(&mut self) -> &mut [T] {
        let data = self.data();
        let size = self.size();
        unsafe { slice::from_raw_parts_mut(data.cast_mut(), size) }
    }
}

impl<T: AbiClass + 'static> array<T> {
    /// ```
    /// use hicc_std::{ArrayString, string};
    /// let mut array = ArrayString::new_10();
    /// array.fill(&string::from(c"hello"));
    /// array.as_array().iter().for_each(|msg| {
    ///     // ...
    ///     println!("msg.size() = {}", msg.size());
    /// });
    /// ```
    pub fn as_array(&self) -> ClassArray<'_, T> {
        unsafe { self.data().into_array(self.size()) }
    }

    /// ```
    /// use hicc_std::{ArrayString, string};
    /// let mut array = ArrayString::new_10();
    /// array.fill(&string::from(c"hello"));
    /// array.as_mut_array().iter_mut().for_each(|mut msg| {
    ///     msg.append(1, b'c' as i8);
    /// });
    /// ```
    pub fn as_mut_array(&mut self) -> ClassMutArray<'_, T> {
        unsafe { self.data().into_mut_ptr().into_mut_array(self.size()) }
    }
}
