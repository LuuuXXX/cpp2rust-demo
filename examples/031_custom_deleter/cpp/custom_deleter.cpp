#include "custom_deleter.h"
#include <iostream>
#include <cstdio>
#include <cstring>

// FileHandle class implementation
FileHandle::FileHandle(const char* filename, const char* mode, FileDeleter deleter)
    : file_(nullptr), deleter_(deleter), filename_(filename) {
    if (filename && mode) {
        file_ = std::fopen(filename, mode);
    }
}

FileHandle::~FileHandle() {
    if (file_) {
        std::fclose(file_);
        file_ = nullptr;
    }
}

bool FileHandle::is_open() const {
    return file_ != nullptr;
}

int FileHandle::read(char* buffer, int size) {
    if (!file_ || !buffer) return -1;
    return static_cast<int>(std::fread(buffer, 1, size, file_));
}

int FileHandle::write(const char* data, int size) {
    if (!file_ || !data) return -1;
    return static_cast<int>(std::fwrite(data, 1, size, file_));
}

const char* FileHandle::filename() const {
    return filename_ ? filename_ : "";
}

void FileHandle::close_file() {
    if (file_) {
        std::fclose(file_);
        file_ = nullptr;
    }
}

void FileHandle::invoke_deleter() {
    if (deleter_) {
        deleter_(this);
    }
}

// Default deleter implementation
void default_file_deleter(struct FileHandle* handle) {
    if (handle) {
        FileHandle* fh = reinterpret_cast<FileHandle*>(handle);
        std::cout << "[DEFAULT] Closing file: " << (fh->filename() ? fh->filename() : "unknown") << std::endl;
        fh->close_file();
        delete fh;
    }
}

// Custom deleter with logging
void logging_file_deleter(struct FileHandle* handle) {
    if (handle) {
        FileHandle* fh = reinterpret_cast<FileHandle*>(handle);
        std::cout << "[LOG] Custom deleter: Closing file with logging: "
                  << (fh->filename() ? fh->filename() : "unknown") << std::endl;
        fh->close_file();
        delete fh;
    }
}

// Reference counted deleter
void refcounted_file_deleter(struct FileHandle* handle) {
    if (handle) {
        FileHandle* fh = reinterpret_cast<FileHandle*>(handle);
        std::cout << "[REF] Reference counted close: "
                  << (fh->filename() ? fh->filename() : "unknown") << std::endl;
        fh->close_file();
        delete fh;
    }
}

// FFI wrapper functions
FileHandle* file_open(const char* filename, const char* mode, void (*deleter)(struct FileHandle*)) {
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
