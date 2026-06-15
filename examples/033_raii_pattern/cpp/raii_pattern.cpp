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
