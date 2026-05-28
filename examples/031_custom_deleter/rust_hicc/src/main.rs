hicc::cpp! {
    #include <iostream>
    #include <cstdio>
    #include <cstring>

    class FileHandle;
    typedef void (*FileDeleter)(FileHandle*);

    class FileHandle {
        FILE* file_;
        FileDeleter deleter_;
        const char* filename_;
    public:
        FileHandle(const char* filename, const char* mode, FileDeleter deleter)
    : file_(nullptr), deleter_(deleter), filename_(filename) {
    if (filename && mode) {
        file_ = std::fopen(filename, mode);
    }
}
        ~FileHandle() {
    if (file_) {
        std::fclose(file_);
        file_ = nullptr;
    }
}
        FileHandle(const FileHandle &) = default;
        FileHandle & operator=(const FileHandle &) {}
        FileHandle(FileHandle &&) = default;
        FileHandle & operator=(FileHandle &&) = default;
        bool is_open() const {
    return file_ != nullptr;
}
        int read(char* buffer, int size) {
    if (!file_ || !buffer) return -1;
    return static_cast<int>(std::fread(buffer, 1, size, file_));
}
        int write(const char* data, int size) {
    if (!file_ || !data) return -1;
    return static_cast<int>(std::fwrite(data, 1, size, file_));
}
        const char* filename() const {
    return filename_ ? filename_ : "";
}
        void close_file() {
    if (file_) {
        std::fclose(file_);
        file_ = nullptr;
    }
}
        void invoke_deleter() {
    if (deleter_) {
        deleter_(this);
    }
}
    };

    void default_file_deleter(FileHandle* handle);

    FileHandle* file_open(const char* filename, const char* mode, FileDeleter deleter) {
        FileHandle* handle = new FileHandle(filename, mode, deleter);
        if (!handle->is_open()) {
            delete handle;
            return nullptr;
        }
        return handle;
    }

    void file_close(FileHandle* handle) {
        if (handle) {
            FileHandle* fh = reinterpret_cast<FileHandle*>(handle);
            fh->invoke_deleter();
        }
    }

    int file_read(FileHandle* handle, char* buffer, int size) {
        if (!handle) return -1;
        FileHandle* fh = reinterpret_cast<FileHandle*>(handle);
        return fh->read(buffer, size);
    }

    int file_write(FileHandle* handle, const char* data, int size) {
        if (!handle) return -1;
        FileHandle* fh = reinterpret_cast<FileHandle*>(handle);
        return fh->write(data, size);
    }

    FileHandle* file_open_default(const char* filename, const char* mode) {
        return file_open(filename, mode, default_file_deleter);
    }

    void default_file_deleter(FileHandle* handle) {
        if (handle) {
            FileHandle* fh = reinterpret_cast<FileHandle*>(handle);
            std::cout << "[DEFAULT] Closing file: " << (fh->filename() ? fh->filename() : "unknown") << std::endl;
            fh->close_file();
            delete fh;
        }
    }

    void logging_file_deleter(FileHandle* handle) {
        if (handle) {
            FileHandle* fh = reinterpret_cast<FileHandle*>(handle);
            std::cout << "[LOG] Custom deleter: Closing file with logging: "
                      << (fh->filename() ? fh->filename() : "unknown") << std::endl;
            fh->close_file();
            delete fh;
        }
    }

    void refcounted_file_deleter(FileHandle* handle) {
        if (handle) {
            FileHandle* fh = reinterpret_cast<FileHandle*>(handle);
            std::cout << "[REF] Reference counted close: "
                      << (fh->filename() ? fh->filename() : "unknown") << std::endl;
            fh->close_file();
            delete fh;
        }
    }
}

hicc::import_class! {
    #[cpp(class = "FileHandle")]
    class FileHandle {
        #[cpp(method = "bool is_open() const")]
        fn is_open(&self) -> bool;

        #[cpp(method = "int read(char* buffer, int size)")]
        fn read(&mut self, buffer: *mut i8, size: i32) -> i32;

        #[cpp(method = "int write(const char* data, int size)")]
        fn write(&mut self, data: *const i8, size: i32) -> i32;

        #[cpp(method = "const char* filename() const")]
        fn filename(&self) -> *const i8;

        #[cpp(method = "void close_file()")]
        fn close_file(&mut self);

        #[cpp(method = "void invoke_deleter()")]
        fn invoke_deleter(&mut self);
    }
}

hicc::import_lib! {
    #![link_name = "custom_deleter"]

    class FileHandle;

    #[cpp(func = "void file_close(FileHandle* handle)")]
    unsafe fn file_close(handle: *mut FileHandle);

    #[cpp(func = "int file_read(FileHandle* handle, char*, int)")]
    unsafe fn file_read(handle: *mut FileHandle, buffer: *mut i8, size: i32) -> i32;

    #[cpp(func = "int file_write(FileHandle* handle, const char*, int)")]
    unsafe fn file_write(handle: *mut FileHandle, data: *const i8, size: i32) -> i32;

    #[cpp(func = "FileHandle* file_open_default(const char*, const char*)")]
    unsafe fn file_open_default(filename: *const i8, mode: *const i8) -> *mut FileHandle;

    #[cpp(func = "void default_file_deleter(FileHandle* handle)")]
    unsafe fn default_file_deleter(handle: *mut FileHandle);

    #[cpp(func = "void logging_file_deleter(FileHandle* handle)")]
    unsafe fn logging_file_deleter(handle: *mut FileHandle);

    #[cpp(func = "void refcounted_file_deleter(FileHandle* handle)")]
    unsafe fn refcounted_file_deleter(handle: *mut FileHandle);
}

fn main() {
    println!("=== 031_custom_deleter - 自定义删除器 ===\n");

    // 使用默认删除器
    let filename = std::ffi::CString::new("test_default.txt").expect("CString::new failed");
    let mode = std::ffi::CString::new("w").expect("CString::new failed");

    let mut handle = unsafe { file_open_default(filename.as_ptr(), mode.as_ptr()) };

    // 写入数据
    let data = std::ffi::CString::new("Hello, custom deleter!").expect("CString::new failed");
    let written = handle.write(data.as_ptr(), data.to_bytes().len() as i32);
    println!("Written {} bytes", written);

    // 关闭文件
    handle.close_file();

    println!("\nRust FFI: 自定义删除器模式");
    println!("1. C++ 允许传递函数指针作为删除器");
    println!("2. 删除器在对象销毁时自动调用");
    println!("3. Rust 可以传入自己的清理函数");
    println!("4. 适用于文件、内存、网络连接等资源");
}


