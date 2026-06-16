#ifndef __SRC_STD_DEQUE_RS
#define __SRC_STD_DEQUE_RS

#include <hicc/hicc.hpp>
#line 0 R"(./src/std_deque.rs)"
#line 6
#include <deque>
    #include <algorithm>
#line 11
template<class T, class Allocator> struct deque_11;
#line 11
namespace hicc { template<class T, class Allocator> struct MethodsType<std::deque<T, Allocator>, void> { typedef deque_11<T,Allocator> methods_type; }; }
#line 11
template<class T, class Allocator> struct deque_11 {
#line 11
typedef std::deque<T, Allocator> Self; typedef void SelfContainer; typedef deque_11<T,Allocator> SelfMethods;
#line 268
static void insert(Self& self, size_t pos, size_t count, const T& val) {
                self.insert(self.begin() + std::min(pos, self.size()), count, val);
            }
#line 284
static void erase(Self& self, size_t pos, size_t count) {
                pos = std::min(pos, self.size());
                count = std::min(count, self.size() - pos);
                self.erase(self.begin() + pos, self.begin() + pos + count);
            }
#line 18
static void _hicc_test_18() { bool (Self::* _18)() const = &Self::empty; (void)_18; }
#line 18
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((bool (Self::*)() const)&Self::empty));
#line 26
static void _hicc_test_26() { size_t (Self::* _26)() const = &Self::size; (void)_26; }
#line 26
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((size_t (Self::*)() const)&Self::size));
#line 34
static void _hicc_test_34() { size_t (Self::* _34)() const = &Self::max_size; (void)_34; }
#line 34
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((size_t (Self::*)() const)&Self::max_size));
#line 44
static void _hicc_test_44() { void (Self::* _44)() = &Self::clear; (void)_44; }
#line 44
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((void (Self::*)())&Self::clear));
#line 55
static void _hicc_test_55() { void (Self::* _55)() = &Self::shrink_to_fit; (void)_55; }
#line 55
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((void (Self::*)())&Self::shrink_to_fit));
#line 67
static void _hicc_test_67() { void (Self::* _67)(size_t, const T&) = &Self::resize; (void)_67; }
#line 67
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((void (Self::*)(size_t, const T&))&Self::resize));
#line 83
static void _hicc_test_83() { const T& (Self::* _83)(size_t) const = &Self::at; (void)_83; }
#line 83
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((const T& (Self::*)(size_t) const)&Self::at));
#line 100
static void _hicc_test_100() { T& (Self::* _100)(size_t) = &Self::at; (void)_100; }
#line 100
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((T& (Self::*)(size_t))&Self::at));
#line 116
static void _hicc_test_116() { const T& (Self::* _116)() const = &Self::back; (void)_116; }
#line 116
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((const T& (Self::*)() const)&Self::back));
#line 143
static void _hicc_test_143() { T& (Self::* _143)() = &Self::back; (void)_143; }
#line 143
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((T& (Self::*)())&Self::back));
#line 159
static void _hicc_test_159() { const T& (Self::* _159)() const = &Self::front; (void)_159; }
#line 159
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((const T& (Self::*)() const)&Self::front));
#line 186
static void _hicc_test_186() { T& (Self::* _186)() = &Self::front; (void)_186; }
#line 186
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((T& (Self::*)())&Self::front));
#line 195
static void _hicc_test_195() { void (Self::* _195)(size_t, const T&) = &Self::assign; (void)_195; }
#line 195
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((void (Self::*)(size_t, const T&))&Self::assign));
#line 212
static void _hicc_test_212() { void (Self::* _212)() = &Self::pop_back; (void)_212; }
#line 212
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((void (Self::*)())&Self::pop_back));
#line 222
static void _hicc_test_222() { void (Self::* _222)(const T&) = &Self::push_back; (void)_222; }
#line 222
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((void (Self::*)(const T&))&Self::push_back));
#line 239
static void _hicc_test_239() { void (Self::* _239)() = &Self::pop_front; (void)_239; }
#line 239
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((void (Self::*)())&Self::pop_front));
#line 248
static void _hicc_test_248() { void (Self::* _248)(const T&) = &Self::push_front; (void)_248; }
#line 248
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((void (Self::*)(const T&))&Self::push_front));
#line 258
static void _hicc_test_258() { void (Self::* _258)(Self&) = &Self::swap; (void)_258; }
#line 258
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((void (Self::*)(Self&))&Self::swap));
#line 272
static void _hicc_test_272() { void (* _272)(Self&, size_t, size_t, const T&) = &SelfMethods::insert; (void)_272; }
#line 272
EXPORT_METHOD_IN(SelfContainer, Self, ((void (*)(Self&, size_t, size_t, const T&))&SelfMethods::insert));
#line 290
static void _hicc_test_290() { void (* _290)(Self&, size_t, size_t) = &SelfMethods::erase; (void)_290; }
#line 290
EXPORT_METHOD_IN(SelfContainer, Self, ((void (*)(Self&, size_t, size_t))&SelfMethods::erase));
#line 11
};

#endif
