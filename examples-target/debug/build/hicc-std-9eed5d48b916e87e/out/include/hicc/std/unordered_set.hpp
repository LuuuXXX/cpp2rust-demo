#ifndef __SRC_STD_UNORDERED_SET_RS
#define __SRC_STD_UNORDERED_SET_RS

#include <hicc/hicc.hpp>
#line 0 R"(./src/std_unordered_set.rs)"
#line 5
#include <unordered_set>
#line 9
template <class T, class Hash, class Pred, class Allocator> struct unordered_set_9;
#line 9
namespace hicc { template <class T, class Hash, class Pred, class Allocator> struct MethodsType<std::unordered_set<T, Hash, Pred, Allocator>, void> { typedef unordered_set_9<T,Hash,Pred,Allocator> methods_type; }; }
#line 125
template <class T, class Hash, class Pred, class Allocator> struct CppUnorderedSetIter_125;
#line 125
namespace hicc { template <class T, class Hash, class Pred, class Allocator> struct MethodsType<typename std::unordered_set<T, Hash, Pred, Allocator>::const_iterator, std::unordered_set<T, Hash, Pred, Allocator>> { typedef CppUnorderedSetIter_125<T,Hash,Pred,Allocator> methods_type; }; }
#line 9
template <class T, class Hash, class Pred, class Allocator> struct unordered_set_9 {
#line 9
typedef std::unordered_set<T, Hash, Pred, Allocator> Self; typedef void SelfContainer; typedef unordered_set_9<T,Hash,Pred,Allocator> SelfMethods;
#line 12
typedef typename Self::iterator iterator;
            typedef typename Self::const_iterator const_iterator;
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
#line 112
static void _hicc_test_112() { size_t (Self::* _112)(const T&) = &Self::erase; (void)_112; }
#line 112
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((size_t (Self::*)(const T&))&Self::erase));
#line 115
static void _hicc_test_115() { const_iterator (Self::* _115)() const = &Self::begin; (void)_115; }
#line 115
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((const_iterator (Self::*)() const)&Self::begin));
#line 117
static void _hicc_test_117() { const_iterator (Self::* _117)() const = &Self::end; (void)_117; }
#line 117
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((const_iterator (Self::*)() const)&Self::end));
#line 9
};
#line 125
template <class T, class Hash, class Pred, class Allocator> struct CppUnorderedSetIter_125 {
#line 125
typedef typename std::unordered_set<T, Hash, Pred, Allocator>::const_iterator Self; typedef std::unordered_set<T, Hash, Pred, Allocator> SelfContainer; typedef CppUnorderedSetIter_125<T,Hash,Pred,Allocator> SelfMethods;
#line 128
static const T& next(Self& self) {
                return *self++;
            }
#line 132
static void _hicc_test_132() { const T& (* _132)(Self&) = &next; (void)_132; }
#line 132
EXPORT_METHOD_IN(SelfContainer, Self, ((const T& (*)(Self&))&next));
#line 134
static void _hicc_test_134() { bool (* _134)(const Self&, const Self&) = &hicc::make_eq<Self, Self>; (void)_134; }
#line 134
EXPORT_METHOD_IN(SelfContainer, Self, ((bool (*)(const Self&, const Self&))&hicc::make_eq<Self, Self>));
#line 125
};
#line 172
template <class T, class Hash, class Pred, class Allocator> struct unordered_multiset_172;
#line 172
namespace hicc { template <class T, class Hash, class Pred, class Allocator> struct MethodsType<std::unordered_multiset<T, Hash, Pred, Allocator>, void> { typedef unordered_multiset_172<T,Hash,Pred,Allocator> methods_type; }; }
#line 285
template <class T, class Hash, class Pred, class Allocator> struct CppUnorderedMultiSetIter_285;
#line 285
namespace hicc { template <class T, class Hash, class Pred, class Allocator> struct MethodsType<typename std::unordered_multiset<T, Hash, Pred, Allocator>::const_iterator, std::unordered_multiset<T, Hash, Pred, Allocator>> { typedef CppUnorderedMultiSetIter_285<T,Hash,Pred,Allocator> methods_type; }; }
#line 172
template <class T, class Hash, class Pred, class Allocator> struct unordered_multiset_172 {
#line 172
typedef std::unordered_multiset<T, Hash, Pred, Allocator> Self; typedef void SelfContainer; typedef unordered_multiset_172<T,Hash,Pred,Allocator> SelfMethods;
#line 175
typedef typename Self::iterator iterator;
            typedef typename Self::const_iterator const_iterator;
#line 232
static bool contains(const Self& self, const T& val) {
                return self.find(val) != self.end();
            }
#line 183
static void _hicc_test_183() { bool (Self::* _183)() const = &Self::empty; (void)_183; }
#line 183
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((bool (Self::*)() const)&Self::empty));
#line 191
static void _hicc_test_191() { size_t (Self::* _191)() const = &Self::size; (void)_191; }
#line 191
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((size_t (Self::*)() const)&Self::size));
#line 199
static void _hicc_test_199() { size_t (Self::* _199)() const = &Self::max_size; (void)_199; }
#line 199
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((size_t (Self::*)() const)&Self::max_size));
#line 209
static void _hicc_test_209() { void (Self::* _209)() = &Self::clear; (void)_209; }
#line 209
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((void (Self::*)())&Self::clear));
#line 219
static void _hicc_test_219() { void (Self::* _219)(Self&) = &Self::swap; (void)_219; }
#line 219
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((void (Self::*)(Self&))&Self::swap));
#line 228
static void _hicc_test_228() { size_t (Self::* _228)(const T&) const = &Self::count; (void)_228; }
#line 228
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((size_t (Self::*)(const T&) const)&Self::count));
#line 242
static void _hicc_test_242() { bool (* _242)(const Self&, const T&) = &SelfMethods::contains; (void)_242; }
#line 242
EXPORT_METHOD_IN(SelfContainer, Self, ((bool (*)(const Self&, const T&))&SelfMethods::contains));
#line 252
static void _hicc_test_252() { void (* _252)(Self&, const Self&) = &hicc::make_assign<Self, Self>; (void)_252; }
#line 252
EXPORT_METHOD_IN(SelfContainer, Self, ((void (*)(Self&, const Self&))&hicc::make_assign<Self, Self>));
#line 262
static void _hicc_test_262() { iterator (Self::* _262)(const T&) = &Self::insert; (void)_262; }
#line 262
static void _e9cf81a88e0192cb33260d269f9e9c90_262(Self& self, const T& _0){ self.insert(_0); }
#line 262
EXPORT_METHOD_IN(SelfContainer, Self, SelfMethods::_e9cf81a88e0192cb33260d269f9e9c90_262);
#line 272
static void _hicc_test_272() { size_t (Self::* _272)(const T&) = &Self::erase; (void)_272; }
#line 272
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((size_t (Self::*)(const T&))&Self::erase));
#line 275
static void _hicc_test_275() { const_iterator (Self::* _275)() const = &Self::begin; (void)_275; }
#line 275
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((const_iterator (Self::*)() const)&Self::begin));
#line 277
static void _hicc_test_277() { const_iterator (Self::* _277)() const = &Self::end; (void)_277; }
#line 277
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((const_iterator (Self::*)() const)&Self::end));
#line 172
};
#line 285
template <class T, class Hash, class Pred, class Allocator> struct CppUnorderedMultiSetIter_285 {
#line 285
typedef typename std::unordered_multiset<T, Hash, Pred, Allocator>::const_iterator Self; typedef std::unordered_multiset<T, Hash, Pred, Allocator> SelfContainer; typedef CppUnorderedMultiSetIter_285<T,Hash,Pred,Allocator> SelfMethods;
#line 288
static const T& next(Self& self) {
                return *self++;
            }
#line 292
static void _hicc_test_292() { const T& (* _292)(Self&) = &SelfMethods::next; (void)_292; }
#line 292
EXPORT_METHOD_IN(SelfContainer, Self, ((const T& (*)(Self&))&SelfMethods::next));
#line 294
static void _hicc_test_294() { bool (* _294)(const Self&, const Self&) = &hicc::make_eq<Self, Self>; (void)_294; }
#line 294
EXPORT_METHOD_IN(SelfContainer, Self, ((bool (*)(const Self&, const Self&))&hicc::make_eq<Self, Self>));
#line 285
};

#endif
