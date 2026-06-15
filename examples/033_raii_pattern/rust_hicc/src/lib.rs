hicc::cpp! {
    #include <string>
    #include <iostream>
    #include <thread>
    #include <mutex>
    #include <fstream>
    #include <cstring>

    #include "raii_pattern.h"

    std::unique_ptr<Mutex> _cpp2rust_make_unique_mutex_0() { return std::make_unique<Mutex>(); }
    std::unique_ptr<Mutex> _cpp2rust_make_unique_mutex_with_name(const char* name) { return std::make_unique<Mutex>(name); }
    std::unique_ptr<ScopedLock> _cpp2rust_make_unique_scoped_lock_with_m(Mutex* m) { return std::make_unique<ScopedLock>(m); }
    std::unique_ptr<FileLock> _cpp2rust_make_unique_file_lock_with_fname(const char* fname) { return std::make_unique<FileLock>(fname); }
}

hicc::import_class! {
    #[cpp(class = "Mutex")]
    pub class Mutex {
        #[cpp(method = "void lock()")]
        pub fn lock(&mut self);

        #[cpp(method = "void unlock()")]
        pub fn unlock(&mut self);

        #[cpp(method = "bool try_lock()")]
        pub fn try_lock(&mut self) -> bool;

        #[cpp(method = "const char* name() const")]
        pub fn name(&self) -> *const i8;
    }
}

hicc::import_class! {
    #[cpp(class = "ScopedLock")]
    pub class ScopedLock {
        #[cpp(method = "bool owns_lock() const")]
        pub fn owns_lock(&self) -> bool;
    }
}

hicc::import_class! {
    #[cpp(class = "FileLock")]
    pub class FileLock {
        #[cpp(method = "void lock()")]
        pub fn lock(&mut self);

        #[cpp(method = "void unlock()")]
        pub fn unlock(&mut self);

        #[cpp(method = "const char* filename() const")]
        pub fn filename(&self) -> *const i8;
    }
}

hicc::import_lib! {
    #![link_name = "raii_pattern"]

    class Mutex;
    class ScopedLock;
    class FileLock;

    #[cpp(func = "std::unique_ptr<Mutex> _cpp2rust_make_unique_mutex_0()")]
    pub fn mutex_new() -> Mutex;

    #[cpp(func = "std::unique_ptr<Mutex> _cpp2rust_make_unique_mutex_with_name(const char*)")]
    pub unsafe fn mutex_new_with_name(name: *const i8) -> Mutex;

    #[cpp(func = "std::unique_ptr<ScopedLock> _cpp2rust_make_unique_scoped_lock_with_m(Mutex*)")]
    pub unsafe fn scoped_lock_new(mutex: *mut Mutex) -> ScopedLock;

    #[cpp(func = "std::unique_ptr<FileLock> _cpp2rust_make_unique_file_lock_with_fname(const char*)")]
    pub unsafe fn file_lock_new(filename: *const i8) -> FileLock;
}
