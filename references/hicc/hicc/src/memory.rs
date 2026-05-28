crate::import_class! {
    #![in_hicc]

    /// 对应`std::shared_ptr<T>`
    pub class shared_ptr<T> {
        pub fn get(&self) -> *mut T;
        /// 对应`std::weak_ptr<T>`的构造函数.
        pub fn weak(&self) -> weak_ptr<T>;
        /// `std::shared_ptr<T>`内部否为空指针.
        ///
        /// 注意`AbiClass::is_null`判断的是`rust`持有的`std::shared_ptr<T>*`是否为空
        pub fn is_empty(&self) -> bool;
    }

    unsafe impl<T: crate::AbiType + Sync> Send for shared_ptr<T> {}
    unsafe impl<T: crate::AbiType + Sync> Sync for shared_ptr<T> {}

    /// 对应`std::weak_ptr<T>`.
    pub class weak_ptr<T> {
        pub fn expired(&self) -> bool;
        pub fn lock(&self) -> shared_ptr<T>;
    }

    unsafe impl<T: crate::AbiType + Sync> Send for weak_ptr<T> {}
    unsafe impl<T: crate::AbiType + Sync> Sync for weak_ptr<T> {}

    /// 对应`std::unique_ptr<T, D>`
    ///
    /// 如果`T`是`c++ class`类型, 且`c++`端的`D`是缺省模板参数, 则会被映射为`T`而非`std::unique_ptr<T>`.
    pub class unique_ptr<T> {
        pub fn get(&self) -> *mut T;
        /// `std::unique_ptr<T, D>`内部否为空指针.
        ///
        /// 注意`AbiClass::is_null`判断的是`rust`持有的`std::unique_ptr<T, D>*`是否为空
        pub fn is_empty(&self) -> bool;
    }

    unsafe impl<T: crate::AbiType + Sync> Send for unique_ptr<T> {}
    unsafe impl<T: crate::AbiType + Sync> Sync for unique_ptr<T> {}
}
