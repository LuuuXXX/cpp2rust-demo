#ifndef __SRC_STD_FORWARD_LIST_RS
#define __SRC_STD_FORWARD_LIST_RS

#include <hicc/hicc.hpp>
#line 0 R"(./src/std_forward_list.rs)"
#line 5
#include <forward_list>
#line 9
template<class T, class Allocator> struct forward_list_9;
#line 9
namespace hicc { template<class T, class Allocator> struct MethodsType<std::forward_list<T, Allocator>, void> { typedef forward_list_9<T,Allocator> methods_type; }; }
#line 366
template<class T, class Allocator> struct CppForwardListIter_366;
#line 366
namespace hicc { template<class T, class Allocator> struct MethodsType<typename std::forward_list<T, Allocator>::const_iterator, std::forward_list<T, Allocator>> { typedef CppForwardListIter_366<T,Allocator> methods_type; }; }
#line 384
template<class T, class Allocator> struct CppForwardListIterMut_384;
#line 384
namespace hicc { template<class T, class Allocator> struct MethodsType<typename std::forward_list<T, Allocator>::iterator, std::forward_list<T, Allocator>> { typedef CppForwardListIterMut_384<T,Allocator> methods_type; }; }
#line 9
template<class T, class Allocator> struct forward_list_9 {
#line 9
typedef std::forward_list<T, Allocator> Self; typedef void SelfContainer; typedef forward_list_9<T,Allocator> SelfMethods;
#line 12
typedef typename Self::iterator iterator;
            typedef typename Self::const_iterator const_iterator;
#line 166
static void remove_if(Self& self, std::function<bool(const T&)> pred) {
                self.remove_if(pred);
            }
#line 186
static void sort(Self& self, std::function<bool(const T&, const T&)> comp) {
                self.sort(comp);
            }
#line 210
static void merge(Self& self, Self& other, std::function<bool(const T&, const T&)> comp) {
                if (self.get_allocator() == other.get_allocator()) {
                    self.merge(other, comp);
                }
            }
#line 245
static void unique(Self& self, std::function<bool(const T&, const T&)> comp) {
                self.unique(comp);
            }
#line 294
static void insert_after(Self& self, iterator& pos, size_t count, const T& val) {
                if (pos != self.end()) {
                    self.insert_after(pos, count, val);
                }
            }
            static void insert_after(Self& self, size_t count, const T& val) {
                    self.insert_after(self.before_begin(), count, val);
            }
#line 319
static void splice_after(Self& self, iterator& pos, Self& other) {
                if (pos != self.end() && self.get_allocator() == other.get_allocator() && &self != &other) {
                    self.splice_after(pos, other);
                }
            }
            static void splice_after(Self& self, Self& other) {
                if (self.get_allocator() == other.get_allocator() && &self != &other) {
                    self.splice_after(self.before_begin(), other);
                }
            }
#line 352
static void erase_after(Self& self, iterator& pos) {
                const_iterator it = pos;
                if (it != self.end() && ++it != self.end()) {
                    self.erase_after(pos);
                }
            }
#line 20
static void _hicc_test_20() { bool (Self::* _20)() const = &Self::empty; (void)_20; }
#line 20
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((bool (Self::*)() const)&Self::empty));
#line 29
static void _hicc_test_29() { size_t (Self::* _29)() const = &Self::max_size; (void)_29; }
#line 29
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((size_t (Self::*)() const)&Self::max_size));
#line 39
static void _hicc_test_39() { void (Self::* _39)() = &Self::clear; (void)_39; }
#line 39
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((void (Self::*)())&Self::clear));
#line 49
static void _hicc_test_49() { void (Self::* _49)(size_t, const T&) = &Self::resize; (void)_49; }
#line 49
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((void (Self::*)(size_t, const T&))&Self::resize));
#line 65
static void _hicc_test_65() { void (Self::* _65)() = &Self::pop_front; (void)_65; }
#line 65
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((void (Self::*)())&Self::pop_front));
#line 74
static void _hicc_test_74() { void (Self::* _74)(const T&) = &Self::push_front; (void)_74; }
#line 74
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((void (Self::*)(const T&))&Self::push_front));
#line 84
static void _hicc_test_84() { void (Self::* _84)(Self&) = &Self::swap; (void)_84; }
#line 84
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((void (Self::*)(Self&))&Self::swap));
#line 95
static void _hicc_test_95() { void (Self::* _95)(size_t, const T&) = &Self::assign; (void)_95; }
#line 95
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((void (Self::*)(size_t, const T&))&Self::assign));
#line 108
static void _hicc_test_108() { void (Self::* _108)() = &Self::reverse; (void)_108; }
#line 108
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((void (Self::*)())&Self::reverse));
#line 124
static void _hicc_test_124() { const T& (Self::* _124)() const = &Self::front; (void)_124; }
#line 124
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((const T& (Self::*)() const)&Self::front));
#line 147
static void _hicc_test_147() { const T& (Self::* _147)() const = &Self::front; (void)_147; }
#line 147
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((const T& (Self::*)() const)&Self::front));
#line 161
static void _hicc_test_161() { void (Self::* _161)(const T&) = &Self::remove; (void)_161; }
#line 161
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((void (Self::*)(const T&))&Self::remove));
#line 182
static void _hicc_test_182() { void (* _182)(Self&, std::function<bool(const T&)>) = &SelfMethods::remove_if; (void)_182; }
#line 182
EXPORT_METHOD_IN(SelfContainer, Self, ((void (*)(Self&, std::function<bool(const T&)>))&SelfMethods::remove_if));
#line 206
static void _hicc_test_206() { void (* _206)(Self&, std::function<bool(const T&, const T&)>) = &SelfMethods::sort; (void)_206; }
#line 206
EXPORT_METHOD_IN(SelfContainer, Self, ((void (*)(Self&, std::function<bool(const T&, const T&)>))&SelfMethods::sort));
#line 240
static void _hicc_test_240() { void (* _240)(Self&, Self&, std::function<bool(const T&, const T&)>) = &SelfMethods::merge; (void)_240; }
#line 240
EXPORT_METHOD_IN(SelfContainer, Self, ((void (*)(Self&, Self&, std::function<bool(const T&, const T&)>))&SelfMethods::merge));
#line 265
static void _hicc_test_265() { void (* _265)(Self&, std::function<bool(const T&, const T&)>) = &SelfMethods::unique; (void)_265; }
#line 265
EXPORT_METHOD_IN(SelfContainer, Self, ((void (*)(Self&, std::function<bool(const T&, const T&)>))&SelfMethods::unique));
#line 281
static void _hicc_test_281() { void (Self::* _281)() = &Self::unique; (void)_281; }
#line 281
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((void (Self::*)())&Self::unique));
#line 284
static void _hicc_test_284() { const_iterator (Self::* _284)() const = &Self::begin; (void)_284; }
#line 284
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((const_iterator (Self::*)() const)&Self::begin));
#line 286
static void _hicc_test_286() { const_iterator (Self::* _286)() const = &Self::end; (void)_286; }
#line 286
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((const_iterator (Self::*)() const)&Self::end));
#line 288
static void _hicc_test_288() { iterator (Self::* _288)() = &Self::begin; (void)_288; }
#line 288
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((iterator (Self::*)())&Self::begin));
#line 290
static void _hicc_test_290() { iterator (Self::* _290)() = &Self::end; (void)_290; }
#line 290
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((iterator (Self::*)())&Self::end));
#line 303
static void _hicc_test_303() { void (* _303)(Self&, iterator&, size_t, const T&) = &insert_after; (void)_303; }
#line 303
EXPORT_METHOD_IN(SelfContainer, Self, ((void (*)(Self&, iterator&, size_t, const T&))&insert_after));
#line 314
static void _hicc_test_314() { void (* _314)(Self&, size_t, const T&) = &insert_after; (void)_314; }
#line 314
EXPORT_METHOD_IN(SelfContainer, Self, ((void (*)(Self&, size_t, const T&))&insert_after));
#line 330
static void _hicc_test_330() { void (* _330)(Self&, iterator&, Self&) = &splice_after; (void)_330; }
#line 330
EXPORT_METHOD_IN(SelfContainer, Self, ((void (*)(Self&, iterator&, Self&))&splice_after));
#line 348
static void _hicc_test_348() { void (* _348)(Self&, Self&) = &splice_after; (void)_348; }
#line 348
EXPORT_METHOD_IN(SelfContainer, Self, ((void (*)(Self&, Self&))&splice_after));
#line 359
static void _hicc_test_359() { void (* _359)(Self&, iterator&) = &erase_after; (void)_359; }
#line 359
EXPORT_METHOD_IN(SelfContainer, Self, ((void (*)(Self&, iterator&))&erase_after));
#line 9
};
#line 366
template<class T, class Allocator> struct CppForwardListIter_366 {
#line 366
typedef typename std::forward_list<T, Allocator>::const_iterator Self; typedef std::forward_list<T, Allocator> SelfContainer; typedef CppForwardListIter_366<T,Allocator> SelfMethods;
#line 369
static const T& next(Self& self) {
                return *self++;
            }
            static const T& get(const Self& self) {
                return *self;
            }
#line 376
static void _hicc_test_376() { const T& (* _376)(Self&) = &SelfMethods::next; (void)_376; }
#line 376
EXPORT_METHOD_IN(SelfContainer, Self, ((const T& (*)(Self&))&SelfMethods::next));
#line 378
static void _hicc_test_378() { const T& (* _378)(const Self&) = &SelfMethods::get; (void)_378; }
#line 378
EXPORT_METHOD_IN(SelfContainer, Self, ((const T& (*)(const Self&))&SelfMethods::get));
#line 380
static void _hicc_test_380() { bool (* _380)(const Self&, const Self&) = &hicc::make_eq<Self, Self>; (void)_380; }
#line 380
EXPORT_METHOD_IN(SelfContainer, Self, ((bool (*)(const Self&, const Self&))&hicc::make_eq<Self, Self>));
#line 366
};
#line 384
template<class T, class Allocator> struct CppForwardListIterMut_384 {
#line 384
typedef typename std::forward_list<T, Allocator>::iterator Self; typedef std::forward_list<T, Allocator> SelfContainer; typedef CppForwardListIterMut_384<T,Allocator> SelfMethods;
#line 387
static T& next(Self& self) {
                return *self++;
            }
            static const T& get(const Self& self) {
                return *self;
            }
            static bool has_next(const Self& self, const Self& end) {
                return self != end && ++Self(self) != end;
            }
#line 397
static void _hicc_test_397() { T& (* _397)(Self&) = &SelfMethods::next; (void)_397; }
#line 397
EXPORT_METHOD_IN(SelfContainer, Self, ((T& (*)(Self&))&SelfMethods::next));
#line 399
static void _hicc_test_399() { const T& (* _399)(const Self&) = &SelfMethods::get; (void)_399; }
#line 399
EXPORT_METHOD_IN(SelfContainer, Self, ((const T& (*)(const Self&))&SelfMethods::get));
#line 401
static void _hicc_test_401() { const T& (* _401)(const Self&) = &SelfMethods::get; (void)_401; }
#line 401
EXPORT_METHOD_IN(SelfContainer, Self, ((const T& (*)(const Self&))&SelfMethods::get));
#line 403
static void _hicc_test_403() { bool (* _403)(const Self&, const Self&) = &hicc::make_eq<Self, Self>; (void)_403; }
#line 403
EXPORT_METHOD_IN(SelfContainer, Self, ((bool (*)(const Self&, const Self&))&hicc::make_eq<Self, Self>));
#line 405
static void _hicc_test_405() { bool (* _405)(const Self&, const Self&) = &SelfMethods::has_next; (void)_405; }
#line 405
EXPORT_METHOD_IN(SelfContainer, Self, ((bool (*)(const Self&, const Self&))&SelfMethods::has_next));
#line 384
};

#endif
