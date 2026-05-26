#include <hicc/hicc.hpp>
#line 0 R"(src/main.rs)"
#line 2
#include <iostream>
    #include <memory>
    #include <unordered_map>


    class SharedData {
        std::string name_;
    public:
        int value;
        SharedData(const char* n) : name_(n ? n : ""), value(0) {}
        ~SharedData() {}
        int useCount() const { return 1; }
        const char* getName() const { return name_.c_str(); }
        SharedData* clone() const { return new SharedData(name_.c_str()); }
        void reset() { name_.clear(); }
        bool expired() const { return name_.empty(); }
    };


    class Cache {
        std::unordered_map<std::string, void*> data_;
    public:
        Cache() : data_() {}
        ~Cache() {}
        SharedData* get(const char* name) {
            if (!name) return nullptr;
            std::string key(name);
            auto it = data_.find(key);
            if (it != data_.end()) {
                return reinterpret_cast<SharedData*>(it->second);
            }
            SharedData* new_data = new SharedData(name);
            data_[key] = reinterpret_cast<void*>(new_data);
            return new_data;
        }
    };


    SharedData* shareddata_new(const char* name) {
        return new SharedData(name);
    }

    void shareddata_delete(SharedData* self_) {
        delete self_;
    }

    SharedData* shareddata_clone(SharedData* self_) {
        return self_ ? self_->clone() : nullptr;
    }

    void shareddata_reset(SharedData* self_) {
        if (self_) self_->reset();
    }

    Cache* cache_new() {
        return new Cache();
    }

    void cache_delete(Cache* self_) {
        delete self_;
    }
#line 66
 struct SharedData_66;
#line 66
namespace hicc { template<> struct MethodsType<SharedData, void> { typedef SharedData_66 methods_type; }; }
#line 84
 struct Cache_84;
#line 84
namespace hicc { template<> struct MethodsType<Cache, void> { typedef Cache_84 methods_type; }; }
#line 66
 struct SharedData_66 {
#line 66
typedef SharedData Self; typedef void SelfContainer; typedef SharedData_66 SelfMethods;
#line 68
static void _hicc_test_68() { int (Self::* _68)() const = &Self::useCount; (void)_68; }
#line 68
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((int (Self::*)() const)&Self::useCount));
#line 71
static void _hicc_test_71() { const char* (Self::* _71)() const = &Self::getName; (void)_71; }
#line 71
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((const char* (Self::*)() const)&Self::getName));
#line 74
static void _hicc_test_74() { SharedData* (Self::* _74)() const = &Self::clone; (void)_74; }
#line 74
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((SharedData* (Self::*)() const)&Self::clone));
#line 77
static void _hicc_test_77() { void (Self::* _77)() = &Self::reset; (void)_77; }
#line 77
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((void (Self::*)())&Self::reset));
#line 80
static void _hicc_test_80() { bool (Self::* _80)() const = &Self::expired; (void)_80; }
#line 80
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((bool (Self::*)() const)&Self::expired));
#line 66
};
#line 84
 struct Cache_84 {
#line 84
typedef Cache Self; typedef void SelfContainer; typedef Cache_84 SelfMethods;
#line 86
static void _hicc_test_86() { SharedData* (Self::* _86)(const char* name) = &Self::get; (void)_86; }
#line 86
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((SharedData* (Self::*)(const char* name))&Self::get));
#line 84
};
#line 92
EXPORT_METHODS_BEG(shared_ptr) {
#line 95
static void _hicc_test_95() { SharedData* (* _95)(const char* name) = &shareddata_new; (void)_95; }
#line 95
EXPORT_METHOD_IN(void, ExportMethods, ((SharedData* (*)(const char* name))&shareddata_new));
#line 97
static void _hicc_test_97() { void (* _97)(SharedData* self_) = &shareddata_delete; (void)_97; }
#line 97
EXPORT_METHOD_IN(void, ExportMethods, ((void (*)(SharedData* self_))&shareddata_delete));
#line 99
static void _hicc_test_99() { SharedData* (* _99)(SharedData* self_) = &shareddata_clone; (void)_99; }
#line 99
EXPORT_METHOD_IN(void, ExportMethods, ((SharedData* (*)(SharedData* self_))&shareddata_clone));
#line 101
static void _hicc_test_101() { void (* _101)(SharedData* self_) = &shareddata_reset; (void)_101; }
#line 101
EXPORT_METHOD_IN(void, ExportMethods, ((void (*)(SharedData* self_))&shareddata_reset));
#line 105
static void _hicc_test_105() { Cache* (* _105)() = &cache_new; (void)_105; }
#line 105
EXPORT_METHOD_IN(void, ExportMethods, ((Cache* (*)())&cache_new));
#line 107
static void _hicc_test_107() { void (* _107)(Cache* self_) = &cache_delete; (void)_107; }
#line 107
EXPORT_METHOD_IN(void, ExportMethods, ((void (*)(Cache* self_))&cache_delete));
#line 92
} EXPORT_METHODS_END();

