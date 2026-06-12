hicc::cpp! {
    #include <iostream>
    #include <cstdio>
    #include <cstring>

    #include "custom_deleter.h"

    typedef void (*FileDeleter)(struct FileHandle*);
}

hicc::import_class! {
    #[cpp(class = "FileHandle", destroy = "refcounted_file_deleter")]
    pub class FileHandle {
        #[cpp(method = "bool is_open() const")]
        pub fn is_open(&self) -> bool;

        #[cpp(method = "int read(char* buffer, int size)")]
        pub fn read(&mut self, buffer: *mut i8, size: i32) -> i32;

        #[cpp(method = "int write(const char* data, int size)")]
        pub fn write(&mut self, data: *const i8, size: i32) -> i32;

        #[cpp(method = "const char* filename() const")]
        pub fn filename(&self) -> *const i8;

        #[cpp(method = "void close_file()")]
        pub fn close_file(&mut self);

        #[cpp(method = "void invoke_deleter()")]
        pub fn invoke_deleter(&mut self);
    }
}

hicc::import_lib! {
    #![link_name = "custom_deleter"]

    class FileHandle;

    // cpp2rust-todo[FP]: 含函数指针参数，需确保回调符合 extern "C" 调用约定
    #[cpp(func = "FileHandle* file_open(const char*, const char*, void (*)(FileHandle*))")]
    pub unsafe fn file_open(filename: *const i8, mode: *const i8, deleter: unsafe extern "C" fn(*mut FileHandle)) -> *mut FileHandle;

    #[cpp(func = "void file_close(FileHandle* handle)")]
    pub unsafe fn file_close(handle: *mut FileHandle);

    #[cpp(func = "int file_read(FileHandle* handle, char*, int)")]
    pub unsafe fn file_read(handle: *mut FileHandle, buffer: *mut i8, size: i32) -> i32;

    #[cpp(func = "int file_write(FileHandle* handle, const char*, int)")]
    pub unsafe fn file_write(handle: *mut FileHandle, data: *const i8, size: i32) -> i32;

    #[cpp(func = "FileHandle* file_open_default(const char*, const char*)")]
    pub unsafe fn file_open_default(filename: *const i8, mode: *const i8) -> *mut FileHandle;
}
