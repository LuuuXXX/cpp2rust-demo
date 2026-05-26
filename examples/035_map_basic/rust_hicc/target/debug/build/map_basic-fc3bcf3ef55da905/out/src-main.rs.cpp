#include <hicc/hicc.hpp>
#line 0 R"(src/main.rs)"
#line 2
#include <map>
    #include <string>

    template<typename K, typename V>
    class MapImpl {
    public:
        std::map<K, V> data;
        MapImpl() = default;
        ~MapImpl() { data.clear(); }
    };

    class StringIntMap {
    public:
        MapImpl<std::string, int>* impl;
        StringIntMap() : impl(new MapImpl<std::string, int>()) {}
        ~StringIntMap() { delete impl; }
        unsigned long size() const { return impl->data.size(); }
        bool empty() const { return impl->data.empty(); }
        bool insert(const char* key, int value) {
            if (!key) return false;
            auto result = impl->data.insert({std::string(key), value});
            return result.second;
        }
        bool erase(const char* key) {
            if (!key) return false;
            return impl->data.erase(std::string(key)) > 0;
        }
        void clear() { impl->data.clear(); }
        int get(const char* key) const {
            if (!key) return 0;
            auto it = impl->data.find(std::string(key));
            if (it != impl->data.end()) return it->second;
            return 0;
        }
        void set(const char* key, int value) {
            if (key) impl->data[std::string(key)] = value;
        }
    };

    StringIntMap* string_int_map_new() { return new StringIntMap(); }
    void string_int_map_delete(StringIntMap* self) { delete self; }
#line 46
 struct StringIntMap_46;
#line 46
namespace hicc { template<> struct MethodsType<StringIntMap, void> { typedef StringIntMap_46 methods_type; }; }
#line 46
 struct StringIntMap_46 {
#line 46
typedef StringIntMap Self; typedef void SelfContainer; typedef StringIntMap_46 SelfMethods;
#line 48
static void _hicc_test_48() { unsigned long (Self::* _48)() const = &Self::size; (void)_48; }
#line 48
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((unsigned long (Self::*)() const)&Self::size));
#line 51
static void _hicc_test_51() { bool (Self::* _51)() const = &Self::empty; (void)_51; }
#line 51
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((bool (Self::*)() const)&Self::empty));
#line 54
static void _hicc_test_54() { bool (Self::* _54)(const char*, int) = &Self::insert; (void)_54; }
#line 54
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((bool (Self::*)(const char*, int))&Self::insert));
#line 57
static void _hicc_test_57() { bool (Self::* _57)(const char*) = &Self::erase; (void)_57; }
#line 57
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((bool (Self::*)(const char*))&Self::erase));
#line 60
static void _hicc_test_60() { void (Self::* _60)() = &Self::clear; (void)_60; }
#line 60
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((void (Self::*)())&Self::clear));
#line 63
static void _hicc_test_63() { int (Self::* _63)(const char*) const = &Self::get; (void)_63; }
#line 63
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((int (Self::*)(const char*) const)&Self::get));
#line 66
static void _hicc_test_66() { void (Self::* _66)(const char*, int) = &Self::set; (void)_66; }
#line 66
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((void (Self::*)(const char*, int))&Self::set));
#line 46
};
#line 72
EXPORT_METHODS_BEG(map_basic) {
#line 76
static void _hicc_test_76() { StringIntMap* (* _76)() = &string_int_map_new; (void)_76; }
#line 76
EXPORT_METHOD_IN(void, ExportMethods, ((StringIntMap* (*)())&string_int_map_new));
#line 79
static void _hicc_test_79() { void (* _79)(StringIntMap* self) = &string_int_map_delete; (void)_79; }
#line 79
EXPORT_METHOD_IN(void, ExportMethods, ((void (*)(StringIntMap* self))&string_int_map_delete));
#line 72
} EXPORT_METHODS_END();

