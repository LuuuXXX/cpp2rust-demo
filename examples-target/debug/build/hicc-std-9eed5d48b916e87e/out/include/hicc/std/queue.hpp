#ifndef __SRC_STD_QUEUE_RS
#define __SRC_STD_QUEUE_RS

#include <hicc/hicc.hpp>
#line 0 R"(./src/std_queue.rs)"
#line 2
#include <queue>
#line 6
template<class T, class Container> struct queue_6;
#line 6
namespace hicc { template<class T, class Container> struct MethodsType<std::queue<T, Container>, void> { typedef queue_6<T,Container> methods_type; }; }
#line 163
template<class T, class Container, class Compare> struct priority_queue_163;
#line 163
namespace hicc { template<class T, class Container, class Compare> struct MethodsType<std::priority_queue<T, Container, Compare>, void> { typedef priority_queue_163<T,Container,Compare> methods_type; }; }
#line 6
template<class T, class Container> struct queue_6 {
#line 6
typedef std::queue<T, Container> Self; typedef void SelfContainer; typedef queue_6<T,Container> SelfMethods;
#line 15
static void _hicc_test_15() { bool (Self::* _15)() const = &Self::empty; (void)_15; }
#line 15
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((bool (Self::*)() const)&Self::empty));
#line 25
static void _hicc_test_25() { size_t (Self::* _25)() const = &Self::size; (void)_25; }
#line 25
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((size_t (Self::*)() const)&Self::size));
#line 43
static void _hicc_test_43() { void (Self::* _43)() = &Self::pop; (void)_43; }
#line 43
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((void (Self::*)())&Self::pop));
#line 53
static void _hicc_test_53() { void (Self::* _53)(const T&) = &Self::push; (void)_53; }
#line 53
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((void (Self::*)(const T&))&Self::push));
#line 69
static void _hicc_test_69() { const T& (Self::* _69)() const = &Self::front; (void)_69; }
#line 69
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((const T& (Self::*)() const)&Self::front));
#line 96
static void _hicc_test_96() { T& (Self::* _96)() = &Self::front; (void)_96; }
#line 96
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((T& (Self::*)())&Self::front));
#line 113
static void _hicc_test_113() { const T& (Self::* _113)() const = &Self::back; (void)_113; }
#line 113
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((const T& (Self::*)() const)&Self::back));
#line 140
static void _hicc_test_140() { T& (Self::* _140)() = &Self::back; (void)_140; }
#line 140
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((T& (Self::*)())&Self::back));
#line 155
static void _hicc_test_155() { void (Self::* _155)(Self&) = &Self::swap; (void)_155; }
#line 155
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((void (Self::*)(Self&))&Self::swap));
#line 6
};
#line 163
template<class T, class Container, class Compare> struct priority_queue_163 {
#line 163
typedef std::priority_queue<T, Container, Compare> Self; typedef void SelfContainer; typedef priority_queue_163<T,Container,Compare> SelfMethods;
#line 172
static void _hicc_test_172() { bool (Self::* _172)() const = &Self::empty; (void)_172; }
#line 172
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((bool (Self::*)() const)&Self::empty));
#line 182
static void _hicc_test_182() { size_t (Self::* _182)() const = &Self::size; (void)_182; }
#line 182
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((size_t (Self::*)() const)&Self::size));
#line 200
static void _hicc_test_200() { void (Self::* _200)() = &Self::pop; (void)_200; }
#line 200
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((void (Self::*)())&Self::pop));
#line 209
static void _hicc_test_209() { void (Self::* _209)(const T&) = &Self::push; (void)_209; }
#line 209
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((void (Self::*)(const T&))&Self::push));
#line 227
static void _hicc_test_227() { const T& (Self::* _227)() const = &Self::top; (void)_227; }
#line 227
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((const T& (Self::*)() const)&Self::top));
#line 242
static void _hicc_test_242() { void (Self::* _242)(Self&) = &Self::swap; (void)_242; }
#line 242
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((void (Self::*)(Self&))&Self::swap));
#line 163
};

#endif
