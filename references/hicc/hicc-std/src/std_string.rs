use crate::*;
use std::cmp::min;
use std::cmp::{Ordering, PartialEq, PartialOrd};
use std::convert::From;
use std::ffi::CStr;
use std::slice;

hicc::cpp! {
    #include <string>
}

hicc::import_class! {
    #[cpp(class = "template<class CharT, class Traits, class Allocator> std::basic_string<CharT, Traits, Allocator>")]
    pub class basic_string<CharT> {
        pub const npos: usize = usize::MAX;
        /// ```
        /// use std::ffi::CStr;
        /// use hicc_std::string;
        /// let s = string::from(c"abc");
        /// let cstr = unsafe { CStr::from_ptr(s.c_str()) };
        /// assert_eq!(cstr, c"abc");
        /// ```
        #[cpp(method = "const CharT* c_str() const")]
        pub fn c_str(&self) -> *const CharT;

        /// ```
        /// use hicc_std::string;
        /// let s = string::new();
        /// assert!(s.is_empty());
        /// ```
        #[cpp(method = "bool empty() const")]
        pub fn is_empty(&self) -> bool;

        /// ```
        /// use hicc_std::string;
        /// let s = string::new();
        /// assert_eq!(s.size(), 0);
        /// ```
        #[cpp(method = "size_t size() const")]
        pub fn size(&self) -> usize;

        /// ```
        /// use hicc_std::string;
        /// let s = string::new();
        /// println!("s.max_size = {}", s.max_size());
        /// ```
        #[cpp(method = "size_t max_size() const")]
        pub fn max_size(&self) -> usize;

        /// ```
        /// use hicc_std::string;
        /// let s = string::new();
        /// println!("s.capacity = {}", s.capacity());
        /// ```
        #[cpp(method = "size_t capacity() const")]
        pub fn capacity(&self) -> usize;

        /// ```
        /// use hicc_std::string;
        /// let mut s = string::from(c"a");
        /// s.resize(3, b'c' as i8);
        /// assert_eq!(s.as_cstr(), c"acc");
        /// ```
        #[cpp(method = "void resize(size_t, CharT) ")]
        pub fn resize(&mut self, n: usize, c: CharT);

        /// ```
        /// use hicc_std::string;
        /// let mut s = string::from(c"abc");
        /// s.reserve(100);
        /// println!("s.capacity = {}", s.capacity());
        /// ```
        #[cpp(method = "void reserve(size_t)")]
        pub fn reserve(&mut self, n: usize);

        /// ```
        /// use hicc_std::string;
        /// let mut s = string::from(c"abc");
        /// s.clear();
        /// assert!(s.is_empty());
        /// ```
        #[cpp(method = "void clear()")]
        pub fn clear(&mut self);

        /// ```
        /// use hicc_std::string;
        /// let mut s = string::from(c"abc");
        /// s.reserve(100);
        /// println!("before shrink_to_fit: capacity = {}", s.capacity());
        /// s.assign(10, b'c' as i8);
        /// s.shrink_to_fit();
        /// println!("after shrink_to_fit: capacity = {}", s.capacity());
        /// ```
        #[cpp(method = "void shrink_to_fit()")]
        pub fn shrink_to_fit(&mut self);

        /// 如果pos超出范围，会返回空字符串.
        /// ```
        /// use hicc_std::string;
        /// let mut s = string::from(c"abc");
        /// let substr = s.substr(1, 2);
        /// assert_eq!(substr.as_cstr(), c"bc");
        /// ```
        pub fn substr(&self, mut pos: usize, len: usize) -> Self {
            if pos > self.size() {
                pos = self.size();
            }
            self._substr(pos, len)
        }
        #[cpp(method = "Self substr(size_t, size_t) const")]
        fn _substr(&self, pos: usize, len: usize) -> Self;

        /// ```
        /// use hicc_std::string;
        /// let s1 = string::from(c"abc");
        /// let s2 = string::from(c"abd");
        /// assert!(s1 < s2);
        /// assert!(s1 != s2);
        /// ```
        #[cpp(method = "int compare(const Self&) const")]
        pub fn compare(&self, other: &Self) -> i32;

        /// ```
        /// use hicc_std::string;
        /// let s1 = string::from(c"abcdef");
        /// let s2 = string::from(c"bcd");
        /// assert_eq!(s1.find_str(&s2, 0), 1);
        /// ```
        #[cpp(method = "size_t find(const Self&, size_t) const")]
        pub fn find_str(&self, target: &Self, pos: usize) -> usize;

        /// ```
        /// use hicc_std::string;
        /// let s = string::from(c"abcdef");
        /// assert_eq!(s.find(b'c' as i8, 1), 2);
        /// ```
        #[cpp(method = "size_t find(CharT, size_t) const")]
        pub fn find(&self, c: CharT, pos: usize) -> usize;

        /// ```
        /// use hicc_std::string;
        /// let s1 = string::from(c"abcdef");
        /// let s2 = string::from(c"bcd");
        /// assert_eq!(s1.rfind_str(&s2, string::npos), 1);
        /// ```
        #[cpp(method = "size_t rfind(const Self&, size_t) const")]
        pub fn rfind_str(&self, target: &Self, pos: usize) -> usize;

        /// ```
        /// use hicc_std::string;
        /// let s = string::from(c"abcdef");
        /// assert_eq!(s.rfind(b'c' as i8, string::npos), 2);
        /// ```
        #[cpp(method = "size_t rfind(CharT, size_t) const")]
        pub fn rfind(&self, c: CharT, pos: usize) -> usize;

        /// ```
        /// use hicc_std::string;
        /// let s1 = string::from(c"abcdef");
        /// let s2 = string::from(c"dfb");
        /// assert_eq!(s1.find_first_of(&s2, 0), 1);
        /// ```
        #[cpp(method = "size_t find_first_of(const Self&, size_t) const")]
        pub fn find_first_of(&self, target: &Self, pos: usize) -> usize;

        /// ```
        /// use hicc_std::string;
        /// let s = string::from(c"abcabc");
        /// assert_eq!(s.find_first(b'c' as i8, 0), 2);
        /// ```
        #[cpp(method = "size_t find_first_of(CharT, size_t) const")]
        pub fn find_first(&self, c: CharT, pos: usize) -> usize;

        /// ```
        /// use hicc_std::string;
        /// let s1 = string::from(c"abcdef");
        /// let s2 = string::from(c"dfb");
        /// assert_eq!(s1.find_last_of(&s2, string::npos), 5);
        /// ```
        #[cpp(method = "size_t find_last_of(const Self&, size_t) const")]
        pub fn find_last_of(&self, target: &Self, pos: usize) -> usize;

        /// ```
        /// use hicc_std::string;
        /// let s = string::from(c"abcabc");
        /// assert_eq!(s.find_last(b'c' as i8, string::npos), 5);
        /// ```
        #[cpp(method = "size_t find_last_of(CharT, size_t) const")]
        pub fn find_last(&self, c: CharT, pos: usize) -> usize;

        /// ```
        /// use hicc_std::string;
        /// let s1 = string::from(c"abcdef");
        /// let s2 = string::from(c"abc");
        /// assert_eq!(s1.find_first_not_of(&s2, 0), 3);
        /// ```
        #[cpp(method = "size_t find_first_not_of(const Self&, size_t) const")]
        pub fn find_first_not_of(&self, target: &Self, pos: usize) -> usize;

        /// ```
        /// use hicc_std::string;
        /// let s = string::from(c"abcabc");
        /// assert_eq!(s.find_first_not(b'a' as i8, 0), 1);
        /// ```
        #[cpp(method = "size_t find_first_not_of(CharT, size_t) const")]
        pub fn find_first_not(&self, c: CharT, pos: usize) -> usize;

        /// ```
        /// use hicc_std::string;
        /// let s1 = string::from(c"abcdef");
        /// let s2 = string::from(c"def");
        /// assert_eq!(s1.find_last_not_of(&s2, string::npos), 2);
        /// ```
        #[cpp(method = "size_t find_last_not_of(const Self&, size_t) const")]
        pub fn find_last_not_of(&self, target: &Self, pos: usize) -> usize;

        /// ```
        /// use hicc_std::string;
        /// let s = string::from(c"abcabc");
        /// assert_eq!(s.find_last_not(b'c' as i8, string::npos), 4);
        /// ```
        #[cpp(method = "size_t find_last_not_of(CharT, size_t) const")]
        pub fn find_last_not(&self, c: CharT, pos: usize) -> usize;

        /// 如果subpos超出范围则不做任何改变.
        /// ```
        /// use hicc_std::string;
        /// let mut s1 = string::from(c"abc");
        /// let s2 = string::from(c"def");
        /// s1.append_str(&s2, 0, 2);
        /// assert_eq!(s1.as_cstr(), c"abcde");
        /// ```
        pub fn append_str(&mut self, s: &Self, subpos: usize, sublen: usize) {
            if subpos < s.size() {
                self._append_str(s, subpos, sublen);
            }
        }
        #[cpp(method = "Self& append(const Self&, size_t, size_t)")]
        fn _append_str(&mut self, s: &Self, subpos: usize, sublen: usize);

        /// ```
        /// use hicc_std::string;
        /// let mut s = string::from(c"abc");
        /// s.append(1, b'd' as i8);
        /// assert_eq!(s.as_cstr(), c"abcd");
        /// ```
        #[cpp(method = "Self& append(size_t, CharT)")]
        pub fn append(&mut self, n: usize, c: CharT);

        /// 如果subpos超出范围不做任何改变, 如果pos超出范围则追加到最后.
        /// ```
        /// use hicc_std::string;
        /// let mut s1 = string::from(c"abc");
        /// let mut s2 = string::from(c"def");
        /// s1.insert_str(0, &s2, 0, string::npos);
        /// assert_eq!(s1.as_cstr(), c"defabc");
        /// s1.insert_str(100, &s2, 0, string::npos);
        /// assert_eq!(s1.as_cstr(), c"defabcdef");
        /// ```
        pub fn insert_str(&mut self, pos: usize, s: &Self, subpos: usize, sublen: usize) {
            if subpos < s.size() {
                self._insert_str(min(pos, self.size()), s, subpos, sublen);
            }
        }
        #[cpp(method = "Self& insert(size_t, const Self&, size_t, size_t)")]
        fn _insert_str(&mut self, pos: usize, s: &Self, subpos: usize, sublen: usize);

        /// 如果pos超出范围则追加到最后.
        /// ```
        /// use hicc_std::string;
        /// let mut s = string::from(c"abc");
        /// s.insert(0, 3, b'd' as i8);
        /// assert_eq!(s.as_cstr(), c"dddabc");
        /// s.insert(110, 3, b'd' as i8);
        /// assert_eq!(s.as_cstr(), c"dddabcddd");
        /// ```
        pub fn insert(&mut self, pos: usize, n: usize, c: CharT::InputType) {
            self._insert(min(pos, self.size()), n, c);
        }
        #[cpp(method = "Self& insert(size_t, size_t, CharT)")]
        fn _insert(&mut self, pos: usize, n: usize, c: CharT);

        /// 如果pos或者subpos超出范围则不做任何改变.
        /// ```
        /// use hicc_std::string;
        /// let mut s1 = string::from(c"abc");
        /// let mut s2 = string::from(c"de");
        /// s1.replace_str(0, string::npos, &s2, 0, string::npos);
        /// assert_eq!(s1.as_cstr(), c"de");
        /// ```
        pub fn replace_str(&mut self, pos: usize, len: usize, s: &Self, subpos: usize, sublen: usize) {
            if pos < self.size() && subpos < s.size() {
                self._replace_str(pos, len, s, subpos, sublen);
            }
        }
        #[cpp(method = "Self& replace(size_t, size_t, const Self&, size_t, size_t)")]
        fn _replace_str(&mut self, pos: usize, len: usize, s: &Self, subpos: usize, sublen: usize);

        /// 如果pos超出范围则不做任何改变.
        /// ```
        /// use hicc_std::string;
        /// let mut s = string::from(c"abc");
        /// s.replace(0, string::npos, 1, b'd' as i8);
        /// assert_eq!(s.as_cstr(), c"d");
        /// ```
        pub fn replace(&mut self, pos: usize, len: usize, n: usize, c: CharT::InputType) {
            if pos < self.size() {
                self._replace(pos, len, n, c);
            }
        }
        #[cpp(method = "Self& replace(size_t, size_t, size_t, CharT)")]
        fn _replace(&mut self, pos: usize, len: usize, n: usize, c: CharT);

        /// 如果subpos超出范围则不做任何改变.
        /// ```
        /// use hicc_std::string;
        /// let mut s1 = string::from(c"abc");
        /// let mut s2 = string::from(c"de");
        /// s1.assign_str(&s2, 0, string::npos);
        /// assert_eq!(s1.as_cstr(), c"de");
        /// ```
        pub fn assign_str(&mut self, s: &Self, subpos: usize, sublen: usize) {
            if subpos < s.size() {
                self._assign_str(s, subpos, sublen);
            }
        }
        #[cpp(method = "Self& assign(const Self&, size_t, size_t)")]
        fn _assign_str(&mut self, s: &Self, subpos: usize, sublen: usize);

        /// ```
        /// use hicc_std::string;
        /// let mut s = string::from(c"abc");
        /// s.assign(1, b'd' as i8);
        /// assert_eq!(s.as_cstr(), c"d");
        /// ```
        #[cpp(method = "Self& assign(size_t, CharT)")]
        pub fn assign(&mut self, n: usize, c: CharT);

        /// 如果pos超出范围则不做任何改变.
        /// ```
        /// use hicc_std::string;
        /// let mut s = string::from(c"abc");
        /// s.erase(1, 1);
        /// assert_eq!(s.as_cstr(), c"ac");
        /// ```
        pub fn erase(&mut self, pos: usize, len: usize) {
            if pos < self.size() {
                self._erase(pos, len);
            }
        }
        #[cpp(method = "Self& erase(size_t, size_t)")]
        fn _erase(&mut self, pos: usize, len: usize);


        /// ```
        /// use hicc_std::string;
        /// let mut s = string::from(c"abc");
        /// s.push_back(b'd' as i8);
        /// assert_eq!(s.as_cstr(), c"abcd");
        /// ```
        #[cpp(method = "void push_back(CharT)")]
        pub fn push_back(&mut self, c: CharT);

        /// 如果为空则不做任何改变.
        /// ```
        /// use hicc_std::string;
        /// let mut s = string::from(c"abc");
        /// s.pop_back();
        /// assert_eq!(s.as_cstr(), c"ab");
        /// ```
        pub fn pop_back(&mut self) {
            if !self.is_empty() {
                self._pop_back();
            }
        }
        #[cpp(method = "void pop_back()")]
        fn _pop_back(&mut self);

        /// ```
        /// use hicc_std::string;
        /// let mut s1 = string::from(c"abc");
        /// let mut s2 = string::from(c"de");
        /// s1.swap(&mut s2);
        /// assert_eq!(s1.as_cstr(), c"de");
        /// assert_eq!(s2.as_cstr(), c"abc");
        /// ```
        #[cpp(method = "void swap(Self&)")]
        pub fn swap(&mut self, other: &mut Self);

        /// ```
        /// use hicc_std::string;
        /// let s = string::from(c"abc");
        /// assert_eq!(s.get(0), Some(&(b'a' as i8)));
        /// assert_eq!(s.get(3), None);
        /// ```
        pub fn get(&self, pos: usize) -> Option<CharT::OutputRef<'_>> {
            if pos < self.size() {
                return Some(self._get(pos));
            }
            None
        }
        #[cpp(method = "const CharT& at(size_t) const")]
        fn _get(&self, pos: usize) -> &CharT;


        /// ```
        /// use hicc_std::string;
        /// let mut s = string::from(c"abc");
        /// assert!(s.get_mut(0).is_some());
        /// assert_eq!(*s.get_mut(0).unwrap(), b'a' as i8);
        /// assert_eq!(s.get(3), None);
        /// ```
        pub fn get_mut(&mut self, pos: usize) -> Option<CharT::OutputRefMut<'_>> {
            if pos < self.size() {
                return Some(self._get_mut(pos));
            }
            None
        }
        #[cpp(method = "CharT& at(size_t)")]
        fn _get_mut(&mut self, pos: usize) -> &mut CharT;
    }

    /// 对应`std::string`
    #[allow(non_camel_case_types)]
    pub type string = basic_string<hicc::Pod<i8>>;
    /// 对应`std::u16string`
    #[allow(non_camel_case_types)]
    pub type u16string = basic_string<hicc::Pod<i16>>;
    /// 对应`std::u32string`
    #[allow(non_camel_case_types)]
    pub type u32string = basic_string<hicc::Pod<i32>>;
}

