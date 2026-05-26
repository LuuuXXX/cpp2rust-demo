#include <hicc/hicc.hpp>
#line 0 R"(src/main.rs)"
#line 2
#include <iostream>
    #include <stack>

    template<typename T>
    class Stack {
    public:
        std::stack<T> data;
        Stack() = default;
        int size() const { return static_cast<int>(data.size()); }
        bool empty() const { return data.empty(); }
        void push(T value) { data.push(value); }
        T top() const { return data.top(); }
        void pop() { data.pop(); }
    };

    class IntStack {
    public:
        Stack<int> impl;
        IntStack() = default;
        int size() const { return impl.size(); }
        bool empty() const { return impl.empty(); }
        void push(int value) { impl.push(value); }
        int top() const { return impl.top(); }
        void pop() { impl.pop(); }
    };

    class DoubleStack {
    public:
        Stack<double> impl;
        DoubleStack() = default;
        int size() const { return impl.size(); }
        bool empty() const { return impl.empty(); }
        void push(double value) { impl.push(value); }
        double top() const { return impl.top(); }
        void pop() { impl.pop(); }
    };

    IntStack* intstack_new(void) {
        return new IntStack();
    }

    void intstack_delete(IntStack* self) {
        delete self;
    }

    int intstack_size(IntStack* self) {
        return self->impl.size();
    }

    int intstack_empty(IntStack* self) {
        return self->impl.empty() ? 1 : 0;
    }

    void intstack_push(IntStack* self, int value) {
        self->impl.push(value);
    }

    int intstack_top(IntStack* self) {
        return self->impl.top();
    }

    void intstack_pop(IntStack* self) {
        self->impl.pop();
    }

    DoubleStack* doublestack_new(void) {
        return new DoubleStack();
    }

    void doublestack_delete(DoubleStack* self) {
        delete self;
    }

    int doublestack_size(DoubleStack* self) {
        return self->impl.size();
    }

    int doublestack_empty(DoubleStack* self) {
        return self->impl.empty() ? 1 : 0;
    }

    void doublestack_push(DoubleStack* self, double value) {
        self->impl.push(value);
    }

    double doublestack_top(DoubleStack* self) {
        return self->impl.top();
    }

    void doublestack_pop(DoubleStack* self) {
        self->impl.pop();
    }
#line 97
 struct IntStack_97;
#line 97
namespace hicc { template<> struct MethodsType<IntStack, void> { typedef IntStack_97 methods_type; }; }
#line 115
 struct DoubleStack_115;
