#include <hicc/hicc.hpp>
#line 0 R"(src/main.rs)"
#line 2
#include <iostream>


    class BinaryOp {
    public:
        int last_a;
        int last_b;
        int last_result;
        BinaryOp() : last_a(0), last_b(0), last_result(0) {}
        void store(int a, int b, int result) { last_a = a; last_b = b; last_result = result; }
        int get_a() const { return last_a; }
        int get_b() const { return last_b; }
        int get_result() const { return last_result; }
    };

    BinaryOp* binary_op_new() { return new BinaryOp(); }
    void binary_op_delete(BinaryOp* self) { delete self; }
#line 22
 struct BinaryOp_22;
#line 22
namespace hicc { template<> struct MethodsType<BinaryOp, void> { typedef BinaryOp_22 methods_type; }; }
#line 22
 struct BinaryOp_22 {
#line 22
typedef BinaryOp Self; typedef void SelfContainer; typedef BinaryOp_22 SelfMethods;
#line 24
static void _hicc_test_24() { int (Self::* _24)() const = &Self::get_a; (void)_24; }
#line 24
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((int (Self::*)() const)&Self::get_a));
#line 27
static void _hicc_test_27() { int (Self::* _27)() const = &Self::get_b; (void)_27; }
#line 27
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((int (Self::*)() const)&Self::get_b));
#line 30
static void _hicc_test_30() { int (Self::* _30)() const = &Self::get_result; (void)_30; }
#line 30
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((int (Self::*)() const)&Self::get_result));
#line 33
static void _hicc_test_33() { void (Self::* _33)(int, int, int) = &Self::store; (void)_33; }
#line 33
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((void (Self::*)(int, int, int))&Self::store));
#line 22
};
#line 39
EXPORT_METHODS_BEG(lambda_basic) {
#line 43
static void _hicc_test_43() { BinaryOp* (* _43)() = &binary_op_new; (void)_43; }
#line 43
EXPORT_METHOD_IN(void, ExportMethods, ((BinaryOp* (*)())&binary_op_new));
#line 46
static void _hicc_test_46() { void (* _46)(BinaryOp* self) = &binary_op_delete; (void)_46; }
#line 46
EXPORT_METHOD_IN(void, ExportMethods, ((void (*)(BinaryOp* self))&binary_op_delete));
#line 39
} EXPORT_METHODS_END();

