#include <hicc/hicc.hpp>
#line 0 R"(src/main.rs)"
#line 2
#include <iostream>

    class Widget {
        int value;
    public:
        Widget(int v);
        explicit Widget(double v);
        ~Widget();
        int getValue() const;
    };

    Widget* widget_new(int value) {
        return new Widget(value);
    }

    Widget* widget_fromInt(int value) {
        return new Widget(value);
    }

    Widget* widget_fromDouble(double value) {
        return new Widget(value);
    }

    void widget_delete(Widget* self) {
        delete self;
    }

    int widget_getValue(Widget* self) {
        return self->getValue();
    }

    Widget::Widget(int v) : value(v) {}
    Widget::Widget(double v) : value(static_cast<int>(v)) {}
    Widget::~Widget() {}
    int Widget::getValue() const { return value; }
#line 40
 struct Widget_40;
#line 40
namespace hicc { template<> struct MethodsType<Widget, void> { typedef Widget_40 methods_type; }; }
#line 40
 struct Widget_40 {
#line 40
typedef Widget Self; typedef void SelfContainer; typedef Widget_40 SelfMethods;
#line 42
static void _hicc_test_42() { int (Self::* _42)() const = &Self::getValue; (void)_42; }
#line 42
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((int (Self::*)() const)&Self::getValue));
#line 40
};
#line 48
EXPORT_METHODS_BEG(explicit_ctor) {
#line 52
static void _hicc_test_52() { Widget* (* _52)(int value) = &widget_new; (void)_52; }
#line 52
EXPORT_METHOD_IN(void, ExportMethods, ((Widget* (*)(int value))&widget_new));
#line 55
static void _hicc_test_55() { Widget* (* _55)(int value) = &widget_fromInt; (void)_55; }
#line 55
EXPORT_METHOD_IN(void, ExportMethods, ((Widget* (*)(int value))&widget_fromInt));
#line 58
static void _hicc_test_58() { Widget* (* _58)(double value) = &widget_fromDouble; (void)_58; }
#line 58
EXPORT_METHOD_IN(void, ExportMethods, ((Widget* (*)(double value))&widget_fromDouble));
#line 61
static void _hicc_test_61() { void (* _61)(Widget* self) = &widget_delete; (void)_61; }
#line 61
EXPORT_METHOD_IN(void, ExportMethods, ((void (*)(Widget* self))&widget_delete));
#line 64
static void _hicc_test_64() { int (* _64)(Widget* self) = &widget_getValue; (void)_64; }
#line 64
EXPORT_METHOD_IN(void, ExportMethods, ((int (*)(Widget* self))&widget_getValue));
#line 48
} EXPORT_METHODS_END();