unsafe impl<CharT: hicc::AbiType + Sync> Send for basic_string<CharT> {}
unsafe impl<CharT: hicc::AbiType + Sync> Sync for basic_string<CharT> {}

impl<T: Sized + 'static> basic_string<hicc::Pod<T>> {
    /// ```
    /// use hicc_std::string;
    /// let mut s = string::with_cstr(c"bcd");
    /// assert_eq!(s.as_slice(), &[b'b' as i8, b'c' as i8, b'd' as i8]);
    /// ```
    pub fn as_slice(&self) -> &[T] {
        let cstr = self.c_str();
        let size = self.size();
        unsafe { slice::from_raw_parts(cstr, size) }
    }

    /// ```
    /// use hicc_std::string;
    /// let mut s = string::with_cstr(c"abc");
    /// s.as_slice_mut().iter_mut().for_each(|mut c| { *c += 1; });
    /// assert_eq!(s.as_slice(), &[b'b' as i8, b'c' as i8, b'd' as i8]);
    /// ```
    pub fn as_slice_mut(&mut self) -> &mut [T] {
        let cstr = self.c_str();
        let size = self.size();
        unsafe { slice::from_raw_parts_mut(cstr.cast_mut(), size) }
    }
}

impl<T: hicc::AbiType> PartialEq for basic_string<T> {
    fn eq(&self, other: &Self) -> bool {
        self.compare(other) == 0
    }
}

