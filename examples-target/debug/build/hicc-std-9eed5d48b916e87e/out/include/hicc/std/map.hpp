#ifndef __SRC_STD_MAP_RS
#define __SRC_STD_MAP_RS

#include <hicc/hicc.hpp>
#line 0 R"(./src/std_map.rs)"
#line 6
#include <map>
#line 10
template<class K, class V, class Compare, class Allocator> struct map_10;
#line 10
namespace hicc { template<class K, class V, class Compare, class Allocator> struct MethodsType<std::map<K, V, Compare, Allocator>, void> { typedef map_10<K,V,Compare,Allocator> methods_type; }; }
#line 159
template<class K, class V, class Compare, class Allocator> struct CppMapIter_159;
#line 159
namespace hicc { template<class K, class V, class Compare, class Allocator> struct MethodsType<typename std::map<K, V, Compare, Allocator>::const_iterator, std::map<K, V, Compare, Allocator>> { typedef CppMapIter_159<K,V,Compare,Allocator> methods_type; }; }
#line 185
template<class K, class V, class Compare, class Allocator> struct CppMapIterMut_185;
#line 185
namespace hicc { template<class K, class V, class Compare, class Allocator> struct MethodsType<typename std::map<K, V, Compare, Allocator>::iterator, std::map<K, V, Compare, Allocator>> { typedef CppMapIterMut_185<K,V,Compare,Allocator> methods_type; }; }
#line 211
template<class K, class V, class Compare, class Allocator> struct CppMapRevIter_211;
#line 211
namespace hicc { template<class K, class V, class Compare, class Allocator> struct MethodsType<typename std::map<K, V, Compare, Allocator>::const_reverse_iterator, std::map<K, V, Compare, Allocator>> { typedef CppMapRevIter_211<K,V,Compare,Allocator> methods_type; }; }
#line 234
template<class K, class V, class Compare, class Allocator> struct CppMapRevIterMut_234;
#line 234
namespace hicc { template<class K, class V, class Compare, class Allocator> struct MethodsType<typename std::map<K, V, Compare, Allocator>::reverse_iterator, std::map<K, V, Compare, Allocator>> { typedef CppMapRevIterMut_234<K,V,Compare,Allocator> methods_type; }; }
#line 10
template<class K, class V, class Compare, class Allocator> struct map_10 {
#line 10
typedef std::map<K, V, Compare, Allocator> Self; typedef void SelfContainer; typedef map_10<K,V,Compare,Allocator> SelfMethods;
#line 13
typedef typename Self::iterator iterator;
            typedef typename Self::reverse_iterator reverse_iterator;
            typedef typename Self::const_iterator const_iterator;
            typedef typename Self::const_reverse_iterator const_reverse_iterator;
#line 76
static bool contains(const Self& self, const K& key) {
                return self.find(key) != self.end();
            }
#line 101
static bool insert(Self& self, const K& key, const V& val) {
                return self.insert(std::make_pair(key, val)).second;
            }
#line 23
static void _hicc_test_23() { bool (Self::* _23)() const = &Self::empty; (void)_23; }
#line 23
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((bool (Self::*)() const)&Self::empty));
#line 31
static void _hicc_test_31() { size_t (Self::* _31)() const = &Self::size; (void)_31; }
#line 31
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((size_t (Self::*)() const)&Self::size));
#line 39
static void _hicc_test_39() { size_t (Self::* _39)() const = &Self::max_size; (void)_39; }
#line 39
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((size_t (Self::*)() const)&Self::max_size));
#line 50
static void _hicc_test_50() { void (Self::* _50)() = &Self::clear; (void)_50; }
#line 50
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((void (Self::*)())&Self::clear));
#line 62
static void _hicc_test_62() { void (Self::* _62)(Self&) = &Self::swap; (void)_62; }
#line 62
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((void (Self::*)(Self&))&Self::swap));
#line 72
static void _hicc_test_72() { size_t (Self::* _72)(const K&) const = &Self::count; (void)_72; }
#line 72
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((size_t (Self::*)(const K&) const)&Self::count));
#line 87
static void _hicc_test_87() { bool (* _87)(const Self&, const K&) = &SelfMethods::contains; (void)_87; }
#line 87
EXPORT_METHOD_IN(SelfContainer, Self, ((bool (*)(const Self&, const K&))&SelfMethods::contains));
#line 97
static void _hicc_test_97() { void (* _97)(Self&, const Self&) = &hicc::make_assign<Self, Self>; (void)_97; }
#line 97
EXPORT_METHOD_IN(SelfContainer, Self, ((void (*)(Self&, const Self&))&hicc::make_assign<Self, Self>));
#line 112
static void _hicc_test_112() { bool (* _112)(Self&, const K&, const V&) = &SelfMethods::insert; (void)_112; }
#line 112
EXPORT_METHOD_IN(SelfContainer, Self, ((bool (*)(Self&, const K&, const V&))&SelfMethods::insert));
#line 122
static void _hicc_test_122() { size_t (Self::* _122)(const K&) = &Self::erase; (void)_122; }
#line 122
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((size_t (Self::*)(const K&))&Self::erase));
#line 125
static void _hicc_test_125() { const_iterator (Self::* _125)(const K&) const = &Self::find; (void)_125; }
#line 125
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((const_iterator (Self::*)(const K&) const)&Self::find));
#line 127
static void _hicc_test_127() { iterator (Self::* _127)(const K&) = &Self::find; (void)_127; }
#line 127
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((iterator (Self::*)(const K&))&Self::find));
#line 129
static void _hicc_test_129() { const_iterator (Self::* _129)(const K&) const = &Self::lower_bound; (void)_129; }
#line 129
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((const_iterator (Self::*)(const K&) const)&Self::lower_bound));
#line 131
static void _hicc_test_131() { iterator (Self::* _131)(const K&) = &Self::lower_bound; (void)_131; }
#line 131
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((iterator (Self::*)(const K&))&Self::lower_bound));
#line 133
static void _hicc_test_133() { const_iterator (Self::* _133)(const K&) const = &Self::upper_bound; (void)_133; }
#line 133
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((const_iterator (Self::*)(const K&) const)&Self::upper_bound));
#line 135
static void _hicc_test_135() { iterator (Self::* _135)(const K&) = &Self::upper_bound; (void)_135; }
#line 135
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((iterator (Self::*)(const K&))&Self::upper_bound));
#line 138
static void _hicc_test_138() { const_iterator (Self::* _138)() const = &Self::begin; (void)_138; }
#line 138
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((const_iterator (Self::*)() const)&Self::begin));
#line 140
static void _hicc_test_140() { iterator (Self::* _140)() = &Self::begin; (void)_140; }
#line 140
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((iterator (Self::*)())&Self::begin));
#line 142
static void _hicc_test_142() { const_iterator (Self::* _142)() const = &Self::end; (void)_142; }
#line 142
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((const_iterator (Self::*)() const)&Self::end));
#line 144
static void _hicc_test_144() { iterator (Self::* _144)() = &Self::end; (void)_144; }
#line 144
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((iterator (Self::*)())&Self::end));
#line 146
static void _hicc_test_146() { const_reverse_iterator (Self::* _146)() const = &Self::rbegin; (void)_146; }
#line 146
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((const_reverse_iterator (Self::*)() const)&Self::rbegin));
#line 148
static void _hicc_test_148() { reverse_iterator (Self::* _148)() = &Self::rbegin; (void)_148; }
#line 148
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((reverse_iterator (Self::*)())&Self::rbegin));
#line 150
static void _hicc_test_150() { const_reverse_iterator (Self::* _150)() const = &Self::rend; (void)_150; }
#line 150
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((const_reverse_iterator (Self::*)() const)&Self::rend));
#line 152
static void _hicc_test_152() { reverse_iterator (Self::* _152)() = &Self::rend; (void)_152; }
#line 152
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((reverse_iterator (Self::*)())&Self::rend));
#line 10
};
#line 159
template<class K, class V, class Compare, class Allocator> struct CppMapIter_159 {
#line 159
typedef typename std::map<K, V, Compare, Allocator>::const_iterator Self; typedef std::map<K, V, Compare, Allocator> SelfContainer; typedef CppMapIter_159<K,V,Compare,Allocator> SelfMethods;
#line 162
typedef typename SelfContainer::const_reverse_iterator const_reverse_iterator;
            static void next(Self& self) {
                ++self;
            }
            static const K& key(const Self& self) {
                return self->first;
            }
            static const V& value(const Self& self) {
                return self->second;
            }
#line 173
static void _hicc_test_173() { void (* _173)(Self&) = &SelfMethods::next; (void)_173; }
#line 173
EXPORT_METHOD_IN(SelfContainer, Self, ((void (*)(Self&))&SelfMethods::next));
#line 175
static void _hicc_test_175() { const K& (* _175)(const Self&) = &SelfMethods::key; (void)_175; }
#line 175
EXPORT_METHOD_IN(SelfContainer, Self, ((const K& (*)(const Self&))&SelfMethods::key));
#line 177
static void _hicc_test_177() { const V& (* _177)(const Self&) = &SelfMethods::value; (void)_177; }
#line 177
EXPORT_METHOD_IN(SelfContainer, Self, ((const V& (*)(const Self&))&SelfMethods::value));
#line 179
static void _hicc_test_179() { bool (* _179)(const Self&, const Self&) = &hicc::make_eq<Self, Self>; (void)_179; }
#line 179
EXPORT_METHOD_IN(SelfContainer, Self, ((bool (*)(const Self&, const Self&))&hicc::make_eq<Self, Self>));
#line 181
static void _hicc_test_181() { const_reverse_iterator (* _181)(Self&&) = &hicc::make_constructor<const_reverse_iterator, Self>; (void)_181; }
#line 181
EXPORT_METHOD_IN(SelfContainer, Self, ((const_reverse_iterator (*)(Self&&))&hicc::make_constructor<const_reverse_iterator, Self>));
#line 159
};
#line 185
template<class K, class V, class Compare, class Allocator> struct CppMapIterMut_185 {
#line 185
typedef typename std::map<K, V, Compare, Allocator>::iterator Self; typedef std::map<K, V, Compare, Allocator> SelfContainer; typedef CppMapIterMut_185<K,V,Compare,Allocator> SelfMethods;
#line 188
typedef typename SelfContainer::reverse_iterator reverse_iterator;
            static void next(Self& self) {
                ++self;
            }
            static const K& key(const Self& self) {
                return self->first;
            }
            static V& value(Self& self) {
                return self->second;
            }
#line 199
static void _hicc_test_199() { void (* _199)(Self&) = &SelfMethods::next; (void)_199; }
#line 199
EXPORT_METHOD_IN(SelfContainer, Self, ((void (*)(Self&))&SelfMethods::next));
#line 201
static void _hicc_test_201() { const K& (* _201)(const Self&) = &SelfMethods::key; (void)_201; }
#line 201
EXPORT_METHOD_IN(SelfContainer, Self, ((const K& (*)(const Self&))&SelfMethods::key));
#line 203
static void _hicc_test_203() { V& (* _203)(Self&) = &SelfMethods::value; (void)_203; }
#line 203
EXPORT_METHOD_IN(SelfContainer, Self, ((V& (*)(Self&))&SelfMethods::value));
#line 205
static void _hicc_test_205() { bool (* _205)(const Self&, const Self&) = &hicc::make_eq<Self, Self>; (void)_205; }
#line 205
EXPORT_METHOD_IN(SelfContainer, Self, ((bool (*)(const Self&, const Self&))&hicc::make_eq<Self, Self>));
#line 207
static void _hicc_test_207() { reverse_iterator (* _207)(Self&&) = &hicc::make_constructor<reverse_iterator, Self>; (void)_207; }
#line 207
EXPORT_METHOD_IN(SelfContainer, Self, ((reverse_iterator (*)(Self&&))&hicc::make_constructor<reverse_iterator, Self>));
#line 185
};
#line 211
template<class K, class V, class Compare, class Allocator> struct CppMapRevIter_211 {
#line 211
typedef typename std::map<K, V, Compare, Allocator>::const_reverse_iterator Self; typedef std::map<K, V, Compare, Allocator> SelfContainer; typedef CppMapRevIter_211<K,V,Compare,Allocator> SelfMethods;
#line 214
static void next(Self& self) {
                ++self;
            }
            static const K& key(const Self& self) {
                return self->first;
            }
            static const V& value(const Self& self) {
                return self->second;
            }
#line 224
static void _hicc_test_224() { void (* _224)(Self&) = &SelfMethods::next; (void)_224; }
#line 224
EXPORT_METHOD_IN(SelfContainer, Self, ((void (*)(Self&))&SelfMethods::next));
#line 226
static void _hicc_test_226() { const K& (* _226)(const Self&) = &SelfMethods::key; (void)_226; }
#line 226
EXPORT_METHOD_IN(SelfContainer, Self, ((const K& (*)(const Self&))&SelfMethods::key));
#line 228
static void _hicc_test_228() { const V& (* _228)(const Self&) = &SelfMethods::value; (void)_228; }
#line 228
EXPORT_METHOD_IN(SelfContainer, Self, ((const V& (*)(const Self&))&SelfMethods::value));
#line 230
static void _hicc_test_230() { bool (* _230)(const Self&, const Self&) = &hicc::make_eq<Self, Self>; (void)_230; }
#line 230
EXPORT_METHOD_IN(SelfContainer, Self, ((bool (*)(const Self&, const Self&))&hicc::make_eq<Self, Self>));
#line 211
};
#line 234
template<class K, class V, class Compare, class Allocator> struct CppMapRevIterMut_234 {
#line 234
typedef typename std::map<K, V, Compare, Allocator>::reverse_iterator Self; typedef std::map<K, V, Compare, Allocator> SelfContainer; typedef CppMapRevIterMut_234<K,V,Compare,Allocator> SelfMethods;
#line 237
static void next(Self& self) {
                ++self;
            }
            static const K& key(const Self& self) {
                return self->first;
            }
            static V& value(Self& self) {
                return self->second;
            }
#line 247
static void _hicc_test_247() { void (* _247)(Self&) = &SelfMethods::next; (void)_247; }
#line 247
EXPORT_METHOD_IN(SelfContainer, Self, ((void (*)(Self&))&SelfMethods::next));
#line 249
static void _hicc_test_249() { const K& (* _249)(const Self&) = &SelfMethods::key; (void)_249; }
#line 249
EXPORT_METHOD_IN(SelfContainer, Self, ((const K& (*)(const Self&))&SelfMethods::key));
#line 251
static void _hicc_test_251() { V& (* _251)(Self&) = &SelfMethods::value; (void)_251; }
#line 251
EXPORT_METHOD_IN(SelfContainer, Self, ((V& (*)(Self&))&SelfMethods::value));
#line 253
static void _hicc_test_253() { bool (* _253)(const Self&, const Self&) = &hicc::make_eq<Self, Self>; (void)_253; }
#line 253
EXPORT_METHOD_IN(SelfContainer, Self, ((bool (*)(const Self&, const Self&))&hicc::make_eq<Self, Self>));
#line 234
};
#line 597
template<class K, class V, class Compare, class Allocator> struct multimap_597;
#line 597
namespace hicc { template<class K, class V, class Compare, class Allocator> struct MethodsType<std::multimap<K, V, Compare, Allocator>, void> { typedef multimap_597<K,V,Compare,Allocator> methods_type; }; }
#line 746
template<class K, class V, class Compare, class Allocator> struct CppMultiMapIter_746;
#line 746
namespace hicc { template<class K, class V, class Compare, class Allocator> struct MethodsType<typename std::multimap<K, V, Compare, Allocator>::const_iterator, std::multimap<K, V, Compare, Allocator>> { typedef CppMultiMapIter_746<K,V,Compare,Allocator> methods_type; }; }
#line 772
template<class K, class V, class Compare, class Allocator> struct CppMultiMapIterMut_772;
#line 772
namespace hicc { template<class K, class V, class Compare, class Allocator> struct MethodsType<typename std::multimap<K, V, Compare, Allocator>::iterator, std::multimap<K, V, Compare, Allocator>> { typedef CppMultiMapIterMut_772<K,V,Compare,Allocator> methods_type; }; }
#line 798
template<class K, class V, class Compare, class Allocator> struct CppMultiMapRevIter_798;
#line 798
namespace hicc { template<class K, class V, class Compare, class Allocator> struct MethodsType<typename std::multimap<K, V, Compare, Allocator>::const_reverse_iterator, std::multimap<K, V, Compare, Allocator>> { typedef CppMultiMapRevIter_798<K,V,Compare,Allocator> methods_type; }; }
#line 821
template<class K, class V, class Compare, class Allocator> struct CppMultiMapRevIterMut_821;
#line 821
namespace hicc { template<class K, class V, class Compare, class Allocator> struct MethodsType<typename std::multimap<K, V, Compare, Allocator>::reverse_iterator, std::multimap<K, V, Compare, Allocator>> { typedef CppMultiMapRevIterMut_821<K,V,Compare,Allocator> methods_type; }; }
#line 597
template<class K, class V, class Compare, class Allocator> struct multimap_597 {
#line 597
typedef std::multimap<K, V, Compare, Allocator> Self; typedef void SelfContainer; typedef multimap_597<K,V,Compare,Allocator> SelfMethods;
#line 600
typedef typename Self::iterator iterator;
            typedef typename Self::const_iterator const_iterator;
            typedef typename Self::reverse_iterator reverse_iterator;
            typedef typename Self::const_reverse_iterator const_reverse_iterator;
#line 664
static bool contains(const Self& self, const K& key) {
                return self.find(key) != self.end();
            }
#line 689
static void insert(Self& self, const K& key, const V& val) {
                self.insert(std::make_pair(key, val));
            }
#line 610
static void _hicc_test_610() { bool (Self::* _610)() const = &Self::empty; (void)_610; }
#line 610
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((bool (Self::*)() const)&Self::empty));
#line 618
static void _hicc_test_618() { size_t (Self::* _618)() const = &Self::size; (void)_618; }
#line 618
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((size_t (Self::*)() const)&Self::size));
#line 626
static void _hicc_test_626() { size_t (Self::* _626)() const = &Self::max_size; (void)_626; }
#line 626
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((size_t (Self::*)() const)&Self::max_size));
#line 637
static void _hicc_test_637() { void (Self::* _637)() = &Self::clear; (void)_637; }
#line 637
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((void (Self::*)())&Self::clear));
#line 649
static void _hicc_test_649() { void (Self::* _649)(Self&) = &Self::swap; (void)_649; }
#line 649
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((void (Self::*)(Self&))&Self::swap));
#line 660
static void _hicc_test_660() { size_t (Self::* _660)(const K&) const = &Self::count; (void)_660; }
#line 660
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((size_t (Self::*)(const K&) const)&Self::count));
#line 675
static void _hicc_test_675() { bool (* _675)(const Self&, const K&) = &SelfMethods::contains; (void)_675; }
#line 675
EXPORT_METHOD_IN(SelfContainer, Self, ((bool (*)(const Self&, const K&))&SelfMethods::contains));
#line 685
static void _hicc_test_685() { void (* _685)(Self&, const Self&) = &hicc::make_assign<Self, Self>; (void)_685; }
#line 685
EXPORT_METHOD_IN(SelfContainer, Self, ((void (*)(Self&, const Self&))&hicc::make_assign<Self, Self>));
#line 700
static void _hicc_test_700() { void (* _700)(Self&, const K&, const V&) = &SelfMethods::insert; (void)_700; }
#line 700
EXPORT_METHOD_IN(SelfContainer, Self, ((void (*)(Self&, const K&, const V&))&SelfMethods::insert));
#line 710
static void _hicc_test_710() { size_t (Self::* _710)(const K&) = &Self::erase; (void)_710; }
#line 710
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((size_t (Self::*)(const K&))&Self::erase));
#line 713
static void _hicc_test_713() { const_iterator (Self::* _713)(const K&) const = &Self::find; (void)_713; }
#line 713
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((const_iterator (Self::*)(const K&) const)&Self::find));
#line 715
static void _hicc_test_715() { iterator (Self::* _715)(const K&) = &Self::find; (void)_715; }
#line 715
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((iterator (Self::*)(const K&))&Self::find));
#line 717
static void _hicc_test_717() { const_iterator (Self::* _717)(const K&) const = &Self::lower_bound; (void)_717; }
#line 717
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((const_iterator (Self::*)(const K&) const)&Self::lower_bound));
#line 719
static void _hicc_test_719() { iterator (Self::* _719)(const K&) = &Self::lower_bound; (void)_719; }
#line 719
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((iterator (Self::*)(const K&))&Self::lower_bound));
#line 721
static void _hicc_test_721() { const_iterator (Self::* _721)(const K&) const = &Self::upper_bound; (void)_721; }
#line 721
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((const_iterator (Self::*)(const K&) const)&Self::upper_bound));
#line 723
static void _hicc_test_723() { iterator (Self::* _723)(const K&) = &Self::upper_bound; (void)_723; }
#line 723
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((iterator (Self::*)(const K&))&Self::upper_bound));
#line 725
static void _hicc_test_725() { const_iterator (Self::* _725)() const = &Self::begin; (void)_725; }
#line 725
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((const_iterator (Self::*)() const)&Self::begin));
#line 727
static void _hicc_test_727() { iterator (Self::* _727)() = &Self::begin; (void)_727; }
#line 727
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((iterator (Self::*)())&Self::begin));
#line 729
static void _hicc_test_729() { const_iterator (Self::* _729)() const = &Self::end; (void)_729; }
#line 729
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((const_iterator (Self::*)() const)&Self::end));
#line 731
static void _hicc_test_731() { iterator (Self::* _731)() = &Self::end; (void)_731; }
#line 731
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((iterator (Self::*)())&Self::end));
#line 733
static void _hicc_test_733() { const_reverse_iterator (Self::* _733)() const = &Self::rbegin; (void)_733; }
#line 733
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((const_reverse_iterator (Self::*)() const)&Self::rbegin));
#line 735
static void _hicc_test_735() { reverse_iterator (Self::* _735)() = &Self::rbegin; (void)_735; }
#line 735
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((reverse_iterator (Self::*)())&Self::rbegin));
#line 737
static void _hicc_test_737() { const_reverse_iterator (Self::* _737)() const = &Self::rend; (void)_737; }
#line 737
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((const_reverse_iterator (Self::*)() const)&Self::rend));
#line 739
static void _hicc_test_739() { reverse_iterator (Self::* _739)() = &Self::rend; (void)_739; }
#line 739
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((reverse_iterator (Self::*)())&Self::rend));
#line 597
};
#line 746
template<class K, class V, class Compare, class Allocator> struct CppMultiMapIter_746 {
#line 746
typedef typename std::multimap<K, V, Compare, Allocator>::const_iterator Self; typedef std::multimap<K, V, Compare, Allocator> SelfContainer; typedef CppMultiMapIter_746<K,V,Compare,Allocator> SelfMethods;
#line 749
typedef typename SelfContainer::const_reverse_iterator const_reverse_iterator;
            static void next(Self& self) {
                ++self;
            }
            static const K& key(const Self& self) {
                return self->first;
            }
            static const V& value(const Self& self) {
                return self->second;
            }
#line 760
static void _hicc_test_760() { void (* _760)(Self&) = &SelfMethods::next; (void)_760; }
#line 760
EXPORT_METHOD_IN(SelfContainer, Self, ((void (*)(Self&))&SelfMethods::next));
#line 762
static void _hicc_test_762() { const K& (* _762)(const Self&) = &SelfMethods::key; (void)_762; }
#line 762
EXPORT_METHOD_IN(SelfContainer, Self, ((const K& (*)(const Self&))&SelfMethods::key));
#line 764
static void _hicc_test_764() { const V& (* _764)(const Self&) = &SelfMethods::value; (void)_764; }
#line 764
EXPORT_METHOD_IN(SelfContainer, Self, ((const V& (*)(const Self&))&SelfMethods::value));
#line 766
static void _hicc_test_766() { bool (* _766)(const Self&, const Self&) = &hicc::make_eq<Self, Self>; (void)_766; }
#line 766
EXPORT_METHOD_IN(SelfContainer, Self, ((bool (*)(const Self&, const Self&))&hicc::make_eq<Self, Self>));
#line 768
static void _hicc_test_768() { const_reverse_iterator (* _768)(Self&&) = &hicc::make_constructor<const_reverse_iterator, Self>; (void)_768; }
#line 768
EXPORT_METHOD_IN(SelfContainer, Self, ((const_reverse_iterator (*)(Self&&))&hicc::make_constructor<const_reverse_iterator, Self>));
#line 746
};
#line 772
template<class K, class V, class Compare, class Allocator> struct CppMultiMapIterMut_772 {
#line 772
typedef typename std::multimap<K, V, Compare, Allocator>::iterator Self; typedef std::multimap<K, V, Compare, Allocator> SelfContainer; typedef CppMultiMapIterMut_772<K,V,Compare,Allocator> SelfMethods;
#line 775
typedef typename SelfContainer::reverse_iterator reverse_iterator;
            static void next(Self& self) {
                ++self;
            }
            static const K& key(const Self& self) {
                return self->first;
            }
            static V& value(Self& self) {
                return self->second;
            }
#line 786
static void _hicc_test_786() { void (* _786)(Self&) = &SelfMethods::next; (void)_786; }
#line 786
EXPORT_METHOD_IN(SelfContainer, Self, ((void (*)(Self&))&SelfMethods::next));
#line 788
static void _hicc_test_788() { const K& (* _788)(const Self&) = &SelfMethods::key; (void)_788; }
#line 788
EXPORT_METHOD_IN(SelfContainer, Self, ((const K& (*)(const Self&))&SelfMethods::key));
#line 790
static void _hicc_test_790() { V& (* _790)(Self&) = &SelfMethods::value; (void)_790; }
#line 790
EXPORT_METHOD_IN(SelfContainer, Self, ((V& (*)(Self&))&SelfMethods::value));
#line 792
static void _hicc_test_792() { bool (* _792)(const Self&, const Self&) = &hicc::make_eq<Self, Self>; (void)_792; }
#line 792
EXPORT_METHOD_IN(SelfContainer, Self, ((bool (*)(const Self&, const Self&))&hicc::make_eq<Self, Self>));
#line 794
static void _hicc_test_794() { reverse_iterator (* _794)(Self&&) = &hicc::make_constructor<reverse_iterator, Self>; (void)_794; }
#line 794
EXPORT_METHOD_IN(SelfContainer, Self, ((reverse_iterator (*)(Self&&))&hicc::make_constructor<reverse_iterator, Self>));
#line 772
};
#line 798
template<class K, class V, class Compare, class Allocator> struct CppMultiMapRevIter_798 {
#line 798
typedef typename std::multimap<K, V, Compare, Allocator>::const_reverse_iterator Self; typedef std::multimap<K, V, Compare, Allocator> SelfContainer; typedef CppMultiMapRevIter_798<K,V,Compare,Allocator> SelfMethods;
#line 801
static void next(Self& self) {
                ++self;
            }
            static const K& key(const Self& self) {
                return self->first;
            }
            static const V& value(const Self& self) {
                return self->second;
            }
#line 811
static void _hicc_test_811() { void (* _811)(Self&) = &SelfMethods::next; (void)_811; }
#line 811
EXPORT_METHOD_IN(SelfContainer, Self, ((void (*)(Self&))&SelfMethods::next));
#line 813
static void _hicc_test_813() { const K& (* _813)(const Self&) = &SelfMethods::key; (void)_813; }
#line 813
EXPORT_METHOD_IN(SelfContainer, Self, ((const K& (*)(const Self&))&SelfMethods::key));
#line 815
static void _hicc_test_815() { const V& (* _815)(const Self&) = &SelfMethods::value; (void)_815; }
#line 815
EXPORT_METHOD_IN(SelfContainer, Self, ((const V& (*)(const Self&))&SelfMethods::value));
#line 817
static void _hicc_test_817() { bool (* _817)(const Self&, const Self&) = &hicc::make_eq<Self, Self>; (void)_817; }
#line 817
EXPORT_METHOD_IN(SelfContainer, Self, ((bool (*)(const Self&, const Self&))&hicc::make_eq<Self, Self>));
#line 798
};
#line 821
template<class K, class V, class Compare, class Allocator> struct CppMultiMapRevIterMut_821 {
#line 821
typedef typename std::multimap<K, V, Compare, Allocator>::reverse_iterator Self; typedef std::multimap<K, V, Compare, Allocator> SelfContainer; typedef CppMultiMapRevIterMut_821<K,V,Compare,Allocator> SelfMethods;
#line 824
static void next(Self& self) {
                ++self;
            }
            static const K& key(const Self& self) {
                return self->first;
            }
            static V& value(Self& self) {
                return self->second;
            }
#line 834
static void _hicc_test_834() { void (* _834)(Self&) = &SelfMethods::next; (void)_834; }
#line 834
EXPORT_METHOD_IN(SelfContainer, Self, ((void (*)(Self&))&SelfMethods::next));
#line 836
static void _hicc_test_836() { const K& (* _836)(const Self&) = &SelfMethods::key; (void)_836; }
#line 836
EXPORT_METHOD_IN(SelfContainer, Self, ((const K& (*)(const Self&))&SelfMethods::key));
#line 838
static void _hicc_test_838() { V& (* _838)(Self&) = &SelfMethods::value; (void)_838; }
#line 838
EXPORT_METHOD_IN(SelfContainer, Self, ((V& (*)(Self&))&SelfMethods::value));
#line 840
static void _hicc_test_840() { bool (* _840)(const Self&, const Self&) = &hicc::make_eq<Self, Self>; (void)_840; }
#line 840
EXPORT_METHOD_IN(SelfContainer, Self, ((bool (*)(const Self&, const Self&))&hicc::make_eq<Self, Self>));
#line 821
};

#endif
