hicc::cpp! {
    #include <iostream>
    #include <cstring>
    #include <cstdlib>
    #include <cstdio>

    class IntHolder {
        int value_;
    public:
        IntHolder(int value) : value_(value) {}
        ~IntHolder() {}
        int get() const { return value_; }
        const char* describe() const {
    static char buf[64];
    snprintf(buf, sizeof(buf), "IntHolder(value=%d)", value_);
    return buf;
}
    };

    class DoubleHolder {
        double value_;
    public:
        DoubleHolder(double value) : value_(value) {}
        ~DoubleHolder() {}
        double get() const { return value_; }
        const char* describe() const {
    static char buf[64];
    snprintf(buf, sizeof(buf), "DoubleHolder(value=%.5f)", value_);
    return buf;
}
    };

    class StringHolder {
        char* value_;
        int length_;
    public:
        StringHolder(const char* value) {
    value_ = strdup(value);
    length_ = strlen(value);
}
        ~StringHolder() {
    if (value_) free(value_);
}
        const char* get() const { return value_; }
        const char* describe() const {
    static char buf[256];
    snprintf(buf, sizeof(buf), "StringHolder(value=\"%s\", length=%d)", value_, length_);
    return buf;
}
    };

    IntHolder* intholder_new(int value) {
        return new IntHolder(value);
    }

    void intholder_delete(IntHolder* self) {
        if (self) delete self;
    }

    DoubleHolder* doubleholder_new(double value) {
        return new DoubleHolder(value);
    }

    void doubleholder_delete(DoubleHolder* self) {
        if (self) delete self;
    }

    StringHolder* stringholder_new(const char* value) {
        return new StringHolder(value);
    }

    void stringholder_delete(StringHolder* self) {
        if (self) delete self;
    }
}

hicc::import_class! {
    #[cpp(class = "IntHolder")]
    class IntHolder {
        #[cpp(method = "int get() const")]
        fn get(&self) -> i32;

        #[cpp(method = "const char* describe() const")]
        fn describe(&self) -> *const i8;
    }
}

hicc::import_class! {
    #[cpp(class = "DoubleHolder")]
    class DoubleHolder {
        #[cpp(method = "double get() const")]
        fn get(&self) -> f64;

        #[cpp(method = "const char* describe() const")]
        fn describe(&self) -> *const i8;
    }
}

hicc::import_class! {
    #[cpp(class = "StringHolder")]
    class StringHolder {
        #[cpp(method = "const char* get() const")]
        fn get(&self) -> *const i8;

        #[cpp(method = "const char* describe() const")]
        fn describe(&self) -> *const i8;
    }
}

hicc::import_lib! {
    #![link_name = "template_specialization"]

    class IntHolder;
    class DoubleHolder;
    class StringHolder;

    #[cpp(func = "IntHolder* intholder_new(int)")]
    fn intholder_new(value: i32) -> *mut IntHolder;

    #[cpp(func = "void intholder_delete(IntHolder* self)")]
    unsafe fn intholder_delete(self_: *mut IntHolder);

    #[cpp(func = "DoubleHolder* doubleholder_new(double)")]
    fn doubleholder_new(value: f64) -> *mut DoubleHolder;

    #[cpp(func = "void doubleholder_delete(DoubleHolder* self)")]
    unsafe fn doubleholder_delete(self_: *mut DoubleHolder);

    #[cpp(func = "StringHolder* stringholder_new(const char*)")]
    unsafe fn stringholder_new(value: *const i8) -> *mut StringHolder;

    #[cpp(func = "void stringholder_delete(StringHolder* self)")]
    unsafe fn stringholder_delete(self_: *mut StringHolder);
}

fn main() {
    println!("=== 026_template_specialization - 模板偏特化 ===\n");

    // IntHolder (通用版本)
    let ih = intholder_new(42);
    let ih_desc = unsafe { std::ffi::CStr::from_ptr(ih.describe()) };
    println!("{}", ih_desc.to_string_lossy());
    println!("  get(): {}", ih.get());
    unsafe { intholder_delete(&ih) };

    println!();

    // DoubleHolder (通用版本)
    let dh = doubleholder_new(3.14159);
    let dh_desc = unsafe { std::ffi::CStr::from_ptr(dh.describe()) };
    println!("{}", dh_desc.to_string_lossy());
    println!("  get(): {:.5}", dh.get());
    unsafe { doubleholder_delete(&dh) };

    println!();

    // StringHolder (char* 特化版本)
    let s = std::ffi::CString::new("Hello, World!").expect("CString::new failed");
    let sh = unsafe { stringholder_new(s.as_ptr()) };
    let sh_desc = unsafe { std::ffi::CStr::from_ptr(sh.describe()) };
    println!("{}", sh_desc.to_string_lossy());
    let sh_val = unsafe { std::ffi::CStr::from_ptr(sh.get()) };
    println!("  get(): {}", sh_val.to_string_lossy());
    unsafe { stringholder_delete(&sh) };

    println!("\nRust FFI: 每个模板特化是独立的结构");
    println!("通用版本: IntHolder, DoubleHolder");
    println!("偏特化: StringHolder (处理 char*)");
}


