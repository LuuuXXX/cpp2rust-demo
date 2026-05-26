#include <hicc/hicc.hpp>
#line 0 R"(src/main.rs)"
#line 2
#include <iostream>
    #include <thread>
    #include <mutex>
    #include <fstream>
    #include <cstring>


    class Mutex {
        std::mutex mtx_;
        std::string name_;
    public:
        Mutex() : mtx_(), name_("unnamed") {}
        explicit Mutex(const char* name) : mtx_(), name_(name ? name : "unnamed") {}
        ~Mutex() {}
        void lock() { mtx_.lock(); }
        void unlock() { mtx_.unlock(); }
        bool try_lock() { return mtx_.try_lock(); }
        const char* name() const { return name_.c_str(); }
    };


    class ScopedLock {
        Mutex* mutex_;
        bool owns_lock_;
    public:
        explicit ScopedLock(Mutex* m) : mutex_(m), owns_lock_(false) {
            if (mutex_) {
                mutex_->lock();
                owns_lock_ = true;
            }
        }
        ~ScopedLock() {
            if (owns_lock_ && mutex_) {
                mutex_->unlock();
            }
        }
        ScopedLock(ScopedLock&& other) noexcept : mutex_(other.mutex_), owns_lock_(other.owns_lock_) {
            other.owns_lock_ = false;
        }
        ScopedLock(const ScopedLock&) = delete;
        ScopedLock& operator=(const ScopedLock&) = delete;
    };


    class FileLock {
        std::mutex mtx_;
        std::ofstream file_;
        std::string filename_;
    public:
        explicit FileLock(const char* fname) : mtx_(), file_(), filename_(fname ? fname : "") {
            if (!filename_.empty()) {
                file_.open(filename_, std::ios::app);
            }
        }
        ~FileLock() {
            if (file_.is_open()) {
                file_.close();
            }
        }
        FileLock(const FileLock&) = delete;
        FileLock& operator=(const FileLock&) = delete;
        void lock() { mtx_.lock(); }
        void unlock() { mtx_.unlock(); }
        const char* filename() const { return filename_.c_str(); }
    };


    Mutex* mutex_new() {
        return new Mutex();
    }

    void mutex_delete(Mutex* self_) {
        if (self_) {
            std::cout << "Mutex '" << self_->name() << "' deleted" << std::endl;
            delete self_;
        }
    }

    void mutex_lock(Mutex* self_) {
        self_->lock();
    }

    void mutex_unlock(Mutex* self_) {
        self_->unlock();
    }

    int mutex_try_lock(Mutex* self_) {
        return self_->try_lock() ? 1 : 0;
    }

    ScopedLock* scoped_lock_new(Mutex* mutex) {
        return new ScopedLock(mutex);
    }

    void scoped_lock_delete(ScopedLock* self_) {
        if (self_) {
            delete self_;
        }
    }

    FileLock* file_lock_new(const char* filename) {
        return new FileLock(filename);
    }

    void file_lock_delete(FileLock* self_) {
        if (self_) {
            delete self_;
        }
    }

    void file_lock_lock(FileLock* self_) {
        self_->lock();
    }

    void file_lock_unlock(FileLock* self_) {
        self_->unlock();
    }
#line 122
 struct Mutex_122;
#line 122
namespace hicc { template<> struct MethodsType<Mutex, void> { typedef Mutex_122 methods_type; }; }
#line 137
 struct ScopedLock_137;
#line 137
namespace hicc { template<> struct MethodsType<ScopedLock, void> { typedef ScopedLock_137 methods_type; }; }
#line 142
 struct FileLock_142;
