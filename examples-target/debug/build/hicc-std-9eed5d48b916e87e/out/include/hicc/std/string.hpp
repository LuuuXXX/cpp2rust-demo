#ifndef __SRC_STD_STRING_RS
#define __SRC_STD_STRING_RS

#include <hicc/hicc.hpp>
#line 0 R"(./src/std_string.rs)"
#line 9
#include <string>
#line 13
template<class CharT, class Traits, class Allocator> struct basic_string_13;
#line 13
namespace hicc { template<class CharT, class Traits, class Allocator> struct MethodsType<std::basic_string<CharT, Traits, Allocator>, void> { typedef basic_string_13<CharT,Traits,Allocator> methods_type; }; }
#line 13
template<class CharT, class Traits, class Allocator> struct basic_string_13 {
#line 13
typedef std::basic_string<CharT, Traits, Allocator> Self; typedef void SelfContainer; typedef basic_string_13<CharT,Traits,Allocator> SelfMethods;
#line 23
static void _hicc_test_23() { const CharT* (Self::* _23)() const = &Self::c_str; (void)_23; }
#line 23
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((const CharT* (Self::*)() const)&Self::c_str));
#line 31
static void _hicc_test_31() { bool (Self::* _31)() const = &Self::empty; (void)_31; }
#line 31
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((bool (Self::*)() const)&Self::empty));
#line 39
static void _hicc_test_39() { size_t (Self::* _39)() const = &Self::size; (void)_39; }
#line 39
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((size_t (Self::*)() const)&Self::size));
#line 47
static void _hicc_test_47() { size_t (Self::* _47)() const = &Self::max_size; (void)_47; }
#line 47
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((size_t (Self::*)() const)&Self::max_size));
#line 55
static void _hicc_test_55() { size_t (Self::* _55)() const = &Self::capacity; (void)_55; }
#line 55
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((size_t (Self::*)() const)&Self::capacity));
#line 64
static void _hicc_test_64() { void (Self::* _64)(size_t, CharT) = &Self::resize; (void)_64; }
#line 64
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((void (Self::*)(size_t, CharT))&Self::resize));
#line 73
static void _hicc_test_73() { void (Self::* _73)(size_t) = &Self::reserve; (void)_73; }
#line 73
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((void (Self::*)(size_t))&Self::reserve));
#line 82
static void _hicc_test_82() { void (Self::* _82)() = &Self::clear; (void)_82; }
#line 82
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((void (Self::*)())&Self::clear));
#line 94
static void _hicc_test_94() { void (Self::* _94)() = &Self::shrink_to_fit; (void)_94; }
#line 94
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((void (Self::*)())&Self::shrink_to_fit));
#line 110
static void _hicc_test_110() { Self (Self::* _110)(size_t, size_t) const = &Self::substr; (void)_110; }
#line 110
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((Self (Self::*)(size_t, size_t) const)&Self::substr));
#line 120
static void _hicc_test_120() { int (Self::* _120)(const Self&) const = &Self::compare; (void)_120; }
#line 120
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((int (Self::*)(const Self&) const)&Self::compare));
#line 129
static void _hicc_test_129() { size_t (Self::* _129)(const Self&, size_t) const = &Self::find; (void)_129; }
#line 129
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((size_t (Self::*)(const Self&, size_t) const)&Self::find));
#line 137
static void _hicc_test_137() { size_t (Self::* _137)(CharT, size_t) const = &Self::find; (void)_137; }
#line 137
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((size_t (Self::*)(CharT, size_t) const)&Self::find));
#line 146
static void _hicc_test_146() { size_t (Self::* _146)(const Self&, size_t) const = &Self::rfind; (void)_146; }
#line 146
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((size_t (Self::*)(const Self&, size_t) const)&Self::rfind));
#line 154
static void _hicc_test_154() { size_t (Self::* _154)(CharT, size_t) const = &Self::rfind; (void)_154; }
#line 154
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((size_t (Self::*)(CharT, size_t) const)&Self::rfind));
#line 163
static void _hicc_test_163() { size_t (Self::* _163)(const Self&, size_t) const = &Self::find_first_of; (void)_163; }
#line 163
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((size_t (Self::*)(const Self&, size_t) const)&Self::find_first_of));
#line 171
static void _hicc_test_171() { size_t (Self::* _171)(CharT, size_t) const = &Self::find_first_of; (void)_171; }
#line 171
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((size_t (Self::*)(CharT, size_t) const)&Self::find_first_of));
#line 180
static void _hicc_test_180() { size_t (Self::* _180)(const Self&, size_t) const = &Self::find_last_of; (void)_180; }
#line 180
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((size_t (Self::*)(const Self&, size_t) const)&Self::find_last_of));
#line 188
static void _hicc_test_188() { size_t (Self::* _188)(CharT, size_t) const = &Self::find_last_of; (void)_188; }
#line 188
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((size_t (Self::*)(CharT, size_t) const)&Self::find_last_of));
#line 197
static void _hicc_test_197() { size_t (Self::* _197)(const Self&, size_t) const = &Self::find_first_not_of; (void)_197; }
#line 197
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((size_t (Self::*)(const Self&, size_t) const)&Self::find_first_not_of));
#line 205
static void _hicc_test_205() { size_t (Self::* _205)(CharT, size_t) const = &Self::find_first_not_of; (void)_205; }
#line 205
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((size_t (Self::*)(CharT, size_t) const)&Self::find_first_not_of));
#line 214
static void _hicc_test_214() { size_t (Self::* _214)(const Self&, size_t) const = &Self::find_last_not_of; (void)_214; }
#line 214
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((size_t (Self::*)(const Self&, size_t) const)&Self::find_last_not_of));
#line 222
static void _hicc_test_222() { size_t (Self::* _222)(CharT, size_t) const = &Self::find_last_not_of; (void)_222; }
#line 222
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((size_t (Self::*)(CharT, size_t) const)&Self::find_last_not_of));
#line 238
static void _hicc_test_238() { Self& (Self::* _238)(const Self&, size_t, size_t) = &Self::append; (void)_238; }
#line 238
static void _043dcc3c3c72e8e1fca8cd2676279bc7_238(Self& self, const Self& _0, size_t _1, size_t _2){ self.append(_0, _1, _2); }
#line 238
EXPORT_METHOD_IN(SelfContainer, Self, SelfMethods::_043dcc3c3c72e8e1fca8cd2676279bc7_238);
#line 247
static void _hicc_test_247() { Self& (Self::* _247)(size_t, CharT) = &Self::append; (void)_247; }
#line 247
static void _b63f9a96e583d644e40d0ecc1bd968a7_247(Self& self, size_t _0, CharT _1){ self.append(_0, _1); }
#line 247
EXPORT_METHOD_IN(SelfContainer, Self, SelfMethods::_b63f9a96e583d644e40d0ecc1bd968a7_247);
#line 265
static void _hicc_test_265() { Self& (Self::* _265)(size_t, const Self&, size_t, size_t) = &Self::insert; (void)_265; }
#line 265
static void _c058fe5827527b51904116fd737f89cf_265(Self& self, size_t _0, const Self& _1, size_t _2, size_t _3){ self.insert(_0, _1, _2, _3); }
#line 265
EXPORT_METHOD_IN(SelfContainer, Self, SelfMethods::_c058fe5827527b51904116fd737f89cf_265);
#line 280
static void _hicc_test_280() { Self& (Self::* _280)(size_t, size_t, CharT) = &Self::insert; (void)_280; }
#line 280
static void _d153dfdb4911fba7bcd8337e631d211b_280(Self& self, size_t _0, size_t _1, CharT _2){ self.insert(_0, _1, _2); }
#line 280
EXPORT_METHOD_IN(SelfContainer, Self, SelfMethods::_d153dfdb4911fba7bcd8337e631d211b_280);
#line 296
static void _hicc_test_296() { Self& (Self::* _296)(size_t, size_t, const Self&, size_t, size_t) = &Self::replace; (void)_296; }
#line 296
static void _d28cc16dac340303033bfe000822b895_296(Self& self, size_t _0, size_t _1, const Self& _2, size_t _3, size_t _4){ self.replace(_0, _1, _2, _3, _4); }
#line 296
EXPORT_METHOD_IN(SelfContainer, Self, SelfMethods::_d28cc16dac340303033bfe000822b895_296);
#line 311
static void _hicc_test_311() { Self& (Self::* _311)(size_t, size_t, size_t, CharT) = &Self::replace; (void)_311; }
#line 311
static void _85f69ca372411330df4b5ed0409980c5_311(Self& self, size_t _0, size_t _1, size_t _2, CharT _3){ self.replace(_0, _1, _2, _3); }
#line 311
EXPORT_METHOD_IN(SelfContainer, Self, SelfMethods::_85f69ca372411330df4b5ed0409980c5_311);
#line 327
static void _hicc_test_327() { Self& (Self::* _327)(const Self&, size_t, size_t) = &Self::assign; (void)_327; }
#line 327
static void _3ad4c7dfc93da6157944005e405de9a8_327(Self& self, const Self& _0, size_t _1, size_t _2){ self.assign(_0, _1, _2); }
#line 327
EXPORT_METHOD_IN(SelfContainer, Self, SelfMethods::_3ad4c7dfc93da6157944005e405de9a8_327);
#line 336
static void _hicc_test_336() { Self& (Self::* _336)(size_t, CharT) = &Self::assign; (void)_336; }
#line 336
static void _7a8a329cb875bc4fda039e9c8a6d439a_336(Self& self, size_t _0, CharT _1){ self.assign(_0, _1); }
#line 336
EXPORT_METHOD_IN(SelfContainer, Self, SelfMethods::_7a8a329cb875bc4fda039e9c8a6d439a_336);
#line 351
static void _hicc_test_351() { Self& (Self::* _351)(size_t, size_t) = &Self::erase; (void)_351; }
#line 351
static void _39801df0a3c246c5e34a345e05f727ae_351(Self& self, size_t _0, size_t _1){ self.erase(_0, _1); }
#line 351
EXPORT_METHOD_IN(SelfContainer, Self, SelfMethods::_39801df0a3c246c5e34a345e05f727ae_351);
#line 361
static void _hicc_test_361() { void (Self::* _361)(CharT) = &Self::push_back; (void)_361; }
#line 361
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((void (Self::*)(CharT))&Self::push_back));
#line 376
static void _hicc_test_376() { void (Self::* _376)() = &Self::pop_back; (void)_376; }
#line 376
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((void (Self::*)())&Self::pop_back));
#line 387
static void _hicc_test_387() { void (Self::* _387)(Self&) = &Self::swap; (void)_387; }
#line 387
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((void (Self::*)(Self&))&Self::swap));
#line 402
static void _hicc_test_402() { const CharT& (Self::* _402)(size_t) const = &Self::at; (void)_402; }
#line 402
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((const CharT& (Self::*)(size_t) const)&Self::at));
#line 419
static void _hicc_test_419() { CharT& (Self::* _419)(size_t) = &Self::at; (void)_419; }
#line 419
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((CharT& (Self::*)(size_t))&Self::at));
#line 13
};

#endif
