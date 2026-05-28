#pragma once

#ifdef __cplusplus
extern "C" {
#endif

// RAII 模式示例
// 展示如何使用 FFI 模拟 RAII 自动资源管理

#include <stddef.h>

// 互斥锁结构
class Mutex;

// 创建互斥锁
Mutex* mutex_new(void);

// 销毁互斥锁
void mutex_delete(Mutex* self);

// 加锁
void mutex_lock(Mutex* self);

// 解锁
void mutex_unlock(Mutex* self);

// 尝试加锁
int mutex_try_lock(Mutex* self);

// 作用域锁/守卫
class ScopedLock;

// 创建作用域锁（构造时自动加锁）
ScopedLock* scoped_lock_new(Mutex* mutex);

// 销毁作用域锁（析构时自动解锁）
void scoped_lock_delete(ScopedLock* self);

// 文件锁结构
class FileLock;

// 创建文件锁
FileLock* file_lock_new(const char* filename);

// 销毁文件锁
void file_lock_delete(FileLock* self);

// 加锁
void file_lock_lock(FileLock* self);

// 解锁
void file_lock_unlock(FileLock* self);

#ifdef __cplusplus
}

// Full class definition - for hicc code generation
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

#endif
