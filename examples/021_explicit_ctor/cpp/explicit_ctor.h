#pragma once

#ifdef __cplusplus
extern "C" {
#endif

struct Widget;

struct Widget* widget_new(int value);
struct Widget* widget_fromInt(int value);
struct Widget* widget_fromDouble(double value);
void widget_delete(struct Widget* self);

int widget_getValue(struct Widget* self);

#ifdef __cplusplus
}

// Full class definition - for hicc code generation
class Widget {
    int value;
public:
    Widget(int v);
    explicit Widget(double v);
    ~Widget();
    int getValue() const;
};

#endif
