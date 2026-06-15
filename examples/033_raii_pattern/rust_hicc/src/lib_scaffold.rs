hicc::cpp! {
    #include <string>
    #include <iostream>
    #include <thread>
    #include <mutex>
    #include <fstream>
    #include <cstring>

    #include "raii_pattern.h"
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

    #[cpp(func = "std::unique_ptr<Mutex> hicc::make_unique<Mutex>()")]
    pub fn mutex_new() -> Mutex;

    #[cpp(func = "std::unique_ptr<Mutex> std::make_unique<Mutex>(const char*)")]
    pub unsafe fn mutex_new_with_name(name: *const i8) -> Mutex;

    #[cpp(func = "std::unique_ptr<ScopedLock> std::make_unique<ScopedLock>(Mutex*)")]
    pub unsafe fn scoped_lock_new_with_m(m: *mut Mutex) -> ScopedLock;

    #[cpp(func = "std::unique_ptr<FileLock> std::make_unique<FileLock>(const char*)")]
    pub unsafe fn file_lock_new_with_fname(fname: *const i8) -> FileLock;
}
