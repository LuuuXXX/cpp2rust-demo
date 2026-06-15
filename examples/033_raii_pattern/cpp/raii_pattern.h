#pragma once

#include <mutex>
#include <fstream>
#include <string>

class Mutex {
    std::mutex mtx_;
    std::string name_;
public:
    explicit Mutex();
    explicit Mutex(const char* name);
    ~Mutex();
    void lock();
    void unlock();
    bool try_lock();
    const char* name() const;
};

class ScopedLock {
    Mutex* mutex_;
    bool owns_lock_;
public:
    explicit ScopedLock(Mutex* m);
    ~ScopedLock();
    ScopedLock(ScopedLock&& other) noexcept;
    ScopedLock(const ScopedLock&) = delete;
    ScopedLock& operator=(const ScopedLock&) = delete;
    bool owns_lock() const { return owns_lock_; }
};

class FileLock {
    std::mutex mtx_;
    std::ofstream file_;
    std::string filename_;
public:
    explicit FileLock(const char* fname);
    ~FileLock();
    FileLock(const FileLock&) = delete;
    FileLock& operator=(const FileLock&) = delete;
    void lock();
    void unlock();
    const char* filename() const;
};
