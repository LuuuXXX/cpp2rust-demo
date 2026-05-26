#include <hicc/hicc.hpp>
#line 0 R"(src/main.rs)"
#line 2
#include <cstdio>
    #include <cstring>
    #include <iostream>





    using FileDeleter = void(*)(struct FileHandle*);


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
        FileHandle(const FileHandle&) = delete;
        FileHandle& operator=(const FileHandle&) = delete;
        FileHandle(FileHandle&&) = default;
        FileHandle& operator=(FileHandle&&) = default;
        bool is_open() const { return file_ != nullptr; }
        int read(char* buffer, int size) {
            if (!file_ || !buffer) return -1;
            return static_cast<int>(std::fread(buffer, 1, size, file_));
        }
        int write(const char* data, int size) {
            if (!file_ || !data) return -1;
            return static_cast<int>(std::fwrite(data, 1, size, file_));
        }
        const char* filename() const { return filename_ ? filename_ : ""; }
        FILE* file() { return file_; }
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


    void default_file_deleter(struct FileHandle* handle) {
        if (handle) {
            FileHandle* fh = reinterpret_cast<FileHandle*>(handle);
            std::cout << "[DEFAULT] Closing file: " << (fh->filename() ? fh->filename() : "unknown") << std::endl;
            fh->close_file();
            delete fh;
        }
    }


    FileHandle* file_open(const char* filename, const char* mode, FileDeleter deleter) {
        FileHandle* handle = new FileHandle(filename, mode, deleter);
        if (!handle->is_open()) {
            delete handle;
            return nullptr;
        }
        return handle;
    }


    FileHandle* file_open_default(const char* filename, const char* mode) {
        return file_open(filename, mode, default_file_deleter);
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
#line 104
 struct FileHandle_104;
#line 104
namespace hicc { template<> struct MethodsType<FileHandle, void> { typedef FileHandle_104 methods_type; }; }
#line 104
 struct FileHandle_104 {
#line 104
typedef FileHandle Self; typedef void SelfContainer; typedef FileHandle_104 SelfMethods;
#line 106
static void _hicc_test_106() { bool (Self::* _106)() const = &Self::is_open; (void)_106; }
#line 106
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((bool (Self::*)() const)&Self::is_open));
#line 109
static void _hicc_test_109() { int (Self::* _109)(char* buffer, int size) = &Self::read; (void)_109; }
#line 109
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((int (Self::*)(char* buffer, int size))&Self::read));
#line 112
static void _hicc_test_112() { int (Self::* _112)(const char* data, int size) = &Self::write; (void)_112; }
#line 112
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((int (Self::*)(const char* data, int size))&Self::write));
#line 115
static void _hicc_test_115() { const char* (Self::* _115)() const = &Self::filename; (void)_115; }
#line 115
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((const char* (Self::*)() const)&Self::filename));
#line 104
};
#line 121
EXPORT_METHODS_BEG(custom_deleter) {
#line 124
static void _hicc_test_124() { FileHandle* (* _124)(const char* filename, const char* mode) = &file_open_default; (void)_124; }
#line 124
EXPORT_METHOD_IN(void, ExportMethods, ((FileHandle* (*)(const char* filename, const char* mode))&file_open_default));
#line 126
static void _hicc_test_126() { void (* _126)(FileHandle* handle) = &file_close; (void)_126; }
#line 126
EXPORT_METHOD_IN(void, ExportMethods, ((void (*)(FileHandle* handle))&file_close));
#line 128
static void _hicc_test_128() { int (* _128)(FileHandle* handle, char* buffer, int size) = &file_read; (void)_128; }
#line 128
EXPORT_METHOD_IN(void, ExportMethods, ((int (*)(FileHandle* handle, char* buffer, int size))&file_read));
#line 130
static void _hicc_test_130() { int (* _130)(FileHandle* handle, const char* data, int size) = &file_write; (void)_130; }
#line 130
EXPORT_METHOD_IN(void, ExportMethods, ((int (*)(FileHandle* handle, const char* data, int size))&file_write));
#line 121
} EXPORT_METHODS_END();

