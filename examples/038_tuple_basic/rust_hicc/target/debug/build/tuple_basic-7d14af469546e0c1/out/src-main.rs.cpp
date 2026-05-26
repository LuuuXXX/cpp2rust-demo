#include <hicc/hicc.hpp>
#line 0 R"(src/main.rs)"
#line 2
#include <tuple>
    #include <string>

    class Tuple2 {
    public:
        std::tuple<int, std::string> data;
        Tuple2(int first, const char* second) : data(first, second ? second : "") {}
        int get_first() const { return std::get<0>(data); }
        const char* get_second() const {
            static thread_local std::string temp;
            temp = std::get<1>(data);
            return temp.c_str();
        }
    };

    class Tuple3 {
    public:
        std::tuple<int, double, std::string> data;
        Tuple3(int first, double second, const char* third) : data(first, second, third ? third : "") {}
        int get_first() const { return std::get<0>(data); }
        double get_second() const { return std::get<1>(data); }
        const char* get_third() const {
            static thread_local std::string temp;
            temp = std::get<2>(data);
            return temp.c_str();
        }
    };

    class Tuple4 {
    public:
        std::tuple<int, double, std::string, int> data;
        Tuple4(int first, double second, const char* third, int fourth) : data(first, second, third ? third : "", fourth) {}
        int get_first() const { return std::get<0>(data); }
        double get_second() const { return std::get<1>(data); }
        const char* get_third() const {
            static thread_local std::string temp;
            temp = std::get<2>(data);
            return temp.c_str();
        }
        int get_fourth() const { return std::get<3>(data); }
    };

    Tuple2* tuple2_new(int first, const char* second) { return new Tuple2(first, second); }
    void tuple2_delete(Tuple2* self) { delete self; }

    Tuple3* tuple3_new(int first, double second, const char* third) { return new Tuple3(first, second, third); }
    void tuple3_delete(Tuple3* self) { delete self; }

    Tuple4* tuple4_new(int first, double second, const char* third, int fourth) { return new Tuple4(first, second, third, fourth); }
    void tuple4_delete(Tuple4* self) { delete self; }

    Tuple2* make_int_string_pair(int i, const char* s) { return new Tuple2(i, s); }
    Tuple3* make_int_double_string(int i, double d, const char* s) { return new Tuple3(i, d, s); }
#line 58
 struct Tuple2_58;
#line 58
namespace hicc { template<> struct MethodsType<Tuple2, void> { typedef Tuple2_58 methods_type; }; }
#line 58
 struct Tuple2_58 {
#line 58
typedef Tuple2 Self; typedef void SelfContainer; typedef Tuple2_58 SelfMethods;
#line 60
static void _hicc_test_60() { int (Self::* _60)() const = &Self::get_first; (void)_60; }
#line 60
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((int (Self::*)() const)&Self::get_first));
#line 63
static void _hicc_test_63() { const char* (Self::* _63)() const = &Self::get_second; (void)_63; }
#line 63
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((const char* (Self::*)() const)&Self::get_second));
#line 58
};
#line 69
 struct Tuple3_69;
#line 69
namespace hicc { template<> struct MethodsType<Tuple3, void> { typedef Tuple3_69 methods_type; }; }
#line 69
 struct Tuple3_69 {
#line 69
typedef Tuple3 Self; typedef void SelfContainer; typedef Tuple3_69 SelfMethods;
#line 71
static void _hicc_test_71() { int (Self::* _71)() const = &Self::get_first; (void)_71; }
#line 71
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((int (Self::*)() const)&Self::get_first));
#line 74
static void _hicc_test_74() { double (Self::* _74)() const = &Self::get_second; (void)_74; }
#line 74
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((double (Self::*)() const)&Self::get_second));
#line 77
static void _hicc_test_77() { const char* (Self::* _77)() const = &Self::get_third; (void)_77; }
#line 77
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((const char* (Self::*)() const)&Self::get_third));
#line 69
};
#line 83
 struct Tuple4_83;
#line 83
namespace hicc { template<> struct MethodsType<Tuple4, void> { typedef Tuple4_83 methods_type; }; }
#line 83
 struct Tuple4_83 {
#line 83
typedef Tuple4 Self; typedef void SelfContainer; typedef Tuple4_83 SelfMethods;
#line 85
static void _hicc_test_85() { int (Self::* _85)() const = &Self::get_first; (void)_85; }
#line 85
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((int (Self::*)() const)&Self::get_first));
#line 88
static void _hicc_test_88() { double (Self::* _88)() const = &Self::get_second; (void)_88; }
#line 88
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((double (Self::*)() const)&Self::get_second));
#line 91
static void _hicc_test_91() { const char* (Self::* _91)() const = &Self::get_third; (void)_91; }
#line 91
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((const char* (Self::*)() const)&Self::get_third));
#line 94
static void _hicc_test_94() { int (Self::* _94)() const = &Self::get_fourth; (void)_94; }
#line 94
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((int (Self::*)() const)&Self::get_fourth));
#line 83
};
#line 100
EXPORT_METHODS_BEG(tuple_basic) {
#line 106
static void _hicc_test_106() { Tuple2* (* _106)(int, const char*) = &tuple2_new; (void)_106; }
#line 106
EXPORT_METHOD_IN(void, ExportMethods, ((Tuple2* (*)(int, const char*))&tuple2_new));
#line 109
static void _hicc_test_109() { void (* _109)(Tuple2* self) = &tuple2_delete; (void)_109; }
#line 109
EXPORT_METHOD_IN(void, ExportMethods, ((void (*)(Tuple2* self))&tuple2_delete));
#line 112
static void _hicc_test_112() { Tuple3* (* _112)(int, double, const char*) = &tuple3_new; (void)_112; }
#line 112
EXPORT_METHOD_IN(void, ExportMethods, ((Tuple3* (*)(int, double, const char*))&tuple3_new));
#line 115
static void _hicc_test_115() { void (* _115)(Tuple3* self) = &tuple3_delete; (void)_115; }
#line 115
EXPORT_METHOD_IN(void, ExportMethods, ((void (*)(Tuple3* self))&tuple3_delete));
#line 118
static void _hicc_test_118() { Tuple4* (* _118)(int, double, const char*, int) = &tuple4_new; (void)_118; }
#line 118
EXPORT_METHOD_IN(void, ExportMethods, ((Tuple4* (*)(int, double, const char*, int))&tuple4_new));
#line 121
static void _hicc_test_121() { void (* _121)(Tuple4* self) = &tuple4_delete; (void)_121; }
#line 121
EXPORT_METHOD_IN(void, ExportMethods, ((void (*)(Tuple4* self))&tuple4_delete));
#line 124
static void _hicc_test_124() { Tuple2* (* _124)(int, const char*) = &make_int_string_pair; (void)_124; }
#line 124
EXPORT_METHOD_IN(void, ExportMethods, ((Tuple2* (*)(int, const char*))&make_int_string_pair));
#line 127
static void _hicc_test_127() { Tuple3* (* _127)(int, double, const char*) = &make_int_double_string; (void)_127; }
#line 127
EXPORT_METHOD_IN(void, ExportMethods, ((Tuple3* (*)(int, double, const char*))&make_int_double_string));
#line 100
} EXPORT_METHODS_END();

