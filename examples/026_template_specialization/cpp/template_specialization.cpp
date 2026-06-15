#include "template_specialization.h"
#include <iostream>
#include <cstring>
#include <cstdlib>
#include <cstdio>

IntHolder::IntHolder(int value) : value_(value) {}
IntHolder::~IntHolder() {}
int IntHolder::get() const { return value_; }
const char* IntHolder::describe() const {
    static char buf[64];
    snprintf(buf, sizeof(buf), "IntHolder(value=%d)", value_);
    return buf;
}

DoubleHolder::DoubleHolder(double value) : value_(value) {}
DoubleHolder::~DoubleHolder() {}
double DoubleHolder::get() const { return value_; }
const char* DoubleHolder::describe() const {
    static char buf[64];
    snprintf(buf, sizeof(buf), "DoubleHolder(value=%.5f)", value_);
    return buf;
}

StringHolder::StringHolder(const char* value) {
    value_ = strdup(value);
    length_ = strlen(value);
}
StringHolder::~StringHolder() {
    if (value_) free(value_);
}
const char* StringHolder::get() const { return value_; }
const char* StringHolder::describe() const {
    static char buf[256];
    snprintf(buf, sizeof(buf), "StringHolder(value=\"%s\", length=%d)", value_, length_);
    return buf;
}
