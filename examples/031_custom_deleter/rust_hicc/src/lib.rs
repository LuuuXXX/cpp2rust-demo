hicc::cpp! {
    #include <iostream>
    #include <cstdio>
    #include <cstring>

    #include "custom_deleter.h"

    std::unique_ptr<FileHandle> file_handle_new_default(const char* filename, const char* mode) {
        return std::unique_ptr<FileHandle>(new FileHandle(filename, mode, default_file_deleter));
    }
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

    #[cpp(func = "std::unique_ptr<FileHandle> file_handle_new_default(const char*, const char*)")]
    pub unsafe fn file_handle_new_default(filename: *const i8, mode: *const i8) -> FileHandle;
}