#line 115
namespace hicc { template<> struct MethodsType<DoubleStack, void> { typedef DoubleStack_115 methods_type; }; }
#line 97
 struct IntStack_97 {
#line 97
typedef IntStack Self; typedef void SelfContainer; typedef IntStack_97 SelfMethods;
#line 99
static void _hicc_test_99() { int (Self::* _99)() const = &Self::size; (void)_99; }
#line 99
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((int (Self::*)() const)&Self::size));
#line 102
static void _hicc_test_102() { bool (Self::* _102)() const = &Self::empty; (void)_102; }
#line 102
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((bool (Self::*)() const)&Self::empty));
#line 105
static void _hicc_test_105() { void (Self::* _105)(int value) = &Self::push; (void)_105; }
#line 105
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((void (Self::*)(int value))&Self::push));
#line 108
static void _hicc_test_108() { int (Self::* _108)() const = &Self::top; (void)_108; }
#line 108
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((int (Self::*)() const)&Self::top));
#line 111
static void _hicc_test_111() { void (Self::* _111)() = &Self::pop; (void)_111; }
#line 111
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((void (Self::*)())&Self::pop));
#line 97
};
#line 115
 struct DoubleStack_115 {
#line 115
typedef DoubleStack Self; typedef void SelfContainer; typedef DoubleStack_115 SelfMethods;
#line 117
static void _hicc_test_117() { int (Self::* _117)() const = &Self::size; (void)_117; }
#line 117
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((int (Self::*)() const)&Self::size));
#line 120
static void _hicc_test_120() { bool (Self::* _120)() const = &Self::empty; (void)_120; }
#line 120
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((bool (Self::*)() const)&Self::empty));
#line 123
static void _hicc_test_123() { void (Self::* _123)(double value) = &Self::push; (void)_123; }
#line 123
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((void (Self::*)(double value))&Self::push));
#line 126
static void _hicc_test_126() { double (Self::* _126)() const = &Self::top; (void)_126; }
#line 126
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((double (Self::*)() const)&Self::top));
#line 129
static void _hicc_test_129() { void (Self::* _129)() = &Self::pop; (void)_129; }
#line 129
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((void (Self::*)())&Self::pop));
#line 115
};
#line 135
EXPORT_METHODS_BEG(template_class) {
#line 140
static void _hicc_test_140() { IntStack* (* _140)() = &intstack_new; (void)_140; }
#line 140
EXPORT_METHOD_IN(void, ExportMethods, ((IntStack* (*)())&intstack_new));
#line 143
static void _hicc_test_143() { void (* _143)(IntStack* self) = &intstack_delete; (void)_143; }
#line 143
EXPORT_METHOD_IN(void, ExportMethods, ((void (*)(IntStack* self))&intstack_delete));
#line 146
static void _hicc_test_146() { int (* _146)(IntStack* self) = &intstack_size; (void)_146; }
#line 146
EXPORT_METHOD_IN(void, ExportMethods, ((int (*)(IntStack* self))&intstack_size));
#line 149
static void _hicc_test_149() { int (* _149)(IntStack* self) = &intstack_empty; (void)_149; }
#line 149
EXPORT_METHOD_IN(void, ExportMethods, ((int (*)(IntStack* self))&intstack_empty));
#line 152
static void _hicc_test_152() { void (* _152)(IntStack* self, int value) = &intstack_push; (void)_152; }
#line 152
EXPORT_METHOD_IN(void, ExportMethods, ((void (*)(IntStack* self, int value))&intstack_push));
#line 155
static void _hicc_test_155() { int (* _155)(IntStack* self) = &intstack_top; (void)_155; }
#line 155
EXPORT_METHOD_IN(void, ExportMethods, ((int (*)(IntStack* self))&intstack_top));
#line 158
static void _hicc_test_158() { void (* _158)(IntStack* self) = &intstack_pop; (void)_158; }
#line 158
EXPORT_METHOD_IN(void, ExportMethods, ((void (*)(IntStack* self))&intstack_pop));
#line 161
static void _hicc_test_161() { DoubleStack* (* _161)() = &doublestack_new; (void)_161; }
#line 161
EXPORT_METHOD_IN(void, ExportMethods, ((DoubleStack* (*)())&doublestack_new));
#line 164
static void _hicc_test_164() { void (* _164)(DoubleStack* self) = &doublestack_delete; (void)_164; }
#line 164
EXPORT_METHOD_IN(void, ExportMethods, ((void (*)(DoubleStack* self))&doublestack_delete));
#line 167
static void _hicc_test_167() { int (* _167)(DoubleStack* self) = &doublestack_size; (void)_167; }
#line 167
EXPORT_METHOD_IN(void, ExportMethods, ((int (*)(DoubleStack* self))&doublestack_size));
#line 170
static void _hicc_test_170() { int (* _170)(DoubleStack* self) = &doublestack_empty; (void)_170; }
#line 170
EXPORT_METHOD_IN(void, ExportMethods, ((int (*)(DoubleStack* self))&doublestack_empty));
#line 173
static void _hicc_test_173() { void (* _173)(DoubleStack* self, double value) = &doublestack_push; (void)_173; }
#line 173
EXPORT_METHOD_IN(void, ExportMethods, ((void (*)(DoubleStack* self, double value))&doublestack_push));
#line 176
static void _hicc_test_176() { double (* _176)(DoubleStack* self) = &doublestack_top; (void)_176; }
#line 176
EXPORT_METHOD_IN(void, ExportMethods, ((double (*)(DoubleStack* self))&doublestack_top));
#line 179
static void _hicc_test_179() { void (* _179)(DoubleStack* self) = &doublestack_pop; (void)_179; }
#line 179
EXPORT_METHOD_IN(void, ExportMethods, ((void (*)(DoubleStack* self))&doublestack_pop));
#line 135
} EXPORT_METHODS_END();

