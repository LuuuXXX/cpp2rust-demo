#pragma once

class Widget {
    int value;
public:
    Widget(int v);
    explicit Widget(double v);
    ~Widget();
    int getValue() const;
};
