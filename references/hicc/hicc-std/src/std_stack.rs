hicc::cpp! {
    #include <stack>
}

hicc::import_class! {
    #[cpp(class = "template<class T, class Container> std::stack<T, Container>")]
    pub class stack<T> {
        /// ```
        /// use hicc_std::StackInt;
        /// let mut stack = StackInt::new();
        /// assert!(stack.is_empty());
        /// stack.push(&1);
        /// assert!(!stack.is_empty());
        /// ```
        #[cpp(method = "bool empty() const")]
        pub fn is_empty(&self) -> bool;

        /// ```
        /// use hicc_std::StackInt;
        /// let mut stack = StackInt::new();
        /// assert_eq!(stack.size(), 0);
        /// stack.push(&1);
        /// assert_eq!(stack.size(), 1);
        /// ```
        #[cpp(method = "size_t size() const")]
        pub fn size(&self) -> usize;

        /// 如果为空不做任何改变.
        /// ```
        /// use hicc_std::StackInt;
        /// let mut stack = StackInt::new();
        /// stack.pop();
        /// stack.push(&1);
        /// assert_eq!(stack.size(), 1);
        /// stack.pop();
        /// assert_eq!(stack.size(), 0);
        /// ```
        pub fn pop(&mut self) {
            if !self.is_empty() {
                unsafe { self._pop() };
            }
        }
        #[cpp(method = "void pop()")]
        unsafe fn _pop(&mut self);

        /// ```
        /// use hicc_std::StackInt;
        /// let mut stack = StackInt::new();
        /// stack.push(&1);
        /// assert_eq!(stack.top(), Some(&1));
        /// ```
        #[cpp(method = "void push(const T&)")]
        pub fn push(&mut self, val: &T);

        /// ```
        /// use hicc_std::StackInt;
        /// let mut stack = StackInt::new();
        /// assert_eq!(stack.top(), None);
        /// stack.push(&1);
        /// assert_eq!(stack.top(), Some(&1));
        /// ```
        pub fn top(&self) -> Option<T::OutputRef<'_>> {
            if !self.is_empty() {
                return unsafe { Some(self._top()) };
            }
            None
        }
        #[cpp(method = "const T& top() const")]
        unsafe fn _top(&self) -> &T;

        /// ```
        /// use hicc_std::StackInt;
        /// let mut stack = StackInt::new();
        /// assert_eq!(stack.top_mut(), None);
        /// stack.push(&1);
        /// *stack.top_mut().unwrap() += 1;
        /// assert_eq!(stack.top_mut(), Some(&mut 2));
        ///
        /// use hicc_std::{string, StackString};
        /// use hicc::AbiClass;
        /// let mut stack = StackString::new();
        /// stack.push(&string::from(c"hello"));
        /// assert_eq!(stack.top(), Some(string::from(c"hello").into_ref()));
        /// stack.top_mut().unwrap().write(string::from(c"world"));
        /// assert_eq!(stack.top(), Some(string::from(c"world").into_ref()));
        /// ```
        pub fn top_mut(&mut self) -> Option<T::OutputRefMut<'_>> {
            if !self.is_empty() {
                return unsafe { Some(self._top_mut()) };
            }
            None
        }
        #[cpp(method = "T& top()")]
        unsafe fn _top_mut(&mut self) -> &mut T;

        /// ```
        /// use hicc_std::StackInt;
        /// let mut stack = StackInt::new();
        /// stack.push(&1);
        /// let mut other = StackInt::new();
        /// other.push(&2);
        /// assert_eq!(stack.top(), Some(&1));
        /// assert_eq!(other.top(), Some(&2));
        /// stack.swap(&mut other);
        /// assert_eq!(stack.top(), Some(&2));
        /// assert_eq!(other.top(), Some(&1));
        /// ```
        #[cpp(method = "void swap(Self&)")]
        pub fn swap(&mut self, other: &mut Self);
    }

    unsafe impl<T: hicc::AbiType + Sync> Send for stack<T> {}
    unsafe impl<T: hicc::AbiType + Sync> Sync for stack<T> {}
}
