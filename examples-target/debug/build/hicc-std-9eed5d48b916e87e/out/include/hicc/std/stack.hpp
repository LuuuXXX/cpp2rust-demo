#ifndef __SRC_STD_STACK_RS
#define __SRC_STD_STACK_RS

#include <hicc/hicc.hpp>
#line 0 R"(./src/std_stack.rs)"
#line 2
#include <stack>
#line 6
template<class T, class Container> struct stack_6;
#line 6
namespace hicc { template<class T, class Container> struct MethodsType<std::stack<T, Container>, void> { typedef stack_6<T,Container> methods_type; }; }
#line 6
template<class T, class Container> struct stack_6 {
#line 6
typedef std::stack<T, Container> Self; typedef void SelfContainer; typedef stack_6<T,Container> SelfMethods;
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
#line 52
static void _hicc_test_52() { void (Self::* _52)(const T&) = &Self::push; (void)_52; }
#line 52
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((void (Self::*)(const T&))&Self::push));
#line 68
static void _hicc_test_68() { const T& (Self::* _68)() const = &Self::top; (void)_68; }
#line 68
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((const T& (Self::*)() const)&Self::top));
#line 93
static void _hicc_test_93() { T& (Self::* _93)() = &Self::top; (void)_93; }
#line 93
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((T& (Self::*)())&Self::top));
#line 108
static void _hicc_test_108() { void (Self::* _108)(Self&) = &Self::swap; (void)_108; }
#line 108
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((void (Self::*)(Self&))&Self::swap));
#line 6
};

#endif
