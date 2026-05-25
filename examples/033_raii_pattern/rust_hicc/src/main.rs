hicc::cpp! {
    #include <iostream>
    #include <thread>
    #include <mutex>
    #include <fstream>
    #include <cstring>

    // 互斥锁结构
    class Mutex {
        std::mutex mtx_;
        std::string name_;
    public:
        Mutex() : mtx_(), name_("unnamed") {}
        explicit Mutex(const char* name) : mtx_(), name_(name ? name : "unnamed") {}
        ~Mutex() {}
        void lock() { mtx_.lock(); }
        void unlock() { mtx_.unlock(); }
        bool try_lock() { return mtx_.try_lock(); }
        const char* name() const { return name_.c_str(); }
    };

    // 作用域锁/守卫
    class ScopedLock {
        Mutex* mutex_;
        bool owns_lock_;
    public:
        explicit ScopedLock(Mutex* m) : mutex_(m), owns_lock_(false) {
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
        ScopedLock(const ScopedLock&) = delete;
        ScopedLock& operator=(const ScopedLock&) = delete;
    };

    // 文件锁结构
    class FileLock {
        std::mutex mtx_;
        std::ofstream file_;
        std::string filename_;
    public:
        explicit FileLock(const char* fname) : mtx_(), file_(), filename_(fname ? fname : "") {
            if (!filename_.empty()) {
                file_.open(filename_, std::ios::app);
            }
        }
        ~FileLock() {
            if (file_.is_open()) {
                file_.close();
            }
        }
        FileLock(const FileLock&) = delete;
        FileLock& operator=(const FileLock&) = delete;
        void lock() { mtx_.lock(); }
        void unlock() { mtx_.unlock(); }
        const char* filename() const { return filename_.c_str(); }
    };

    // FFI wrapper functions
    Mutex* mutex_new() {
        return new Mutex();
    }

    void mutex_delete(Mutex* self_) {
        if (self_) {
            std::cout << "Mutex '" << self_->name() << "' deleted" << std::endl;
            delete self_;
        }
    }

    void mutex_lock(Mutex* self_) {
        self_->lock();
    }

    void mutex_unlock(Mutex* self_) {
        self_->unlock();
    }

    int mutex_try_lock(Mutex* self_) {
        return self_->try_lock() ? 1 : 0;
    }

    ScopedLock* scoped_lock_new(Mutex* mutex) {
        return new ScopedLock(mutex);
    }

    void scoped_lock_delete(ScopedLock* self_) {
        if (self_) {
            delete self_;
        }
    }

    FileLock* file_lock_new(const char* filename) {
        return new FileLock(filename);
    }

    void file_lock_delete(FileLock* self_) {
        if (self_) {
            delete self_;
        }
    }

    void file_lock_lock(FileLock* self_) {
        self_->lock();
    }

    void file_lock_unlock(FileLock* self_) {
        self_->unlock();
    }
}

hicc::import_class! {
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
    }

    #[cpp(class = "ScopedLock")]
    class ScopedLock {
        // ScopedLock RAII 自动解锁
    }

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
    #[cpp(func = "Mutex* mutex_new()")]
    fn mutex_new() -> *mut Mutex;
    #[cpp(func = "void mutex_delete(Mutex* self_)")]
    unsafe fn mutex_delete(self_: *mut Mutex);
    #[cpp(func = "void mutex_lock(Mutex* self_)")]
    fn mutex_lock(self_: *mut Mutex);
    #[cpp(func = "void mutex_unlock(Mutex* self_)")]
    fn mutex_unlock(self_: *mut Mutex);
    #[cpp(func = "int mutex_try_lock(Mutex* self_)")]
    fn mutex_try_lock(self_: *mut Mutex) -> i32;

    class ScopedLock;
    #[cpp(func = "ScopedLock* scoped_lock_new(Mutex* mutex)")]
    fn scoped_lock_new(mutex: *mut Mutex) -> *mut ScopedLock;
    #[cpp(func = "void scoped_lock_delete(ScopedLock* self_)")]
    unsafe fn scoped_lock_delete(self_: *mut ScopedLock);

    class FileLock;
    #[cpp(func = "FileLock* file_lock_new(const char* filename)")]
    fn file_lock_new(filename: *const i8) -> *mut FileLock;
    #[cpp(func = "void file_lock_delete(FileLock* self_)")]
    unsafe fn file_lock_delete(self_: *mut FileLock);
    #[cpp(func = "void file_lock_lock(FileLock* self_)")]
    fn file_lock_lock(self_: *mut FileLock);
    #[cpp(func = "void file_lock_unlock(FileLock* self_)")]
    fn file_lock_unlock(self_: *mut FileLock);
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
    let lock = scoped_lock_new(&mutex2);
    println!("Inside scoped lock region");
    println!("ScopedLock will auto-unlock on delete");
    unsafe { scoped_lock_delete(&lock) };
    unsafe { mutex_delete(&mutex2) };

    println!();

    // FileLock 示例
    println!("--- FileLock Demo ---");
    let filename = std::ffi::CString::new("raii_test.txt").expect("CString::new failed");
    let mut file_lock = file_lock_new(filename.as_ptr());
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