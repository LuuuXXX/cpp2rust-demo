#include <hicc/hicc.hpp>
#line 0 R"(src/main.rs)"
#line 2
#include <array>

    template<typename T, unsigned long N>
    class ArrayImpl {
    public:
        std::array<T, N> data;
        ArrayImpl() : data() {}
        explicit ArrayImpl(const T* values) {
            if (values) {
                for (unsigned long i = 0; i < N; ++i) {
                    data[i] = values[i];
                }
            }
        }
    };

    class IntArray5 {
    public:
        ArrayImpl<int, 5>* impl;
        IntArray5() : impl(new ArrayImpl<int, 5>()) {}
        explicit IntArray5(const int* values) : impl(new ArrayImpl<int, 5>(values)) {}
        ~IntArray5() { delete impl; }
        unsigned long size() const { return 5; }
        bool empty() const { return false; }
        int get(unsigned long index) const { return index < 5 ? impl->data[index] : 0; }
        void set(unsigned long index, int value) { if (index < 5) impl->data[index] = value; }
        int* data() { return impl->data.data(); }
        int at(unsigned long index) const { return impl->data.at(index); }
    };

    IntArray5* int_array5_new() { return new IntArray5(); }
    IntArray5* int_array5_new_from(const int* values) { return new IntArray5(values); }
    void int_array5_delete(IntArray5* self) { delete self; }
#line 38
 struct IntArray5_38;
#line 38
namespace hicc { template<> struct MethodsType<IntArray5, void> { typedef IntArray5_38 methods_type; }; }
#line 38
 struct IntArray5_38 {
#line 38
typedef IntArray5 Self; typedef void SelfContainer; typedef IntArray5_38 SelfMethods;
#line 40
static void _hicc_test_40() { unsigned long (Self::* _40)() const = &Self::size; (void)_40; }
#line 40
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((unsigned long (Self::*)() const)&Self::size));
#line 43
static void _hicc_test_43() { bool (Self::* _43)() const = &Self::empty; (void)_43; }
#line 43
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((bool (Self::*)() const)&Self::empty));
#line 46
static void _hicc_test_46() { int (Self::* _46)(unsigned long) const = &Self::get; (void)_46; }
#line 46
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((int (Self::*)(unsigned long) const)&Self::get));
#line 49
static void _hicc_test_49() { void (Self::* _49)(unsigned long, int) = &Self::set; (void)_49; }
#line 49
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((void (Self::*)(unsigned long, int))&Self::set));
#line 52
static void _hicc_test_52() { int* (Self::* _52)() = &Self::data; (void)_52; }
#line 52
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((int* (Self::*)())&Self::data));
#line 55
static void _hicc_test_55() { int (Self::* _55)(unsigned long) const = &Self::at; (void)_55; }
#line 55
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((int (Self::*)(unsigned long) const)&Self::at));
#line 38
};
#line 61
EXPORT_METHODS_BEG(array_basic) {
#line 65
static void _hicc_test_65() { IntArray5* (* _65)() = &int_array5_new; (void)_65; }
#line 65
EXPORT_METHOD_IN(void, ExportMethods, ((IntArray5* (*)())&int_array5_new));
#line 68
static void _hicc_test_68() { IntArray5* (* _68)(const int* values) = &int_array5_new_from; (void)_68; }
#line 68
EXPORT_METHOD_IN(void, ExportMethods, ((IntArray5* (*)(const int* values))&int_array5_new_from));
#line 71
static void _hicc_test_71() { void (* _71)(IntArray5* self) = &int_array5_delete; (void)_71; }
#line 71
EXPORT_METHOD_IN(void, ExportMethods, ((void (*)(IntArray5* self))&int_array5_delete));
#line 61
} EXPORT_METHODS_END();