#line 142
namespace hicc { template<> struct MethodsType<FileLock, void> { typedef FileLock_142 methods_type; }; }
#line 122
 struct Mutex_122 {
#line 122
typedef Mutex Self; typedef void SelfContainer; typedef Mutex_122 SelfMethods;
#line 124
static void _hicc_test_124() { void (Self::* _124)() = &Self::lock; (void)_124; }
#line 124
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((void (Self::*)())&Self::lock));
#line 127
static void _hicc_test_127() { void (Self::* _127)() = &Self::unlock; (void)_127; }
#line 127
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((void (Self::*)())&Self::unlock));
#line 130
static void _hicc_test_130() { bool (Self::* _130)() = &Self::try_lock; (void)_130; }
#line 130
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((bool (Self::*)())&Self::try_lock));
#line 133
static void _hicc_test_133() { const char* (Self::* _133)() const = &Self::name; (void)_133; }
#line 133
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((const char* (Self::*)() const)&Self::name));
#line 122
};
#line 137
 struct ScopedLock_137 {
#line 137
typedef ScopedLock Self; typedef void SelfContainer; typedef ScopedLock_137 SelfMethods;
#line 137
};
#line 142
 struct FileLock_142 {
#line 142
typedef FileLock Self; typedef void SelfContainer; typedef FileLock_142 SelfMethods;
#line 144
static void _hicc_test_144() { void (Self::* _144)() = &Self::lock; (void)_144; }
#line 144
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((void (Self::*)())&Self::lock));
#line 147
static void _hicc_test_147() { void (Self::* _147)() = &Self::unlock; (void)_147; }
#line 147
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((void (Self::*)())&Self::unlock));
#line 150
static void _hicc_test_150() { const char* (Self::* _150)() const = &Self::filename; (void)_150; }
#line 150
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((const char* (Self::*)() const)&Self::filename));
#line 142
};
#line 156
EXPORT_METHODS_BEG(raii_pattern) {
#line 159
static void _hicc_test_159() { Mutex* (* _159)() = &mutex_new; (void)_159; }
#line 159
EXPORT_METHOD_IN(void, ExportMethods, ((Mutex* (*)())&mutex_new));
#line 161
static void _hicc_test_161() { void (* _161)(Mutex* self_) = &mutex_delete; (void)_161; }
#line 161
EXPORT_METHOD_IN(void, ExportMethods, ((void (*)(Mutex* self_))&mutex_delete));
#line 163
static void _hicc_test_163() { void (* _163)(Mutex* self_) = &mutex_lock; (void)_163; }
#line 163
EXPORT_METHOD_IN(void, ExportMethods, ((void (*)(Mutex* self_))&mutex_lock));
#line 165
static void _hicc_test_165() { void (* _165)(Mutex* self_) = &mutex_unlock; (void)_165; }
#line 165
EXPORT_METHOD_IN(void, ExportMethods, ((void (*)(Mutex* self_))&mutex_unlock));
#line 167
static void _hicc_test_167() { int (* _167)(Mutex* self_) = &mutex_try_lock; (void)_167; }
#line 167
EXPORT_METHOD_IN(void, ExportMethods, ((int (*)(Mutex* self_))&mutex_try_lock));
#line 171
static void _hicc_test_171() { ScopedLock* (* _171)(Mutex* mutex) = &scoped_lock_new; (void)_171; }
#line 171
EXPORT_METHOD_IN(void, ExportMethods, ((ScopedLock* (*)(Mutex* mutex))&scoped_lock_new));
#line 173
static void _hicc_test_173() { void (* _173)(ScopedLock* self_) = &scoped_lock_delete; (void)_173; }
#line 173
EXPORT_METHOD_IN(void, ExportMethods, ((void (*)(ScopedLock* self_))&scoped_lock_delete));
#line 177
static void _hicc_test_177() { FileLock* (* _177)(const char* filename) = &file_lock_new; (void)_177; }
#line 177
EXPORT_METHOD_IN(void, ExportMethods, ((FileLock* (*)(const char* filename))&file_lock_new));
#line 179
static void _hicc_test_179() { void (* _179)(FileLock* self_) = &file_lock_delete; (void)_179; }
#line 179
EXPORT_METHOD_IN(void, ExportMethods, ((void (*)(FileLock* self_))&file_lock_delete));
#line 181
static void _hicc_test_181() { void (* _181)(FileLock* self_) = &file_lock_lock; (void)_181; }
#line 181
EXPORT_METHOD_IN(void, ExportMethods, ((void (*)(FileLock* self_))&file_lock_lock));
#line 183
static void _hicc_test_183() { void (* _183)(FileLock* self_) = &file_lock_unlock; (void)_183; }
#line 183
EXPORT_METHOD_IN(void, ExportMethods, ((void (*)(FileLock* self_))&file_lock_unlock));
#line 156
} EXPORT_METHODS_END();

