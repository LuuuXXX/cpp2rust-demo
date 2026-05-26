#include <hicc/hicc.hpp>
#line 0 R"(src/main.rs)"
#line 2
#include <iostream>
    #include <cstring>

    class DataFetcher {
        const char* name;
        mutable int cache_count;
        char cache_data[256];
    public:
        DataFetcher(const char* n);
        ~DataFetcher();
        const char* getName() const;
        int getCacheCount() const;
        void refresh();
    };

    DataFetcher* datafetcher_new(const char* name) {
        return new DataFetcher(name);
    }

    void datafetcher_delete(DataFetcher* self) {
        delete self;
    }

    const char* datafetcher_getName(DataFetcher* self) {
        return self->getName();
    }

    int datafetcher_getCacheCount(DataFetcher* self) {
        return self->getCacheCount();
    }

    void datafetcher_refresh(DataFetcher* self) {
        self->refresh();
    }

    DataFetcher::DataFetcher(const char* n) : cache_count(0) {
        name = n;
    }
    DataFetcher::~DataFetcher() {}
    const char* DataFetcher::getName() const { return name; }
    int DataFetcher::getCacheCount() const { return cache_count; }
    void DataFetcher::refresh() { cache_count++; }
#line 47
 struct DataFetcher_47;
#line 47
namespace hicc { template<> struct MethodsType<DataFetcher, void> { typedef DataFetcher_47 methods_type; }; }
#line 47
 struct DataFetcher_47 {
#line 47
typedef DataFetcher Self; typedef void SelfContainer; typedef DataFetcher_47 SelfMethods;
#line 49
static void _hicc_test_49() { const char* (Self::* _49)() const = &Self::getName; (void)_49; }
#line 49
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((const char* (Self::*)() const)&Self::getName));
#line 52
static void _hicc_test_52() { int (Self::* _52)() const = &Self::getCacheCount; (void)_52; }
#line 52
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((int (Self::*)() const)&Self::getCacheCount));
#line 55
static void _hicc_test_55() { void (Self::* _55)() = &Self::refresh; (void)_55; }
#line 55
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((void (Self::*)())&Self::refresh));
#line 47
};
#line 61
EXPORT_METHODS_BEG(mutable_member) {
#line 65
static void _hicc_test_65() { DataFetcher* (* _65)(const char* name) = &datafetcher_new; (void)_65; }
#line 65
EXPORT_METHOD_IN(void, ExportMethods, ((DataFetcher* (*)(const char* name))&datafetcher_new));
#line 68
static void _hicc_test_68() { void (* _68)(DataFetcher* self) = &datafetcher_delete; (void)_68; }
#line 68
EXPORT_METHOD_IN(void, ExportMethods, ((void (*)(DataFetcher* self))&datafetcher_delete));
#line 71
static void _hicc_test_71() { const char* (* _71)(DataFetcher* self) = &datafetcher_getName; (void)_71; }
#line 71
EXPORT_METHOD_IN(void, ExportMethods, ((const char* (*)(DataFetcher* self))&datafetcher_getName));
#line 74
static void _hicc_test_74() { int (* _74)(DataFetcher* self) = &datafetcher_getCacheCount; (void)_74; }
#line 74
EXPORT_METHOD_IN(void, ExportMethods, ((int (*)(DataFetcher* self))&datafetcher_getCacheCount));
#line 77
static void _hicc_test_77() { void (* _77)(DataFetcher* self) = &datafetcher_refresh; (void)_77; }
#line 77
EXPORT_METHOD_IN(void, ExportMethods, ((void (*)(DataFetcher* self))&datafetcher_refresh));
#line 61
} EXPORT_METHODS_END();

