#ifndef __SRC_STD_SET_RS
#define __SRC_STD_SET_RS

#include <hicc/hicc.hpp>
#line 0 R"(./src/std_set.rs)"
#line 5
#include <set>
#line 9
template<class T, class Compare, class Allocator> struct set_9;
#line 9
namespace hicc { template<class T, class Compare, class Allocator> struct MethodsType<std::set<T, Compare, Allocator>, void> { typedef set_9<T,Compare,Allocator> methods_type; }; }
#line 135
template<class T, class Compare, class Allocator> struct CppSetIter_135;
#line 135
namespace hicc { template<class T, class Compare, class Allocator> struct MethodsType<typename std::set<T, Compare, Allocator>::const_iterator, std::set<T, Compare, Allocator>> { typedef CppSetIter_135<T,Compare,Allocator> methods_type; }; }
#line 151
template<class T, class Compare, class Allocator> struct CppSetRevIter_151;
#line 151
namespace hicc { template<class T, class Compare, class Allocator> struct MethodsType<typename std::set<T, Compare, Allocator>::const_reverse_iterator, std::set<T, Compare, Allocator>> { typedef CppSetRevIter_151<T,Compare,Allocator> methods_type; }; }
#line 9
template<class T, class Compare, class Allocator> struct set_9 {
#line 9
typedef std::set<T, Compare, Allocator> Self; typedef void SelfContainer; typedef set_9<T,Compare,Allocator> SelfMethods;
#line 12
typedef typename Self::const_iterator const_iterator;
            typedef typename Self::const_reverse_iterator const_reverse_iterator;
#line 69
static bool contains(const Self& self, const T& val) {
                return self.find(val) != self.end();
            }
#line 93
static bool insert(Self& self, const T& val) {
                return self.insert(val).second;
            }
#line 20
static void _hicc_test_20() { bool (Self::* _20)() const = &Self::empty; (void)_20; }
#line 20
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((bool (Self::*)() const)&Self::empty));
#line 28
static void _hicc_test_28() { size_t (Self::* _28)() const = &Self::size; (void)_28; }
#line 28
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((size_t (Self::*)() const)&Self::size));
#line 36
static void _hicc_test_36() { size_t (Self::* _36)() const = &Self::max_size; (void)_36; }
#line 36
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((size_t (Self::*)() const)&Self::max_size));
#line 46
static void _hicc_test_46() { void (Self::* _46)() = &Self::clear; (void)_46; }
#line 46
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((void (Self::*)())&Self::clear));
#line 56
static void _hicc_test_56() { void (Self::* _56)(Self&) = &Self::swap; (void)_56; }
#line 56
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((void (Self::*)(Self&))&Self::swap));
#line 65
static void _hicc_test_65() { size_t (Self::* _65)(const T&) const = &Self::count; (void)_65; }
#line 65
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((size_t (Self::*)(const T&) const)&Self::count));
#line 79
static void _hicc_test_79() { bool (* _79)(const Self&, const T&) = &SelfMethods::contains; (void)_79; }
#line 79
EXPORT_METHOD_IN(SelfContainer, Self, ((bool (*)(const Self&, const T&))&SelfMethods::contains));
#line 89
static void _hicc_test_89() { void (* _89)(Self&, const Self&) = &hicc::make_assign<Self, Self>; (void)_89; }
#line 89
EXPORT_METHOD_IN(SelfContainer, Self, ((void (*)(Self&, const Self&))&hicc::make_assign<Self, Self>));
#line 103
static void _hicc_test_103() { bool (* _103)(Self&, const T&) = &SelfMethods::insert; (void)_103; }
#line 103
EXPORT_METHOD_IN(SelfContainer, Self, ((bool (*)(Self&, const T&))&SelfMethods::insert));
#line 113
static void _hicc_test_113() { size_t (Self::* _113)(const T&) = &Self::erase; (void)_113; }
#line 113
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((size_t (Self::*)(const T&))&Self::erase));
#line 116
static void _hicc_test_116() { const_iterator (Self::* _116)(const T&) const = &Self::find; (void)_116; }
#line 116
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((const_iterator (Self::*)(const T&) const)&Self::find));
#line 118
static void _hicc_test_118() { const_iterator (Self::* _118)(const T&) const = &Self::lower_bound; (void)_118; }
#line 118
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((const_iterator (Self::*)(const T&) const)&Self::lower_bound));
#line 120
static void _hicc_test_120() { const_iterator (Self::* _120)(const T&) const = &Self::upper_bound; (void)_120; }
#line 120
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((const_iterator (Self::*)(const T&) const)&Self::upper_bound));
#line 122
static void _hicc_test_122() { const_iterator (Self::* _122)() const = &Self::begin; (void)_122; }
#line 122
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((const_iterator (Self::*)() const)&Self::begin));
#line 124
static void _hicc_test_124() { const_iterator (Self::* _124)() const = &Self::end; (void)_124; }
#line 124
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((const_iterator (Self::*)() const)&Self::end));
#line 126
static void _hicc_test_126() { const_reverse_iterator (Self::* _126)() const = &Self::rbegin; (void)_126; }
#line 126
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((const_reverse_iterator (Self::*)() const)&Self::rbegin));
#line 128
static void _hicc_test_128() { const_reverse_iterator (Self::* _128)() const = &Self::rend; (void)_128; }
#line 128
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((const_reverse_iterator (Self::*)() const)&Self::rend));
#line 9
};
#line 135
template<class T, class Compare, class Allocator> struct CppSetIter_135 {
#line 135
typedef typename std::set<T, Compare, Allocator>::const_iterator Self; typedef std::set<T, Compare, Allocator> SelfContainer; typedef CppSetIter_135<T,Compare,Allocator> SelfMethods;
#line 138
typedef typename SelfContainer::const_reverse_iterator const_reverse_iterator;
            static const T& next(Self& self) {
                return *self++;
            }
#line 143
static void _hicc_test_143() { const T& (* _143)(Self&) = &SelfMethods::next; (void)_143; }
#line 143
EXPORT_METHOD_IN(SelfContainer, Self, ((const T& (*)(Self&))&SelfMethods::next));
#line 145
static void _hicc_test_145() { bool (* _145)(const Self&, const Self&) = &hicc::make_eq<Self, Self>; (void)_145; }
#line 145
EXPORT_METHOD_IN(SelfContainer, Self, ((bool (*)(const Self&, const Self&))&hicc::make_eq<Self, Self>));
#line 147
static void _hicc_test_147() { const_reverse_iterator (* _147)(Self&&) = &hicc::make_constructor<const_reverse_iterator, Self>; (void)_147; }
#line 147
EXPORT_METHOD_IN(SelfContainer, Self, ((const_reverse_iterator (*)(Self&&))&hicc::make_constructor<const_reverse_iterator, Self>));
#line 135
};
#line 151
template<class T, class Compare, class Allocator> struct CppSetRevIter_151 {
#line 151
typedef typename std::set<T, Compare, Allocator>::const_reverse_iterator Self; typedef std::set<T, Compare, Allocator> SelfContainer; typedef CppSetRevIter_151<T,Compare,Allocator> SelfMethods;
#line 154
static const T& next(Self& self) {
                return *self++;
            }
#line 158
static void _hicc_test_158() { const T& (* _158)(Self&) = &SelfMethods::next; (void)_158; }
#line 158
EXPORT_METHOD_IN(SelfContainer, Self, ((const T& (*)(Self&))&SelfMethods::next));
#line 160
static void _hicc_test_160() { bool (* _160)(const Self&, const Self&) = &hicc::make_eq<Self, Self>; (void)_160; }
#line 160
EXPORT_METHOD_IN(SelfContainer, Self, ((bool (*)(const Self&, const Self&))&hicc::make_eq<Self, Self>));
#line 151
};
#line 286
template<class T, class Compare, class Allocator> struct multiset_286;
#line 286
namespace hicc { template<class T, class Compare, class Allocator> struct MethodsType<std::multiset<T, Compare, Allocator>, void> { typedef multiset_286<T,Compare,Allocator> methods_type; }; }
#line 410
template<class T, class Compare, class Allocator> struct CppMultiSetIter_410;
#line 410
namespace hicc { template<class T, class Compare, class Allocator> struct MethodsType<typename std::multiset<T, Compare, Allocator>::const_iterator, std::multiset<T, Compare, Allocator>> { typedef CppMultiSetIter_410<T,Compare,Allocator> methods_type; }; }
#line 426
template<class T, class Compare, class Allocator> struct CppMultiSetRevIter_426;
#line 426
namespace hicc { template<class T, class Compare, class Allocator> struct MethodsType<typename std::multiset<T, Compare, Allocator>::const_reverse_iterator, std::multiset<T, Compare, Allocator>> { typedef CppMultiSetRevIter_426<T,Compare,Allocator> methods_type; }; }
#line 286
template<class T, class Compare, class Allocator> struct multiset_286 {
#line 286
typedef std::multiset<T, Compare, Allocator> Self; typedef void SelfContainer; typedef multiset_286<T,Compare,Allocator> SelfMethods;
#line 289
typedef typename Self::iterator iterator;
            typedef typename Self::const_iterator const_iterator;
            typedef typename Self::const_reverse_iterator const_reverse_iterator;
#line 347
static bool contains(const Self& self, const T& val) {
                return self.find(val) != self.end();
            }
#line 298
static void _hicc_test_298() { bool (Self::* _298)() const = &Self::empty; (void)_298; }
#line 298
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((bool (Self::*)() const)&Self::empty));
#line 306
static void _hicc_test_306() { size_t (Self::* _306)() const = &Self::size; (void)_306; }
#line 306
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((size_t (Self::*)() const)&Self::size));
#line 314
static void _hicc_test_314() { size_t (Self::* _314)() const = &Self::max_size; (void)_314; }
#line 314
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((size_t (Self::*)() const)&Self::max_size));
#line 324
static void _hicc_test_324() { void (Self::* _324)() = &Self::clear; (void)_324; }
#line 324
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((void (Self::*)())&Self::clear));
#line 334
static void _hicc_test_334() { void (Self::* _334)(Self&) = &Self::swap; (void)_334; }
#line 334
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((void (Self::*)(Self&))&Self::swap));
#line 343
static void _hicc_test_343() { size_t (Self::* _343)(const T&) const = &Self::count; (void)_343; }
#line 343
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((size_t (Self::*)(const T&) const)&Self::count));
#line 357
static void _hicc_test_357() { bool (* _357)(const Self&, const T&) = &SelfMethods::contains; (void)_357; }
#line 357
EXPORT_METHOD_IN(SelfContainer, Self, ((bool (*)(const Self&, const T&))&SelfMethods::contains));
#line 367
static void _hicc_test_367() { void (* _367)(Self&, const Self&) = &hicc::make_assign<Self, Self>; (void)_367; }
#line 367
EXPORT_METHOD_IN(SelfContainer, Self, ((void (*)(Self&, const Self&))&hicc::make_assign<Self, Self>));
#line 377
static void _hicc_test_377() { iterator (Self::* _377)(const T&) = &Self::insert; (void)_377; }
#line 377
static void _e9cf81a88e0192cb33260d269f9e9c90_377(Self& self, const T& _0){ self.insert(_0); }
#line 377
EXPORT_METHOD_IN(SelfContainer, Self, SelfMethods::_e9cf81a88e0192cb33260d269f9e9c90_377);
#line 387
static void _hicc_test_387() { size_t (Self::* _387)(const T&) = &Self::erase; (void)_387; }
#line 387
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((size_t (Self::*)(const T&))&Self::erase));
#line 390
static void _hicc_test_390() { const_iterator (Self::* _390)(const T&) const = &Self::find; (void)_390; }
#line 390
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((const_iterator (Self::*)(const T&) const)&Self::find));
#line 392
static void _hicc_test_392() { const_iterator (Self::* _392)(const T&) const = &Self::lower_bound; (void)_392; }
#line 392
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((const_iterator (Self::*)(const T&) const)&Self::lower_bound));
#line 394
static void _hicc_test_394() { const_iterator (Self::* _394)(const T&) const = &Self::upper_bound; (void)_394; }
#line 394
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((const_iterator (Self::*)(const T&) const)&Self::upper_bound));
#line 396
static void _hicc_test_396() { const_iterator (Self::* _396)() const = &Self::begin; (void)_396; }
#line 396
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((const_iterator (Self::*)() const)&Self::begin));
#line 398
static void _hicc_test_398() { const_iterator (Self::* _398)() const = &Self::end; (void)_398; }
#line 398
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((const_iterator (Self::*)() const)&Self::end));
#line 400
static void _hicc_test_400() { const_reverse_iterator (Self::* _400)() const = &Self::crbegin; (void)_400; }
#line 400
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((const_reverse_iterator (Self::*)() const)&Self::crbegin));
#line 402
static void _hicc_test_402() { const_reverse_iterator (Self::* _402)() const = &Self::crend; (void)_402; }
#line 402
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((const_reverse_iterator (Self::*)() const)&Self::crend));
#line 286
};
#line 410
template<class T, class Compare, class Allocator> struct CppMultiSetIter_410 {
#line 410
typedef typename std::multiset<T, Compare, Allocator>::const_iterator Self; typedef std::multiset<T, Compare, Allocator> SelfContainer; typedef CppMultiSetIter_410<T,Compare,Allocator> SelfMethods;
#line 413
typedef typename SelfContainer::const_reverse_iterator const_reverse_iterator;
            static const T& next(Self& self) {
                return *self++;
            }
#line 418
static void _hicc_test_418() { const T& (* _418)(Self&) = &SelfMethods::next; (void)_418; }
#line 418
EXPORT_METHOD_IN(SelfContainer, Self, ((const T& (*)(Self&))&SelfMethods::next));
#line 420
static void _hicc_test_420() { bool (* _420)(const Self&, const Self&) = &hicc::make_eq<Self, Self>; (void)_420; }
#line 420
EXPORT_METHOD_IN(SelfContainer, Self, ((bool (*)(const Self&, const Self&))&hicc::make_eq<Self, Self>));
#line 422
static void _hicc_test_422() { const_reverse_iterator (* _422)(Self&&) = &hicc::make_constructor<const_reverse_iterator, Self>; (void)_422; }
#line 422
EXPORT_METHOD_IN(SelfContainer, Self, ((const_reverse_iterator (*)(Self&&))&hicc::make_constructor<const_reverse_iterator, Self>));
#line 410
};
#line 426
template<class T, class Compare, class Allocator> struct CppMultiSetRevIter_426 {
#line 426
typedef typename std::multiset<T, Compare, Allocator>::const_reverse_iterator Self; typedef std::multiset<T, Compare, Allocator> SelfContainer; typedef CppMultiSetRevIter_426<T,Compare,Allocator> SelfMethods;
#line 429
static const T& next(Self& self) {
                return *self++;
            }
#line 433
static void _hicc_test_433() { const T& (* _433)(Self&) = &SelfMethods::next; (void)_433; }
#line 433
EXPORT_METHOD_IN(SelfContainer, Self, ((const T& (*)(Self&))&SelfMethods::next));
#line 435
static void _hicc_test_435() { bool (* _435)(const Self&, const Self&) = &hicc::make_eq<Self, Self>; (void)_435; }
#line 435
EXPORT_METHOD_IN(SelfContainer, Self, ((bool (*)(const Self&, const Self&))&hicc::make_eq<Self, Self>));
#line 426
};

#endif
