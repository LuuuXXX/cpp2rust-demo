#ifndef __SRC_STD_ARRAY_RS
#define __SRC_STD_ARRAY_RS

#include <hicc/hicc.hpp>
#line 0 R"(./src/std_array.rs)"
#line 5
#include <array>
#line 9
template<class T, size_t N> struct array_9;
#line 9
namespace hicc { template<class T, size_t N> struct MethodsType<std::array<T, N>, void> { typedef array_9<T,N> methods_type; }; }
#line 9
template<class T, size_t N> struct array_9 {
#line 9
typedef std::array<T, N> Self; typedef void SelfContainer; typedef array_9<T,N> SelfMethods;
#line 11
static void _hicc_test_11() { const T* (Self::* _11)() const = &Self::data; (void)_11; }
#line 11
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((const T* (Self::*)() const)&Self::data));
#line 18
static void _hicc_test_18() { size_t (Self::* _18)() const = &Self::size; (void)_18; }
#line 18
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((size_t (Self::*)() const)&Self::size));
#line 42
static void _hicc_test_42() { const T& (Self::* _42)(size_t) const = &Self::at; (void)_42; }
#line 42
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((const T& (Self::*)(size_t) const)&Self::at));
#line 57
static void _hicc_test_57() { T& (Self::* _57)(size_t) = &Self::at; (void)_57; }
#line 57
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((T& (Self::*)(size_t))&Self::at));
#line 72
static void _hicc_test_72() { const T& (Self::* _72)() const = &Self::back; (void)_72; }
#line 72
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((const T& (Self::*)() const)&Self::back));
#line 88
static void _hicc_test_88() { T& (Self::* _88)() = &Self::back; (void)_88; }
#line 88
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((T& (Self::*)())&Self::back));
#line 102
static void _hicc_test_102() { const T& (Self::* _102)() const = &Self::front; (void)_102; }
#line 102
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((const T& (Self::*)() const)&Self::front));
#line 118
static void _hicc_test_118() { T& (Self::* _118)() = &Self::front; (void)_118; }
#line 118
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((T& (Self::*)())&Self::front));
#line 127
static void _hicc_test_127() { void (Self::* _127)(const T&) = &Self::fill; (void)_127; }
#line 127
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((void (Self::*)(const T&))&Self::fill));
#line 9
};

#endif