impl<T: hicc::AbiType> PartialOrd for basic_string<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match self.compare(other) {
            0 => Some(Ordering::Equal),
            1.. => Some(Ordering::Greater),
            _ => Some(Ordering::Less),
        }
    }
}

impl string {
    pub fn new() -> Self {
        string_new()
    }

    pub fn with_cstr(s: &CStr) -> Self {
        unsafe { string_with_cstr(s.as_ptr()) }
    }

    pub fn with_buf(b: &[u8]) -> Self {
        unsafe { string_with_buf(b.as_ptr(), b.len()) }
    }

    pub fn as_cstr(&self) -> &CStr {
        unsafe { CStr::from_ptr(self.c_str()) }
    }
}

impl From<&CStr> for string {
    fn from(s: &CStr) -> Self {
        unsafe { string_with_cstr(s.as_ptr()) }
    }
}

impl Default for string {
    fn default() -> Self {
        Self::new()
    }
}

impl u16string {
    pub fn new() -> Self {
        u16string_new()
    }

    pub fn with_buf(b: &[u16]) -> Self {
        unsafe { u16string_with_buf(b.as_ptr(), b.len()) }
    }
}

impl Default for u16string {
    fn default() -> Self {
        Self::new()
    }
}

impl u32string {
    pub fn new() -> Self {
        u32string_new()
    }

    pub fn with_buf(b: &[u32]) -> Self {
        unsafe { u32string_with_buf(b.as_ptr(), b.len()) }
    }
}

impl Default for u32string {
    fn default() -> Self {
        Self::new()
    }
}
