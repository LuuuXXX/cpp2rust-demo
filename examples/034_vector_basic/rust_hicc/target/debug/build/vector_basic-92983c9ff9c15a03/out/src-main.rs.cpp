#include <hicc/hicc.hpp>
#line 0 R"(src/main.rs)"
#line 2
#include <vector>

    template<typename T>
    class VectorImpl {
    public:
        std::vector<T> data;
        VectorImpl() = default;
        ~VectorImpl() { data.clear(); }
    };

    class IntVector {
    public:
        VectorImpl<int>* impl;
        IntVector() : impl(new VectorImpl<int>()) {}
        ~IntVector() { delete impl; }
        unsigned long size() const { return impl->data.size(); }
        unsigned long capacity() const { return impl->data.capacity(); }
        bool empty() const { return impl->data.empty(); }
        void push_back(int value) { impl->data.push_back(value); }
        int get(unsigned long index) const { return index < impl->data.size() ? impl->data[index] : 0; }
        void set(unsigned long index, int value) { if (index < impl->data.size()) impl->data[index] = value; }
        void clear() { impl->data.clear(); }
        int* data() { return impl->data.data(); }
    };

    IntVector* int_vector_new() { return new IntVector(); }
    void int_vector_delete(IntVector* self) { delete self; }
#line 32
 struct IntVector_32;
#line 32
namespace hicc { template<> struct MethodsType<IntVector, void> { typedef IntVector_32 methods_type; }; }
#line 32
 struct IntVector_32 {
#line 32
typedef IntVector Self; typedef void SelfContainer; typedef IntVector_32 SelfMethods;
#line 34
static void _hicc_test_34() { unsigned long (Self::* _34)() const = &Self::size; (void)_34; }
#line 34
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((unsigned long (Self::*)() const)&Self::size));
#line 37
static void _hicc_test_37() { unsigned long (Self::* _37)() const = &Self::capacity; (void)_37; }
#line 37
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((unsigned long (Self::*)() const)&Self::capacity));
#line 40
static void _hicc_test_40() { bool (Self::* _40)() const = &Self::empty; (void)_40; }
#line 40
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((bool (Self::*)() const)&Self::empty));
#line 43
static void _hicc_test_43() { void (Self::* _43)(int) = &Self::push_back; (void)_43; }
#line 43
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((void (Self::*)(int))&Self::push_back));
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
static void _hicc_test_55() { void (Self::* _55)() = &Self::clear; (void)_55; }
#line 55
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((void (Self::*)())&Self::clear));
#line 32
};
#line 61
EXPORT_METHODS_BEG(vector_basic) {
#line 65
static void _hicc_test_65() { IntVector* (* _65)() = &int_vector_new; (void)_65; }
#line 65
EXPORT_METHOD_IN(void, ExportMethods, ((IntVector* (*)())&int_vector_new));
#line 68
static void _hicc_test_68() { void (* _68)(IntVector* self) = &int_vector_delete; (void)_68; }
#line 68
EXPORT_METHOD_IN(void, ExportMethods, ((void (*)(IntVector* self))&int_vector_delete));
#line 61
} EXPORT_METHODS_END();

