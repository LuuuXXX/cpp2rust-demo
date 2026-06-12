hicc::cpp! {
    #include <stddef.h>
    #include <string>
    #include <iostream>
    #include <thread>
    #include <mutex>
    #include <fstream>
    #include <cstring>

    #include "raii_pattern.h"
}

hicc::import_class! {
    #[cpp(class = "Mutex", destroy = "mutex_delete")]
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
    #[cpp(class = "ScopedLock", destroy = "scoped_lock_delete")]
    pub class ScopedLock {
        #[cpp(method = "bool owns_lock() const")]
        pub fn owns_lock(&self) -> bool;
    }
}

hicc::import_class! {
    #[cpp(class = "FileLock", destroy = "file_lock_delete")]
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

    #[cpp(func = "Mutex* mutex_new()")]
    pub fn mutex_new() -> Mutex;

    #[cpp(func = "ScopedLock* scoped_lock_new(Mutex* mutex)")]
    pub unsafe fn scoped_lock_new(mutex: *mut Mutex) -> ScopedLock;

    #[cpp(func = "FileLock* file_lock_new(const char*)")]
    pub unsafe fn file_lock_new(filename: *const i8) -> FileLock;
}
