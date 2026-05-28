hicc::cpp! {
    #include <stddef.h>
    #include <string>
    #include <iostream>
    #include <thread>
    #include <mutex>
    #include <fstream>
    #include <cstring>

    class Mutex {
        std::mutex mtx_;
        std::string name_;
    public:
        Mutex() : mtx_(), name_("unnamed") {
}
        Mutex(const char* name) : mtx_(), name_(name ? name : "unnamed") {
}
        ~Mutex() {
}
        void lock() {
    mtx_.lock();
}
        void unlock() {
    mtx_.unlock();
}
        bool try_lock() {
    return mtx_.try_lock();
}
        const char* name() const {
    return name_.c_str();
}
    };

    class ScopedLock {
        Mutex* mutex_;
        bool owns_lock_;
    public:
        ScopedLock(Mutex* m) : mutex_(m), owns_lock_(false) {
    if (mutex_) {
        mutex_->lock();
        owns_lock_ = true;
    }
}
        ~ScopedLock() {
    if (owns_lock_ && mutex_) {
        mutex_->unlock();
    }
}
        ScopedLock(ScopedLock&& other) noexcept : mutex_(other.mutex_), owns_lock_(other.owns_lock_) {
    other.owns_lock_ = false;
}
        ScopedLock(const ScopedLock &) = default;
        ScopedLock & operator=(const ScopedLock &) {}
    };

    class FileLock {
        std::mutex mtx_;
        std::ofstream file_;
        std::string filename_;
    public:
        FileLock(const char* fname) : mtx_(), file_(), filename_(fname ? fname : "") {
    if (!filename_.empty()) {
        file_.open(filename_, std::ios::app);
    }
}
        ~FileLock() {
    if (file_.is_open()) {
        file_.close();
    }
}
        FileLock(const FileLock &) = default;
        FileLock & operator=(const FileLock &) {}
        void lock() {
    mtx_.lock();
}
        void unlock() {
    mtx_.unlock();
}
        const char* filename() const {
    return filename_.c_str();
}
    };

    Mutex* mutex_new() {
        return new Mutex();
    }

    void mutex_delete(Mutex* self) {
        if (self) {
            std::cout << "Mutex '" << self->name() << "' deleted" << std::endl;
            delete self;
        }
    }

    ScopedLock* scoped_lock_new(Mutex* mutex) {
        return new ScopedLock(mutex);
    }

    void scoped_lock_delete(ScopedLock* self) {
        if (self) {
            delete self;
        }
    }

    FileLock* file_lock_new(const char* filename) {
        return new FileLock(filename);
    }

    void file_lock_delete(FileLock* self) {
        if (self) {
            delete self;
        }
    }
}

hicc::import_class! {
    #[cpp(class = "FileLock")]
    class FileLock {
        #[cpp(method = "void lock()")]
        fn lock(&mut self);

        #[cpp(method = "void unlock()")]
        fn unlock(&mut self);

        #[cpp(method = "const char* filename() const")]
        fn filename(&self) -> *const i8;
    }
}

hicc::import_lib! {
    #![link_name = "raii_pattern"]

    class Mutex;
    class ScopedLock;
    class FileLock;

    #[cpp(class = "Mutex")]
    class Mutex {
        #[cpp(method = "void lock()")]
        fn lock(&mut self);

        #[cpp(method = "void unlock()")]
        fn unlock(&mut self);

        #[cpp(method = "bool try_lock()")]
        fn try_lock(&mut self) -> bool;

        #[cpp(method = "const char* name() const")]
        fn name(&self) -> *const i8;

        #[cpp(func = "Mutex* mutex_new()")]
        fn new() -> *mut Mutex;

        #[cpp(func = "void mutex_delete(Mutex* self)")]
        unsafe fn delete(self_: *mut Mutex);
    }

    #[cpp(func = "ScopedLock* scoped_lock_new(Mutex* mutex)")]
    unsafe fn scoped_lock_new(mutex: *mut Mutex) -> *mut ScopedLock;

    #[cpp(func = "void scoped_lock_delete(ScopedLock* self)")]
    unsafe fn scoped_lock_delete(self_: *mut ScopedLock);

    #[cpp(func = "FileLock* file_lock_new(const char*)")]
    unsafe fn file_lock_new(filename: *const i8) -> *mut FileLock;

    #[cpp(func = "void file_lock_delete(FileLock* self)")]
    unsafe fn file_lock_delete(self_: *mut FileLock);
}

fn main() {
    println!("=== 033_raii_pattern - RAII 模式 ===\n");

    // 手动锁定/解锁示例
    println!("--- Manual Lock/Unlock ---");
    let mut mutex = unsafe { mutex_new() };
    mutex.lock();
    println!("Critical section started");
    println!("Critical section ended");
    mutex.unlock();
    unsafe { mutex_delete(&mutex) };

    println!();

    // ScopedLock 示例（模拟 RAII 自动解锁）
    println!("--- ScopedLock Demo ---");
    let mutex2 = unsafe { mutex_new() };
    let lock = unsafe { scoped_lock_new(&mutex2) };
    println!("Inside scoped lock region");
    println!("ScopedLock will auto-unlock on delete");
    unsafe { scoped_lock_delete(&lock) };
    unsafe { mutex_delete(&mutex2) };

    println!();

    // FileLock 示例
    println!("--- FileLock Demo ---");
    let filename = std::ffi::CString::new("raii_test.txt").expect("CString::new failed");
    let mut file_lock = unsafe { file_lock_new(filename.as_ptr()) };
    file_lock.lock();
    println!("File is locked, performing I/O...");
    file_lock.unlock();
    unsafe { file_lock_delete(&file_lock) };

    println!("\nRust FFI: RAII 模式映射");
    println!("1. C++ RAII: 构造函数加锁，析构函数解锁");
    println!("2. Rust 等效: Drop trait 自动调用");
    println!("3. FFI 边界: ScopedLock 对象在 Rust 析构时自动释放");
    println!("4. 推荐模式: Rust 封装 RAII guard 类型");
}


