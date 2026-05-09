#include "widget.hpp"
#include <cstdio>

static int g_instance_count = 0;

Widget::Widget(int id) : id_(id) {
    ++g_instance_count;
    std::printf("Widget(%d) created\n", id_);
}

Widget::~Widget() {
    --g_instance_count;
    std::printf("Widget(%d) destroyed\n", id_);
}

void Widget::update(double x, double y) {
    x_ = x;
    y_ = y;
    std::printf("Widget(%d).update(x=%f, y=%f)\n", id_, x_, y_);
}

int Widget::getId() const { return id_; }
bool Widget::isVisible() const { return visible_; }
int Widget::instanceCount() { return g_instance_count; }
