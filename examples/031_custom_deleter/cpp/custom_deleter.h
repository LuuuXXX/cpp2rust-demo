#pragma once

#include <cstdio>

#ifdef __cplusplus
extern "C" {
#endif

struct FileHandle;

void default_file_deleter(struct FileHandle* handle);
void logging_file_deleter(struct FileHandle* handle);
void refcounted_file_deleter(struct FileHandle* handle);

#ifdef __cplusplus
}
#endif

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
