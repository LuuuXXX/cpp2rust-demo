#pragma once

#ifdef __cplusplus
extern "C" {
#endif

// 自定义删除器示例
// 展示如何将 C++ 的自定义删除器通过 FFI 传递给 Rust

// 文件句柄结构体
struct FileHandle;

// 文件删除器函数类型
typedef void (*FileDeleter)(struct FileHandle*);

// 创建文件句柄，使用原始函数指针参数（会被 hicc (*)过滤器排除在 import_lib! 之外）
FileHandle* file_open(const char* filename, const char* mode, void (*deleter)(struct FileHandle*));

// 关闭文件句柄
void file_close(FileHandle* handle);

// 读取文件
int file_read(FileHandle* handle, char* buffer, int size);

// 写入文件
int file_write(FileHandle* handle, const char* data, int size);

// 创建使用默认删除器的文件句柄
FileHandle* file_open_default(const char* filename, const char* mode);

// 通用删除器函数
void default_file_deleter(struct FileHandle* handle);

// 带日志的删除器
void logging_file_deleter(struct FileHandle* handle);

// 引用计数删除器
void refcounted_file_deleter(struct FileHandle* handle);

#ifdef __cplusplus
}

// Full class definition - for hicc code generation
#include <cstdio>

using FileDeleter = void(*)(struct FileHandle*);

class FileHandle {
    FILE* file_;
    FileDeleter deleter_;
    const char* filename_;
public:
    FileHandle(const char* filename, const char* mode, FileDeleter deleter);
    ~FileHandle();
    FileHandle(const FileHandle&) = delete;
    FileHandle& operator=(const FileHandle&) = delete;
    FileHandle(FileHandle&&) = default;
    FileHandle& operator=(FileHandle&&) = default;
    bool is_open() const;
    int read(char* buffer, int size);
    int write(const char* data, int size);
    const char* filename() const;
    void close_file();
    void invoke_deleter();
};

#endif
