#ifndef __SRC_STD_VECTOR_RS
#define __SRC_STD_VECTOR_RS

#include <hicc/hicc.hpp>
#line 0 R"(./src/std_vector.rs)"
#line 5
#include <vector>
    #include <algorithm>
#line 10
template<class T, class Allocator> struct vector_10;
#line 10
namespace hicc { template<class T, class Allocator> struct MethodsType<std::vector<T, Allocator>, void> { typedef vector_10<T,Allocator> methods_type; }; }
#line 10
template<class T, class Allocator> struct vector_10 {
#line 10
typedef std::vector<T, Allocator> Self; typedef void SelfContainer; typedef vector_10<T,Allocator> SelfMethods;
#line 254
static void insert(Self& self, size_t pos, size_t count, const T& val) {
                self.insert(self.begin() + std::min(pos, self.size()), count, val);
            }
#line 270
static void erase(Self& self, size_t pos, size_t count) {
                pos = std::min(pos, self.size());
                count = std::min(count, self.size() - pos);
                self.erase(self.begin() + pos, self.begin() + pos + count);
            }
#line 12
static void _hicc_test_12() { const T* (Self::* _12)() const = &Self::data; (void)_12; }
#line 12
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((const T* (Self::*)() const)&Self::data));
#line 19
static void _hicc_test_19() { bool (Self::* _19)() const = &Self::empty; (void)_19; }
#line 19
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((bool (Self::*)() const)&Self::empty));
#line 27
static void _hicc_test_27() { size_t (Self::* _27)() const = &Self::size; (void)_27; }
#line 27
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((size_t (Self::*)() const)&Self::size));
#line 35
static void _hicc_test_35() { size_t (Self::* _35)() const = &Self::max_size; (void)_35; }
#line 35
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((size_t (Self::*)() const)&Self::max_size));
#line 43
static void _hicc_test_43() { size_t (Self::* _43)() const = &Self::capacity; (void)_43; }
#line 43
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((size_t (Self::*)() const)&Self::capacity));
#line 53
static void _hicc_test_53() { void (Self::* _53)() = &Self::clear; (void)_53; }
#line 53
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((void (Self::*)())&Self::clear));
#line 63
static void _hicc_test_63() { void (Self::* _63)(size_t) = &Self::reserve; (void)_63; }
#line 63
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((void (Self::*)(size_t))&Self::reserve));
#line 73
static void _hicc_test_73() { void (Self::* _73)() = &Self::shrink_to_fit; (void)_73; }
#line 73
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((void (Self::*)())&Self::shrink_to_fit));
#line 84
static void _hicc_test_84() { void (Self::* _84)(size_t, const T&) = &Self::resize; (void)_84; }
#line 84
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((void (Self::*)(size_t, const T&))&Self::resize));
#line 100
static void _hicc_test_100() { const T& (Self::* _100)(size_t) const = &Self::at; (void)_100; }
#line 100
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((const T& (Self::*)(size_t) const)&Self::at));
#line 117
static void _hicc_test_117() { T& (Self::* _117)(size_t) = &Self::at; (void)_117; }
#line 117
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((T& (Self::*)(size_t))&Self::at));
#line 133
static void _hicc_test_133() { const T& (Self::* _133)() const = &Self::back; (void)_133; }
#line 133
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((const T& (Self::*)() const)&Self::back));
#line 160
static void _hicc_test_160() { T& (Self::* _160)() = &Self::back; (void)_160; }
#line 160
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((T& (Self::*)())&Self::back));
#line 176
static void _hicc_test_176() { const T& (Self::* _176)() const = &Self::front; (void)_176; }
#line 176
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((const T& (Self::*)() const)&Self::front));
#line 204
static void _hicc_test_204() { T& (Self::* _204)() = &Self::front; (void)_204; }
#line 204
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((T& (Self::*)())&Self::front));
#line 213
static void _hicc_test_213() { void (Self::* _213)(size_t, const T&) = &Self::assign; (void)_213; }
#line 213
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((void (Self::*)(size_t, const T&))&Self::assign));
#line 231
static void _hicc_test_231() { void (Self::* _231)() = &Self::pop_back; (void)_231; }
#line 231
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((void (Self::*)())&Self::pop_back));
#line 240
static void _hicc_test_240() { void (Self::* _240)(const T&) = &Self::push_back; (void)_240; }
#line 240
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((void (Self::*)(const T&))&Self::push_back));
#line 250
static void _hicc_test_250() { void (Self::* _250)(Self&) = &Self::swap; (void)_250; }
#line 250
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((void (Self::*)(Self&))&Self::swap));
#line 266
static void _hicc_test_266() { void (* _266)(Self&, size_t, size_t, const T&) = &SelfMethods::insert; (void)_266; }
#line 266
EXPORT_METHOD_IN(SelfContainer, Self, ((void (*)(Self&, size_t, size_t, const T&))&SelfMethods::insert));
#line 284
static void _hicc_test_284() { void (* _284)(Self&, size_t, size_t) = &SelfMethods::erase; (void)_284; }
#line 284
EXPORT_METHOD_IN(SelfContainer, Self, ((void (*)(Self&, size_t, size_t))&SelfMethods::erase));
#line 10
};
#line 346
template<class Allocator> struct VecBool_346;
#line 346
namespace hicc { template<class Allocator> struct MethodsType<std::vector<bool, Allocator>, void> { typedef VecBool_346<Allocator> methods_type; }; }
#line 346
template<class Allocator> struct VecBool_346 {
#line 346
typedef std::vector<bool, Allocator> Self; typedef void SelfContainer; typedef VecBool_346<Allocator> SelfMethods;
#line 438
static void set(Self& self, size_t pos, bool val) {
                if (pos < self.size()) {
                    self[pos] = val;
                }
            }
#line 522
static void push_back(Self& self, bool val) {
                self.push_back(val);
            }
#line 546
static void insert(Self& self, size_t pos, size_t count, bool val) {
                self.insert(self.begin() + std::min(pos, self.size()), count, val);
            }
#line 565
static void erase(Self& self, size_t pos, size_t count) {
                pos = std::min(pos, self.size());
                count = std::min(count, self.size() - pos);
                self.erase(self.begin() + pos, self.begin() + pos + count);
            }
#line 353
static void _hicc_test_353() { bool (Self::* _353)() const = &Self::empty; (void)_353; }
#line 353
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((bool (Self::*)() const)&Self::empty));
#line 361
static void _hicc_test_361() { size_t (Self::* _361)() const = &Self::size; (void)_361; }
#line 361
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((size_t (Self::*)() const)&Self::size));
#line 369
static void _hicc_test_369() { size_t (Self::* _369)() const = &Self::max_size; (void)_369; }
#line 369
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((size_t (Self::*)() const)&Self::max_size));
#line 377
static void _hicc_test_377() { size_t (Self::* _377)() const = &Self::capacity; (void)_377; }
#line 377
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((size_t (Self::*)() const)&Self::capacity));
#line 387
static void _hicc_test_387() { void (Self::* _387)() = &Self::clear; (void)_387; }
#line 387
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((void (Self::*)())&Self::clear));
#line 397
static void _hicc_test_397() { void (Self::* _397)(size_t) = &Self::reserve; (void)_397; }
#line 397
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((void (Self::*)(size_t))&Self::reserve));
#line 407
static void _hicc_test_407() { void (Self::* _407)() = &Self::shrink_to_fit; (void)_407; }
#line 407
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((void (Self::*)())&Self::shrink_to_fit));
#line 418
static void _hicc_test_418() { void (Self::* _418)(size_t, bool) = &Self::resize; (void)_418; }
#line 418
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((void (Self::*)(size_t, bool))&Self::resize));
#line 434
static void _hicc_test_434() { bool (Self::* _434)(size_t) const = &Self::at; (void)_434; }
#line 434
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((bool (Self::*)(size_t) const)&Self::at));
#line 456
static void _hicc_test_456() { void (* _456)(Self&, size_t, bool) = &SelfMethods::set; (void)_456; }
#line 456
EXPORT_METHOD_IN(SelfContainer, Self, ((void (*)(Self&, size_t, bool))&SelfMethods::set));
#line 472
static void _hicc_test_472() { bool (Self::* _472)() const = &Self::back; (void)_472; }
#line 472
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((bool (Self::*)() const)&Self::back));
#line 487
static void _hicc_test_487() { bool (Self::* _487)() const = &Self::front; (void)_487; }
#line 487
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((bool (Self::*)() const)&Self::front));
#line 501
static void _hicc_test_501() { void (Self::* _501)(size_t, const bool&) = &Self::assign; (void)_501; }
#line 501
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((void (Self::*)(size_t, const bool&))&Self::assign));
#line 518
static void _hicc_test_518() { void (Self::* _518)() = &Self::pop_back; (void)_518; }
#line 518
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((void (Self::*)())&Self::pop_back));
#line 532
static void _hicc_test_532() { void (* _532)(Self&, bool) = &SelfMethods::push_back; (void)_532; }
#line 532
EXPORT_METHOD_IN(SelfContainer, Self, ((void (*)(Self&, bool))&SelfMethods::push_back));
#line 542
static void _hicc_test_542() { void (Self::* _542)(Self&) = &Self::swap; (void)_542; }
#line 542
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((void (Self::*)(Self&))&Self::swap));
#line 561
static void _hicc_test_561() { void (* _561)(Self&, size_t, size_t, bool) = &SelfMethods::insert; (void)_561; }
#line 561
EXPORT_METHOD_IN(SelfContainer, Self, ((void (*)(Self&, size_t, size_t, bool))&SelfMethods::insert));
#line 580
static void _hicc_test_580() { void (* _580)(Self&, size_t, size_t) = &SelfMethods::erase; (void)_580; }
#line 580
EXPORT_METHOD_IN(SelfContainer, Self, ((void (*)(Self&, size_t, size_t))&SelfMethods::erase));
#line 346
};

#endif
