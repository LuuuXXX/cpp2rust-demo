hicc::cpp! {
    #include <queue>
}

hicc::import_class! {
    #[cpp(class = "template<class T, class Container> std::queue<T, Container>")]
    pub class queue<T> {
        /// ```
        /// use hicc_std::QueueInt;
        /// let mut queue = QueueInt::new();
        /// assert!(queue.is_empty());
        /// queue.push(&1);
        /// assert!(!queue.is_empty());
        /// ```
        #[cpp(method = "bool empty() const")]
        pub fn is_empty(&self) -> bool;

        /// ```
        /// use hicc_std::QueueInt;
        /// let mut queue = QueueInt::new();
        /// assert_eq!(queue.size(), 0);
        /// queue.push(&1);
        /// assert_eq!(queue.size(), 1);
        /// ```
        #[cpp(method = "size_t size() const")]
        pub fn size(&self) -> usize;

        /// 如果为空不做任何改变.
        /// ```
        /// use hicc_std::QueueInt;
        /// let mut queue = QueueInt::new();
        /// queue.pop();
        /// queue.push(&1);
        /// assert_eq!(queue.size(), 1);
        /// queue.pop();
        /// assert_eq!(queue.size(), 0);
        /// ```
        pub fn pop(&mut self) {
            if !self.is_empty() {
                unsafe { self._pop() };
            }
        }
        #[cpp(method = "void pop()")]
        unsafe fn _pop(&mut self);

        /// ```
        /// use hicc_std::QueueInt;
        /// let mut queue = QueueInt::new();
        /// queue.push(&1);
        /// queue.push(&2);
        /// assert_eq!(queue.front(), Some(&1));
        /// ```
        #[cpp(method = "void push(const T&)")]
        pub fn push(&mut self, val: &T);

        /// ```
        /// use hicc_std::QueueInt;
        /// let mut queue = QueueInt::new();
        /// assert_eq!(queue.front(), None);
        /// queue.push(&1);
        /// assert_eq!(queue.front(), Some(&1));
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
        /// use hicc_std::QueueInt;
        /// let mut queue = QueueInt::new();
        /// assert_eq!(queue.front_mut(), None);
        /// queue.push(&1);
        /// queue.push(&3);
        /// *queue.front_mut().unwrap() += 1;
        /// assert_eq!(queue.front(), Some(&2));
        ///
        /// use hicc_std::{string, QueueString};
        /// use hicc::AbiClass;
        /// let mut queue = QueueString::new();
        /// queue.push(&string::from(c"hello"));
        /// queue.push(&string::new());
        /// assert_eq!(queue.front(), Some(string::from(c"hello").into_ref()));
        /// queue.front_mut().unwrap().write(string::from(c"world"));
        /// assert_eq!(queue.front(), Some(string::from(c"world").into_ref()));
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
        /// use hicc_std::QueueInt;
        /// let mut queue = QueueInt::new();
        /// assert_eq!(queue.back(), None);
        /// queue.push(&2);
        /// queue.push(&1);
        /// assert_eq!(queue.back(), Some(&1));
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
        /// use hicc_std::QueueInt;
        /// let mut queue = QueueInt::new();
        /// assert_eq!(queue.back(), None);
        /// queue.push(&3);
        /// queue.push(&1);
        /// *queue.back_mut().unwrap() += 1;
        /// assert_eq!(queue.back(), Some(&2));
        ///
        /// use hicc_std::{string, QueueString};
        /// use hicc::AbiClass;
        /// let mut queue = QueueString::new();
        /// queue.push(&string::new());
        /// queue.push(&string::from(c"hello"));
        /// assert_eq!(queue.back(), Some(string::from(c"hello").into_ref()));
        /// queue.back_mut().unwrap().write(string::from(c"world"));
        /// assert_eq!(queue.back(), Some(string::from(c"world").into_ref()));
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
        /// use hicc_std::QueueInt;
        /// let mut queue = QueueInt::new();
        /// queue.push(&1);
        /// let mut other = QueueInt::new();
        /// other.push(&2);
        /// assert_eq!(queue.front(), Some(&1));
        /// assert_eq!(other.front(), Some(&2));
        /// queue.swap(&mut other);
        /// assert_eq!(queue.front(), Some(&2));
        /// assert_eq!(other.front(), Some(&1));
        /// ```
        #[cpp(method = "void swap(Self&)")]
        pub fn swap(&mut self, other: &mut Self);
    }


    unsafe impl<T: hicc::AbiType + Sync> Send for queue<T> {}
    unsafe impl<T: hicc::AbiType + Sync> Sync for queue<T> {}

    #[cpp(class = "template<class T, class Container, class Compare> std::priority_queue<T, Container, Compare>")]
    pub class priority_queue<T> {
        /// ```
        /// use hicc_std::PriorityQueueInt;
        /// let mut priority_queue = PriorityQueueInt::new();
        /// assert!(priority_queue.is_empty());
        /// priority_queue.push(&1);
        /// assert!(!priority_queue.is_empty());
        /// ```
        #[cpp(method = "bool empty() const")]
        pub fn is_empty(&self) -> bool;

        /// ```
        /// use hicc_std::PriorityQueueInt;
        /// let mut priority_queue = PriorityQueueInt::new();
        /// assert_eq!(priority_queue.size(), 0);
        /// priority_queue.push(&1);
        /// assert_eq!(priority_queue.size(), 1);
        /// ```
        #[cpp(method = "size_t size() const")]
        pub fn size(&self) -> usize;

        /// 如果为空不做任何改变.
        /// ```
        /// use hicc_std::PriorityQueueInt;
        /// let mut priority_queue = PriorityQueueInt::new();
        /// priority_queue.pop();
        /// priority_queue.push(&1);
        /// assert_eq!(priority_queue.size(), 1);
        /// priority_queue.pop();
        /// assert_eq!(priority_queue.size(), 0);
        /// ```
        pub fn pop(&mut self) {
            if !self.is_empty() {
                unsafe { self._pop(); }
            }
        }
        #[cpp(method = "void pop()")]
        unsafe fn _pop(&mut self);

        /// ```
        /// use hicc_std::PriorityQueueInt;
        /// let mut priority_queue = PriorityQueueInt::new();
        /// priority_queue.push(&1);
        /// assert_eq!(priority_queue.top(), Some(&1));
        /// ```
        #[cpp(method = "void push(const T&)")]
        pub fn push(&mut self, val: &T);

        /// ```
        /// use hicc_std::PriorityQueueInt;
        /// let mut priority_queue = PriorityQueueInt::new();
        /// assert_eq!(priority_queue.top(), None);
        /// priority_queue.push(&1);
        /// priority_queue.push(&3);
        /// priority_queue.push(&2);
        /// assert_eq!(priority_queue.top(), Some(&3));
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
        /// use hicc_std::PriorityQueueInt;
        /// let mut priority_queue = PriorityQueueInt::new();
        /// priority_queue.push(&1);
        /// let mut other = PriorityQueueInt::new();
        /// other.push(&2);
        /// assert_eq!(priority_queue.top(), Some(&1));
        /// assert_eq!(other.top(), Some(&2));
        /// priority_queue.swap(&mut other);
        /// assert_eq!(priority_queue.top(), Some(&2));
        /// assert_eq!(other.top(), Some(&1));
        /// ```
        #[cpp(method = "void swap(Self&)")]
        pub fn swap(&mut self, other: &mut Self);
    }

    unsafe impl<T: hicc::AbiType + Sync> Send for priority_queue<T> {}
    unsafe impl<T: hicc::AbiType + Sync> Sync for priority_queue<T> {}
}
