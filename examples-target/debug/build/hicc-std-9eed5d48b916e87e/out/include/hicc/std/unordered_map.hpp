#ifndef __SRC_STD_UNORDERED_MAP_RS
#define __SRC_STD_UNORDERED_MAP_RS

#include <hicc/hicc.hpp>
#line 0 R"(./src/std_unordered_map.rs)"
#line 6
#include <unordered_map>
#line 10
template <class K, class V, class Hash, class Pred, class Allocator> struct unordered_map_10;
#line 10
namespace hicc { template <class K, class V, class Hash, class Pred, class Allocator> struct MethodsType<std::unordered_map<K, V, Hash, Pred, Allocator>, void> { typedef unordered_map_10<K,V,Hash,Pred,Allocator> methods_type; }; }
#line 140
template <class K, class V, class Hash, class Pred, class Allocator> struct CppUnorderedMapIter_140;
#line 140
namespace hicc { template <class K, class V, class Hash, class Pred, class Allocator> struct MethodsType<typename std::unordered_map<K, V, Hash, Pred, Allocator>::const_iterator, std::unordered_map<K, V, Hash, Pred, Allocator>> { typedef CppUnorderedMapIter_140<K,V,Hash,Pred,Allocator> methods_type; }; }
#line 163
template <class K, class V, class Hash, class Pred, class Allocator> struct CppUnorderedMapIterMut_163;
#line 163
namespace hicc { template <class K, class V, class Hash, class Pred, class Allocator> struct MethodsType<typename std::unordered_map<K, V, Hash, Pred, Allocator>::iterator, std::unordered_map<K, V, Hash, Pred, Allocator>> { typedef CppUnorderedMapIterMut_163<K,V,Hash,Pred,Allocator> methods_type; }; }
#line 10
template <class K, class V, class Hash, class Pred, class Allocator> struct unordered_map_10 {
#line 10
typedef std::unordered_map<K, V, Hash, Pred, Allocator> Self; typedef void SelfContainer; typedef unordered_map_10<K,V,Hash,Pred,Allocator> SelfMethods;
#line 13
typedef typename Self::iterator iterator;
            typedef typename Self::const_iterator const_iterator;
#line 74
static bool contains(const Self& self, const K& key) {
                return self.find(key) != self.end();
            }
#line 99
static bool insert(Self& self, const K& key, const V& val) {
                return self.insert(std::make_pair(key, val)).second;
            }
#line 21
static void _hicc_test_21() { bool (Self::* _21)() const = &Self::empty; (void)_21; }
#line 21
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((bool (Self::*)() const)&Self::empty));
#line 29
static void _hicc_test_29() { size_t (Self::* _29)() const = &Self::size; (void)_29; }
#line 29
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((size_t (Self::*)() const)&Self::size));
#line 37
static void _hicc_test_37() { size_t (Self::* _37)() const = &Self::max_size; (void)_37; }
#line 37
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((size_t (Self::*)() const)&Self::max_size));
#line 48
static void _hicc_test_48() { void (Self::* _48)() = &Self::clear; (void)_48; }
#line 48
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((void (Self::*)())&Self::clear));
#line 60
static void _hicc_test_60() { void (Self::* _60)(Self&) = &Self::swap; (void)_60; }
#line 60
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((void (Self::*)(Self&))&Self::swap));
#line 70
static void _hicc_test_70() { size_t (Self::* _70)(const K&) const = &Self::count; (void)_70; }
#line 70
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((size_t (Self::*)(const K&) const)&Self::count));
#line 85
static void _hicc_test_85() { bool (* _85)(const Self&, const K&) = &SelfMethods::contains; (void)_85; }
#line 85
EXPORT_METHOD_IN(SelfContainer, Self, ((bool (*)(const Self&, const K&))&SelfMethods::contains));
#line 95
static void _hicc_test_95() { void (* _95)(Self&, const Self&) = &hicc::make_assign<Self, Self>; (void)_95; }
#line 95
EXPORT_METHOD_IN(SelfContainer, Self, ((void (*)(Self&, const Self&))&hicc::make_assign<Self, Self>));
#line 110
static void _hicc_test_110() { bool (* _110)(Self&, const K&, const V&) = &SelfMethods::insert; (void)_110; }
#line 110
EXPORT_METHOD_IN(SelfContainer, Self, ((bool (*)(Self&, const K&, const V&))&SelfMethods::insert));
#line 120
static void _hicc_test_120() { size_t (Self::* _120)(const K&) = &Self::erase; (void)_120; }
#line 120
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((size_t (Self::*)(const K&))&Self::erase));
#line 123
static void _hicc_test_123() { const_iterator (Self::* _123)(const K&) const = &Self::find; (void)_123; }
#line 123
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((const_iterator (Self::*)(const K&) const)&Self::find));
#line 125
static void _hicc_test_125() { iterator (Self::* _125)(const K&) = &Self::find; (void)_125; }
#line 125
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((iterator (Self::*)(const K&))&Self::find));
#line 127
static void _hicc_test_127() { const_iterator (Self::* _127)() const = &Self::begin; (void)_127; }
#line 127
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((const_iterator (Self::*)() const)&Self::begin));
#line 129
static void _hicc_test_129() { iterator (Self::* _129)() = &Self::begin; (void)_129; }
#line 129
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((iterator (Self::*)())&Self::begin));
#line 131
static void _hicc_test_131() { const_iterator (Self::* _131)() const = &Self::end; (void)_131; }
#line 131
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((const_iterator (Self::*)() const)&Self::end));
#line 133
static void _hicc_test_133() { iterator (Self::* _133)() = &Self::end; (void)_133; }
#line 133
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((iterator (Self::*)())&Self::end));
#line 10
};
#line 140
template <class K, class V, class Hash, class Pred, class Allocator> struct CppUnorderedMapIter_140 {
#line 140
typedef typename std::unordered_map<K, V, Hash, Pred, Allocator>::const_iterator Self; typedef std::unordered_map<K, V, Hash, Pred, Allocator> SelfContainer; typedef CppUnorderedMapIter_140<K,V,Hash,Pred,Allocator> SelfMethods;
#line 143
static void next(Self& self) {
                ++self;
            }
            static const K& key(const Self& self) {
                return self->first;
            }
            static const V& value(const Self& self) {
                return self->second;
            }
#line 153
static void _hicc_test_153() { void (* _153)(Self&) = &SelfMethods::next; (void)_153; }
#line 153
EXPORT_METHOD_IN(SelfContainer, Self, ((void (*)(Self&))&SelfMethods::next));
#line 155
static void _hicc_test_155() { const K& (* _155)(const Self&) = &SelfMethods::key; (void)_155; }
#line 155
EXPORT_METHOD_IN(SelfContainer, Self, ((const K& (*)(const Self&))&SelfMethods::key));
#line 157
static void _hicc_test_157() { const V& (* _157)(const Self&) = &SelfMethods::value; (void)_157; }
#line 157
EXPORT_METHOD_IN(SelfContainer, Self, ((const V& (*)(const Self&))&SelfMethods::value));
#line 159
static void _hicc_test_159() { bool (* _159)(const Self&, const Self&) = &hicc::make_eq<Self, Self>; (void)_159; }
#line 159
EXPORT_METHOD_IN(SelfContainer, Self, ((bool (*)(const Self&, const Self&))&hicc::make_eq<Self, Self>));
#line 140
};
#line 163
template <class K, class V, class Hash, class Pred, class Allocator> struct CppUnorderedMapIterMut_163 {
#line 163
typedef typename std::unordered_map<K, V, Hash, Pred, Allocator>::iterator Self; typedef std::unordered_map<K, V, Hash, Pred, Allocator> SelfContainer; typedef CppUnorderedMapIterMut_163<K,V,Hash,Pred,Allocator> SelfMethods;
#line 166
static void next(Self& self) {
                ++self;
            }
            static const K& key(const Self& self) {
                return self->first;
            }
            static V& value(Self& self) {
                return self->second;
            }
#line 176
static void _hicc_test_176() { void (* _176)(Self&) = &SelfMethods::next; (void)_176; }
#line 176
EXPORT_METHOD_IN(SelfContainer, Self, ((void (*)(Self&))&SelfMethods::next));
#line 178
static void _hicc_test_178() { const K& (* _178)(const Self&) = &SelfMethods::key; (void)_178; }
#line 178
EXPORT_METHOD_IN(SelfContainer, Self, ((const K& (*)(const Self&))&SelfMethods::key));
#line 180
static void _hicc_test_180() { V& (* _180)(Self&) = &SelfMethods::value; (void)_180; }
#line 180
EXPORT_METHOD_IN(SelfContainer, Self, ((V& (*)(Self&))&SelfMethods::value));
#line 182
static void _hicc_test_182() { bool (* _182)(const Self&, const Self&) = &hicc::make_eq<Self, Self>; (void)_182; }
#line 182
EXPORT_METHOD_IN(SelfContainer, Self, ((bool (*)(const Self&, const Self&))&hicc::make_eq<Self, Self>));
#line 163
};
#line 307
template <class K, class V, class Hash, class Pred, class Allocator> struct unordered_multimap_307;
#line 307
namespace hicc { template <class K, class V, class Hash, class Pred, class Allocator> struct MethodsType<std::unordered_multimap<K, V, Hash, Pred, Allocator>, void> { typedef unordered_multimap_307<K,V,Hash,Pred,Allocator> methods_type; }; }
#line 439
template <class K, class V, class Hash, class Pred, class Allocator> struct CppUnorderedMultiMapIter_439;
#line 439
namespace hicc { template <class K, class V, class Hash, class Pred, class Allocator> struct MethodsType<typename std::unordered_multimap<K, V, Hash, Pred, Allocator>::const_iterator, std::unordered_multimap<K, V, Hash, Pred, Allocator>> { typedef CppUnorderedMultiMapIter_439<K,V,Hash,Pred,Allocator> methods_type; }; }
#line 462
template <class K, class V, class Hash, class Pred, class Allocator> struct CppUnorderedMultiMapIterMut_462;
#line 462
namespace hicc { template <class K, class V, class Hash, class Pred, class Allocator> struct MethodsType<typename std::unordered_multimap<K, V, Hash, Pred, Allocator>::iterator, std::unordered_multimap<K, V, Hash, Pred, Allocator>> { typedef CppUnorderedMultiMapIterMut_462<K,V,Hash,Pred,Allocator> methods_type; }; }
#line 307
template <class K, class V, class Hash, class Pred, class Allocator> struct unordered_multimap_307 {
#line 307
typedef std::unordered_multimap<K, V, Hash, Pred, Allocator> Self; typedef void SelfContainer; typedef unordered_multimap_307<K,V,Hash,Pred,Allocator> SelfMethods;
#line 310
typedef typename Self::iterator iterator;
            typedef typename Self::const_iterator const_iterator;
#line 372
static bool contains(const Self& self, const K& key) {
                return self.find(key) != self.end();
            }
#line 398
static void insert(Self& self, const K& key, const V& val) {
                self.insert(std::make_pair(key, val));
            }
#line 318
static void _hicc_test_318() { bool (Self::* _318)() const = &Self::empty; (void)_318; }
#line 318
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((bool (Self::*)() const)&Self::empty));
#line 326
static void _hicc_test_326() { size_t (Self::* _326)() const = &Self::size; (void)_326; }
#line 326
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((size_t (Self::*)() const)&Self::size));
#line 334
static void _hicc_test_334() { size_t (Self::* _334)() const = &Self::max_size; (void)_334; }
#line 334
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((size_t (Self::*)() const)&Self::max_size));
#line 345
static void _hicc_test_345() { void (Self::* _345)() = &Self::clear; (void)_345; }
#line 345
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((void (Self::*)())&Self::clear));
#line 357
static void _hicc_test_357() { void (Self::* _357)(Self&) = &Self::swap; (void)_357; }
#line 357
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((void (Self::*)(Self&))&Self::swap));
#line 368
static void _hicc_test_368() { size_t (Self::* _368)(const K&) const = &Self::count; (void)_368; }
#line 368
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((size_t (Self::*)(const K&) const)&Self::count));
#line 383
static void _hicc_test_383() { bool (* _383)(const Self&, const K&) = &SelfMethods::contains; (void)_383; }
#line 383
EXPORT_METHOD_IN(SelfContainer, Self, ((bool (*)(const Self&, const K&))&SelfMethods::contains));
#line 394
static void _hicc_test_394() { void (* _394)(Self&, const Self&) = &hicc::make_assign<Self, Self>; (void)_394; }
#line 394
EXPORT_METHOD_IN(SelfContainer, Self, ((void (*)(Self&, const Self&))&hicc::make_assign<Self, Self>));
#line 409
static void _hicc_test_409() { void (* _409)(Self&, const K&, const V&) = &SelfMethods::insert; (void)_409; }
#line 409
EXPORT_METHOD_IN(SelfContainer, Self, ((void (*)(Self&, const K&, const V&))&SelfMethods::insert));
#line 419
static void _hicc_test_419() { size_t (Self::* _419)(const K&) = &Self::erase; (void)_419; }
#line 419
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((size_t (Self::*)(const K&))&Self::erase));
#line 422
static void _hicc_test_422() { const_iterator (Self::* _422)(const K&) const = &Self::find; (void)_422; }
#line 422
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((const_iterator (Self::*)(const K&) const)&Self::find));
#line 424
static void _hicc_test_424() { iterator (Self::* _424)(const K&) = &Self::find; (void)_424; }
#line 424
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((iterator (Self::*)(const K&))&Self::find));
#line 426
static void _hicc_test_426() { const_iterator (Self::* _426)() const = &Self::begin; (void)_426; }
#line 426
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((const_iterator (Self::*)() const)&Self::begin));
#line 428
static void _hicc_test_428() { iterator (Self::* _428)() = &Self::begin; (void)_428; }
#line 428
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((iterator (Self::*)())&Self::begin));
#line 430
static void _hicc_test_430() { const_iterator (Self::* _430)() const = &Self::end; (void)_430; }
#line 430
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((const_iterator (Self::*)() const)&Self::end));
#line 432
static void _hicc_test_432() { iterator (Self::* _432)() = &Self::end; (void)_432; }
#line 432
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((iterator (Self::*)())&Self::end));
#line 307
};
#line 439
template <class K, class V, class Hash, class Pred, class Allocator> struct CppUnorderedMultiMapIter_439 {
#line 439
typedef typename std::unordered_multimap<K, V, Hash, Pred, Allocator>::const_iterator Self; typedef std::unordered_multimap<K, V, Hash, Pred, Allocator> SelfContainer; typedef CppUnorderedMultiMapIter_439<K,V,Hash,Pred,Allocator> SelfMethods;
#line 442
static void next(Self& self) {
                ++self;
            }
            static const K& key(const Self& self) {
                return self->first;
            }
            static const V& value(const Self& self) {
                return self->second;
            }
#line 452
static void _hicc_test_452() { void (* _452)(Self&) = &SelfMethods::next; (void)_452; }
#line 452
EXPORT_METHOD_IN(SelfContainer, Self, ((void (*)(Self&))&SelfMethods::next));
#line 454
static void _hicc_test_454() { const K& (* _454)(const Self&) = &SelfMethods::key; (void)_454; }
#line 454
EXPORT_METHOD_IN(SelfContainer, Self, ((const K& (*)(const Self&))&SelfMethods::key));
#line 456
static void _hicc_test_456() { const V& (* _456)(const Self&) = &SelfMethods::value; (void)_456; }
#line 456
EXPORT_METHOD_IN(SelfContainer, Self, ((const V& (*)(const Self&))&SelfMethods::value));
#line 458
static void _hicc_test_458() { bool (* _458)(const Self&, const Self&) = &hicc::make_eq<Self, Self>; (void)_458; }
#line 458
EXPORT_METHOD_IN(SelfContainer, Self, ((bool (*)(const Self&, const Self&))&hicc::make_eq<Self, Self>));
#line 439
};
#line 462
template <class K, class V, class Hash, class Pred, class Allocator> struct CppUnorderedMultiMapIterMut_462 {
#line 462
typedef typename std::unordered_multimap<K, V, Hash, Pred, Allocator>::iterator Self; typedef std::unordered_multimap<K, V, Hash, Pred, Allocator> SelfContainer; typedef CppUnorderedMultiMapIterMut_462<K,V,Hash,Pred,Allocator> SelfMethods;
#line 465
static void next(Self& self) {
                ++self;
            }
            static const K& key(const Self& self) {
                return self->first;
            }
            static V& value(Self& self) {
                return self->second;
            }
#line 475
static void _hicc_test_475() { void (* _475)(Self&) = &SelfMethods::next; (void)_475; }
#line 475
EXPORT_METHOD_IN(SelfContainer, Self, ((void (*)(Self&))&SelfMethods::next));
#line 477
static void _hicc_test_477() { const K& (* _477)(const Self&) = &SelfMethods::key; (void)_477; }
#line 477
EXPORT_METHOD_IN(SelfContainer, Self, ((const K& (*)(const Self&))&SelfMethods::key));
#line 479
static void _hicc_test_479() { V& (* _479)(Self&) = &SelfMethods::value; (void)_479; }
#line 479
EXPORT_METHOD_IN(SelfContainer, Self, ((V& (*)(Self&))&SelfMethods::value));
#line 481
static void _hicc_test_481() { bool (* _481)(const Self&, const Self&) = &hicc::make_eq<Self, Self>; (void)_481; }
#line 481
EXPORT_METHOD_IN(SelfContainer, Self, ((bool (*)(const Self&, const Self&))&hicc::make_eq<Self, Self>));
#line 462
};

#endif
