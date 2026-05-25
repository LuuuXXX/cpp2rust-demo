#include "raii_pattern.h"
#include <iostream>
#include <thread>
#include <mutex>
#include <fstream>
#include <cstring>

// Mutex class implementation
Mutex::Mutex() : mtx_(), name_("unnamed") {
}

Mutex::Mutex(const char* name) : mtx_(), name_(name ? name : "unnamed") {
}

Mutex::~Mutex() {
}

void Mutex::lock() {
    mtx_.lock();
}

void Mutex::unlock() {
    mtx_.unlock();
}

bool Mutex::try_lock() {
    return mtx_.try_lock();
}

const char* Mutex::name() const {
    return name_.c_str();
}

// ScopedLock class implementation
ScopedLock::ScopedLock(Mutex* m) : mutex_(m), owns_lock_(false) {
    if (mutex_) {
        mutex_->lock();
        owns_lock_ = true;
    }
}

ScopedLock::~ScopedLock() {
    if (owns_lock_ && mutex_) {
        mutex_->unlock();
    }
}

ScopedLock::ScopedLock(ScopedLock&& other) noexcept : mutex_(other.mutex_), owns_lock_(other.owns_lock_) {
    other.owns_lock_ = false;
}

// FileLock class implementation
FileLock::FileLock(const char* fname) : mtx_(), file_(), filename_(fname ? fname : "") {
    if (!filename_.empty()) {
        file_.open(filename_, std::ios::app);
    }
}

FileLock::~FileLock() {
    if (file_.is_open()) {
        file_.close();
    }
}

void FileLock::lock() {
    mtx_.lock();
}

void FileLock::unlock() {
    mtx_.unlock();
}

const char* FileLock::filename() const {
    return filename_.c_str();
}

// FFI wrapper functions
Mutex* mutex_new(void) {
    return new Mutex();
}

void mutex_delete(Mutex* self) {
    if (self) {
        std::cout << "Mutex '" << self->name() << "' deleted" << std::endl;
        delete self;
    }
}

void mutex_lock(Mutex* self) {
    self->lock();
}

void mutex_unlock(Mutex* self) {
    self->unlock();
}

int mutex_try_lock(Mutex* self) {
    return self->try_lock() ? 1 : 0;
}

ScopedLock* scoped_lock_new(Mutex* mutex) {
    return new ScopedLock(mutex);
}

void scoped_lock_delete(ScopedLock* self) {
    if (self) {
        delete self;
    }
}

FileLock* file_lock_new(const char* filename) {
    return new FileLock(filename);
}

void file_lock_delete(FileLock* self) {
    if (self) {
        delete self;
    }
}

void file_lock_lock(FileLock* self) {
    self->lock();
}

void file_lock_unlock(FileLock* self) {
    self->unlock();
}
