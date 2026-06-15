hicc::cpp! {
    #include <string>
    #include <iostream>
    #include <thread>
    #include <mutex>
    #include <fstream>
    #include <cstring>

    #include "raii_pattern.h"

    extern "C" {
        Mutex* hicc_mutex_new() { return new Mutex(); }
        Mutex* hicc_mutex_new_with_name(const char* name) { return new Mutex(name); }
        ScopedLock* hicc_scoped_lock_new(Mutex* m) { return new ScopedLock(m); }
        FileLock* hicc_file_lock_new(const char* fname) { return new FileLock(fname); }
    }
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

    #[cpp(func = "Mutex* hicc_mutex_new()")]
    pub fn mutex_new() -> Mutex;

    #[cpp(func = "Mutex* hicc_mutex_new_with_name(const char*)")]
    pub unsafe fn mutex_new_with_name(name: *const i8) -> Mutex;

    #[cpp(func = "ScopedLock* hicc_scoped_lock_new(Mutex*)")]
    pub unsafe fn scoped_lock_new(mutex: *mut Mutex) -> ScopedLock;

    #[cpp(func = "FileLock* hicc_file_lock_new(const char*)")]
    pub unsafe fn file_lock_new(filename: *const i8) -> FileLock;
}
