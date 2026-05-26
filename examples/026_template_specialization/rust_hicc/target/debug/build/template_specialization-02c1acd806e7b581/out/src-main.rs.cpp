#include <hicc/hicc.hpp>
#line 0 R"(src/main.rs)"
#line 2
#include <cstdio>
    #include <cstdlib>
    #include <cstring>


    class IntHolder {
        int value_;
    public:
        explicit IntHolder(int value) : value_(value) {}
        ~IntHolder() {}
        int get() const { return value_; }
        const char* describe() const {
            static char buf[64];
            snprintf(buf, sizeof(buf), "IntHolder(value=%d)", value_);
            return buf;
        }
    };

    IntHolder* intholder_new(int value) {
        return new IntHolder(value);
    }

    void intholder_delete(IntHolder* self_) {
        if (self_) delete self_;
    }


    class DoubleHolder {
        double value_;
    public:
        explicit DoubleHolder(double value) : value_(value) {}
        ~DoubleHolder() {}
        double get() const { return value_; }
        const char* describe() const {
            static char buf[64];
            snprintf(buf, sizeof(buf), "DoubleHolder(value=%.5f)", value_);
            return buf;
        }
    };

    DoubleHolder* doubleholder_new(double value) {
        return new DoubleHolder(value);
    }

    void doubleholder_delete(DoubleHolder* self_) {
        if (self_) delete self_;
    }


    class StringHolder {
        char* value_;
        int length_;
    public:
        explicit StringHolder(const char* value) {
            value_ = strdup(value);
            length_ = strlen(value);
        }
        ~StringHolder() {
            if (value_) free(value_);
        }
        const char* get() const { return value_; }
        const char* describe() const {
            static char buf[256];
            snprintf(buf, sizeof(buf), "StringHolder(value=\"%s\", length=%d)", value_, length_);
            return buf;
        }
    };

    StringHolder* stringholder_new(const char* value) {
        return new StringHolder(value);
    }

    void stringholder_delete(StringHolder* self_) {
        if (self_) delete self_;
    }
#line 80
 struct IntHolder_80;
#line 80
namespace hicc { template<> struct MethodsType<IntHolder, void> { typedef IntHolder_80 methods_type; }; }
#line 89
 struct DoubleHolder_89;
#line 89
namespace hicc { template<> struct MethodsType<DoubleHolder, void> { typedef DoubleHolder_89 methods_type; }; }
#line 98
 struct StringHolder_98;
#line 98
namespace hicc { template<> struct MethodsType<StringHolder, void> { typedef StringHolder_98 methods_type; }; }
#line 80
 struct IntHolder_80 {
#line 80
typedef IntHolder Self; typedef void SelfContainer; typedef IntHolder_80 SelfMethods;
#line 82
static void _hicc_test_82() { int (Self::* _82)() const = &Self::get; (void)_82; }
#line 82
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((int (Self::*)() const)&Self::get));
#line 85
static void _hicc_test_85() { const char* (Self::* _85)() const = &Self::describe; (void)_85; }
#line 85
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((const char* (Self::*)() const)&Self::describe));
#line 80
};
#line 89
 struct DoubleHolder_89 {
#line 89
typedef DoubleHolder Self; typedef void SelfContainer; typedef DoubleHolder_89 SelfMethods;
#line 91
static void _hicc_test_91() { double (Self::* _91)() const = &Self::get; (void)_91; }
#line 91
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((double (Self::*)() const)&Self::get));
#line 94
static void _hicc_test_94() { const char* (Self::* _94)() const = &Self::describe; (void)_94; }
#line 94
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((const char* (Self::*)() const)&Self::describe));
#line 89
};
#line 98
 struct StringHolder_98 {
#line 98
typedef StringHolder Self; typedef void SelfContainer; typedef StringHolder_98 SelfMethods;
#line 100
static void _hicc_test_100() { const char* (Self::* _100)() const = &Self::get; (void)_100; }
#line 100
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((const char* (Self::*)() const)&Self::get));
#line 103
static void _hicc_test_103() { const char* (Self::* _103)() const = &Self::describe; (void)_103; }
#line 103
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((const char* (Self::*)() const)&Self::describe));
#line 98
};
#line 109
EXPORT_METHODS_BEG(template_specialization) {
#line 112
static void _hicc_test_112() { IntHolder* (* _112)(int value) = &intholder_new; (void)_112; }
#line 112
EXPORT_METHOD_IN(void, ExportMethods, ((IntHolder* (*)(int value))&intholder_new));
#line 114
static void _hicc_test_114() { void (* _114)(IntHolder* self_) = &intholder_delete; (void)_114; }
#line 114
EXPORT_METHOD_IN(void, ExportMethods, ((void (*)(IntHolder* self_))&intholder_delete));
#line 118
static void _hicc_test_118() { DoubleHolder* (* _118)(double value) = &doubleholder_new; (void)_118; }
#line 118
EXPORT_METHOD_IN(void, ExportMethods, ((DoubleHolder* (*)(double value))&doubleholder_new));
#line 120
static void _hicc_test_120() { void (* _120)(DoubleHolder* self_) = &doubleholder_delete; (void)_120; }
#line 120
EXPORT_METHOD_IN(void, ExportMethods, ((void (*)(DoubleHolder* self_))&doubleholder_delete));
#line 124
static void _hicc_test_124() { StringHolder* (* _124)(const char* value) = &stringholder_new; (void)_124; }
#line 124
EXPORT_METHOD_IN(void, ExportMethods, ((StringHolder* (*)(const char* value))&stringholder_new));
#line 126
static void _hicc_test_126() { void (* _126)(StringHolder* self_) = &stringholder_delete; (void)_126; }
#line 126
EXPORT_METHOD_IN(void, ExportMethods, ((void (*)(StringHolder* self_))&stringholder_delete));
#line 109
} EXPORT_METHODS_END();

