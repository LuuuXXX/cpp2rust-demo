#ifndef __SRC_STD_LIST_RS
#define __SRC_STD_LIST_RS

#include <hicc/hicc.hpp>
#line 0 R"(./src/std_list.rs)"
#line 5
#include <list>
#line 9
template<class T, class Allocator> struct list_9;
#line 9
namespace hicc { template<class T, class Allocator> struct MethodsType<std::list<T, Allocator>, void> { typedef list_9<T,Allocator> methods_type; }; }
#line 413
template<class T, class Allocator> struct CppListIter_413;
#line 413
namespace hicc { template<class T, class Allocator> struct MethodsType<typename std::list<T, Allocator>::const_iterator, std::list<T, Allocator>> { typedef CppListIter_413<T,Allocator> methods_type; }; }
#line 426
template<class T, class Allocator> struct CppListIterMut_426;
#line 426
namespace hicc { template<class T, class Allocator> struct MethodsType<typename std::list<T, Allocator>::iterator, std::list<T, Allocator>> { typedef CppListIterMut_426<T,Allocator> methods_type; }; }
#line 439
template<class T, class Allocator> struct CppListRevIter_439;
#line 439
namespace hicc { template<class T, class Allocator> struct MethodsType<typename std::list<T, Allocator>::const_reverse_iterator, std::list<T, Allocator>> { typedef CppListRevIter_439<T,Allocator> methods_type; }; }
#line 452
template<class T, class Allocator> struct CppListRevIterMut_452;
#line 452
namespace hicc { template<class T, class Allocator> struct MethodsType<typename std::list<T, Allocator>::reverse_iterator, std::list<T, Allocator>> { typedef CppListRevIterMut_452<T,Allocator> methods_type; }; }
#line 9
template<class T, class Allocator> struct list_9 {
#line 9
typedef std::list<T, Allocator> Self; typedef void SelfContainer; typedef list_9<T,Allocator> SelfMethods;
#line 12
typedef typename Self::iterator iterator;
            typedef typename Self::const_iterator const_iterator;
            typedef typename Self::reverse_iterator reverse_iterator;
            typedef typename Self::const_reverse_iterator const_reverse_iterator;
#line 239
static void remove_if(Self& self, std::function<bool(const T&)> comp) {
                self.remove_if(comp);
            }
#line 259
static void sort(Self& self, std::function<bool(const T&, const T&)> comp) {
                self.sort(comp);
            }
#line 283
static void merge(Self& self, Self& other, std::function<bool(const T&, const T&)> comp) {
                if (self.get_allocator() == other.get_allocator()) {
                    self.merge(other, comp);
                }
            }
#line 317
static void unique(Self& self, std::function<bool(const T&, const T&)> comp) {
                self.unique(comp);
            }
#line 379
static void insert(Self& self, iterator& pos, size_t ncount, const T& val, iterator& end) {
                pos = self.insert(pos, ncount, val);
                end = self.end();
            }
#line 388
static void splice(Self& self, iterator& pos, Self& other, iterator& end) {
                if (&self != &other && self.get_allocator() == other.get_allocator()) {
                    self.splice(pos, other);
                    end = self.end();
                }
            }
#line 399
static void erase(Self& self, iterator& pos, iterator& end) {
                if (pos != self.end()) {
                    pos = self.erase(pos);
                    end = self.end();
                }
            }
#line 22
static void _hicc_test_22() { bool (Self::* _22)() const = &Self::empty; (void)_22; }
#line 22
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((bool (Self::*)() const)&Self::empty));
#line 30
static void _hicc_test_30() { size_t (Self::* _30)() const = &Self::size; (void)_30; }
#line 30
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((size_t (Self::*)() const)&Self::size));
#line 38
static void _hicc_test_38() { size_t (Self::* _38)() const = &Self::max_size; (void)_38; }
#line 38
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((size_t (Self::*)() const)&Self::max_size));
#line 48
static void _hicc_test_48() { void (Self::* _48)() = &Self::clear; (void)_48; }
#line 48
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((void (Self::*)())&Self::clear));
#line 58
static void _hicc_test_58() { void (Self::* _58)(size_t, const T&) = &Self::resize; (void)_58; }
#line 58
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((void (Self::*)(size_t, const T&))&Self::resize));
#line 74
static void _hicc_test_74() { void (Self::* _74)() = &Self::pop_back; (void)_74; }
#line 74
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((void (Self::*)())&Self::pop_back));
#line 84
static void _hicc_test_84() { void (Self::* _84)(const T&) = &Self::push_back; (void)_84; }
#line 84
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((void (Self::*)(const T&))&Self::push_back));
#line 100
static void _hicc_test_100() { void (Self::* _100)() = &Self::pop_front; (void)_100; }
#line 100
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((void (Self::*)())&Self::pop_front));
#line 110
static void _hicc_test_110() { void (Self::* _110)(const T&) = &Self::push_front; (void)_110; }
#line 110
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((void (Self::*)(const T&))&Self::push_front));
#line 120
static void _hicc_test_120() { void (Self::* _120)(Self&) = &Self::swap; (void)_120; }
#line 120
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((void (Self::*)(Self&))&Self::swap));
#line 130
static void _hicc_test_130() { void (Self::* _130)(size_t, const T&) = &Self::assign; (void)_130; }
#line 130
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((void (Self::*)(size_t, const T&))&Self::assign));
#line 144
static void _hicc_test_144() { void (Self::* _144)() = &Self::reverse; (void)_144; }
#line 144
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((void (Self::*)())&Self::reverse));
#line 160
static void _hicc_test_160() { const T& (Self::* _160)() const = &Self::back; (void)_160; }
#line 160
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((const T& (Self::*)() const)&Self::back));
#line 184
static void _hicc_test_184() { T& (Self::* _184)() = &Self::back; (void)_184; }
#line 184
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((T& (Self::*)())&Self::back));
#line 199
static void _hicc_test_199() { const T& (Self::* _199)() const = &Self::front; (void)_199; }
#line 199
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((const T& (Self::*)() const)&Self::front));
#line 222
static void _hicc_test_222() { T& (Self::* _222)() = &Self::front; (void)_222; }
#line 222
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((T& (Self::*)())&Self::front));
#line 235
static void _hicc_test_235() { void (Self::* _235)(const T&) = &Self::remove; (void)_235; }
#line 235
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((void (Self::*)(const T&))&Self::remove));
#line 255
static void _hicc_test_255() { void (* _255)(Self&, std::function<bool(const T&)>) = &SelfMethods::remove_if; (void)_255; }
#line 255
EXPORT_METHOD_IN(SelfContainer, Self, ((void (*)(Self&, std::function<bool(const T&)>))&SelfMethods::remove_if));
#line 279
static void _hicc_test_279() { void (* _279)(Self&, std::function<bool(const T&, const T&)>) = &SelfMethods::sort; (void)_279; }
#line 279
EXPORT_METHOD_IN(SelfContainer, Self, ((void (*)(Self&, std::function<bool(const T&, const T&)>))&SelfMethods::sort));
#line 313
static void _hicc_test_313() { void (* _313)(Self&, Self&, std::function<bool(const T&, const T&)>) = &SelfMethods::merge; (void)_313; }
#line 313
EXPORT_METHOD_IN(SelfContainer, Self, ((void (*)(Self&, Self&, std::function<bool(const T&, const T&)>))&SelfMethods::merge));
#line 337
static void _hicc_test_337() { void (* _337)(Self&, std::function<bool(const T&, const T&)>) = &SelfMethods::unique; (void)_337; }
#line 337
EXPORT_METHOD_IN(SelfContainer, Self, ((void (*)(Self&, std::function<bool(const T&, const T&)>))&SelfMethods::unique));
#line 353
static void _hicc_test_353() { void (Self::* _353)() = &Self::unique; (void)_353; }
#line 353
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((void (Self::*)())&Self::unique));
#line 356
static void _hicc_test_356() { const_iterator (Self::* _356)() const = &Self::begin; (void)_356; }
#line 356
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((const_iterator (Self::*)() const)&Self::begin));
#line 358
static void _hicc_test_358() { const_iterator (Self::* _358)() const = &Self::end; (void)_358; }
#line 358
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((const_iterator (Self::*)() const)&Self::end));
#line 362
static void _hicc_test_362() { iterator (Self::* _362)() = &Self::begin; (void)_362; }
#line 362
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((iterator (Self::*)())&Self::begin));
#line 364
static void _hicc_test_364() { iterator (Self::* _364)() = &Self::end; (void)_364; }
#line 364
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((iterator (Self::*)())&Self::end));
#line 367
static void _hicc_test_367() { const_reverse_iterator (Self::* _367)() const = &Self::rbegin; (void)_367; }
#line 367
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((const_reverse_iterator (Self::*)() const)&Self::rbegin));
#line 369
static void _hicc_test_369() { const_reverse_iterator (Self::* _369)() const = &Self::rend; (void)_369; }
#line 369
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((const_reverse_iterator (Self::*)() const)&Self::rend));
#line 373
static void _hicc_test_373() { reverse_iterator (Self::* _373)() = &Self::rbegin; (void)_373; }
#line 373
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((reverse_iterator (Self::*)())&Self::rbegin));
#line 375
static void _hicc_test_375() { reverse_iterator (Self::* _375)() = &Self::rend; (void)_375; }
#line 375
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((reverse_iterator (Self::*)())&Self::rend));
#line 384
static void _hicc_test_384() { void (* _384)(Self&, iterator&, size_t, const T&, iterator&) = &SelfMethods::insert; (void)_384; }
#line 384
EXPORT_METHOD_IN(SelfContainer, Self, ((void (*)(Self&, iterator&, size_t, const T&, iterator&))&SelfMethods::insert));
#line 395
static void _hicc_test_395() { void (* _395)(Self&, iterator&, Self&, iterator&) = &SelfMethods::splice; (void)_395; }
#line 395
EXPORT_METHOD_IN(SelfContainer, Self, ((void (*)(Self&, iterator&, Self&, iterator&))&SelfMethods::splice));
#line 406
static void _hicc_test_406() { void (* _406)(Self&, iterator&, iterator&) = &SelfMethods::erase; (void)_406; }
#line 406
EXPORT_METHOD_IN(SelfContainer, Self, ((void (*)(Self&, iterator&, iterator&))&SelfMethods::erase));
#line 9
};
#line 413
template<class T, class Allocator> struct CppListIter_413 {
#line 413
typedef typename std::list<T, Allocator>::const_iterator Self; typedef std::list<T, Allocator> SelfContainer; typedef CppListIter_413<T,Allocator> SelfMethods;
#line 416
static const T& next(Self& self) {
                return *self++;
            }
#line 420
static void _hicc_test_420() { const T& (* _420)(Self&) = &SelfMethods::next; (void)_420; }
#line 420
EXPORT_METHOD_IN(SelfContainer, Self, ((const T& (*)(Self&))&SelfMethods::next));
#line 422
static void _hicc_test_422() { bool (* _422)(const Self&, const Self&) = &hicc::make_eq<Self, Self>; (void)_422; }
#line 422
EXPORT_METHOD_IN(SelfContainer, Self, ((bool (*)(const Self&, const Self&))&hicc::make_eq<Self, Self>));
#line 413
};
#line 426
template<class T, class Allocator> struct CppListIterMut_426 {
#line 426
typedef typename std::list<T, Allocator>::iterator Self; typedef std::list<T, Allocator> SelfContainer; typedef CppListIterMut_426<T,Allocator> SelfMethods;
#line 429
static T& next(Self& self) {
                return *self++;
            }
#line 433
static void _hicc_test_433() { T& (* _433)(Self&) = &SelfMethods::next; (void)_433; }
#line 433
EXPORT_METHOD_IN(SelfContainer, Self, ((T& (*)(Self&))&SelfMethods::next));
#line 435
static void _hicc_test_435() { bool (* _435)(const Self&, const Self&) = &hicc::make_eq; (void)_435; }
#line 435
EXPORT_METHOD_IN(SelfContainer, Self, ((bool (*)(const Self&, const Self&))&hicc::make_eq));
#line 426
};
#line 439
template<class T, class Allocator> struct CppListRevIter_439 {
#line 439
typedef typename std::list<T, Allocator>::const_reverse_iterator Self; typedef std::list<T, Allocator> SelfContainer; typedef CppListRevIter_439<T,Allocator> SelfMethods;
#line 442
static const T& next(Self& self) {
                return *self++;
            }
#line 446
static void _hicc_test_446() { const T& (* _446)(Self&) = &SelfMethods::next; (void)_446; }
#line 446
EXPORT_METHOD_IN(SelfContainer, Self, ((const T& (*)(Self&))&SelfMethods::next));
#line 448
static void _hicc_test_448() { bool (* _448)(const Self&, const Self&) = &hicc::make_eq; (void)_448; }
#line 448
EXPORT_METHOD_IN(SelfContainer, Self, ((bool (*)(const Self&, const Self&))&hicc::make_eq));
#line 439
};
#line 452
template<class T, class Allocator> struct CppListRevIterMut_452 {
#line 452
typedef typename std::list<T, Allocator>::reverse_iterator Self; typedef std::list<T, Allocator> SelfContainer; typedef CppListRevIterMut_452<T,Allocator> SelfMethods;
#line 455
static T& next(Self& self) {
                return *self++;
            }
#line 459
static void _hicc_test_459() { T& (* _459)(Self&) = &SelfMethods::next; (void)_459; }
#line 459
EXPORT_METHOD_IN(SelfContainer, Self, ((T& (*)(Self&))&SelfMethods::next));
#line 461
static void _hicc_test_461() { bool (* _461)(const Self&, const Self&) = &hicc::make_eq<Self, Self>; (void)_461; }
#line 461
EXPORT_METHOD_IN(SelfContainer, Self, ((bool (*)(const Self&, const Self&))&hicc::make_eq<Self, Self>));
#line 452
};

#endif
