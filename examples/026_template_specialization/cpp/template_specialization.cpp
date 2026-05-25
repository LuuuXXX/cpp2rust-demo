#include "template_specialization.h"
#include <iostream>
#include <cstring>
#include <cstdlib>
#include <cstdio>

// C-compatible wrapper functions
IntHolder* intholder_new(int value) {
    return new IntHolder(value);
}

void intholder_delete(IntHolder* self) {
    if (self) delete self;
}

int intholder_get(IntHolder* self) {
    return self->get();
}

const char* intholder_describe(IntHolder* self) {
    return self->describe();
}

DoubleHolder* doubleholder_new(double value) {
    return new DoubleHolder(value);
}

void doubleholder_delete(DoubleHolder* self) {
    if (self) delete self;
}

double doubleholder_get(DoubleHolder* self) {
    return self->get();
}

const char* doubleholder_describe(DoubleHolder* self) {
    return self->describe();
}

StringHolder* stringholder_new(const char* value) {
    return new StringHolder(value);
}

void stringholder_delete(StringHolder* self) {
    if (self) delete self;
}

const char* stringholder_get(StringHolder* self) {
    return self->get();
}

const char* stringholder_describe(StringHolder* self) {
    return self->describe();
}

// IntHolder implementation
IntHolder::IntHolder(int value) : value_(value) {}
IntHolder::~IntHolder() {}
int IntHolder::get() const { return value_; }
const char* IntHolder::describe() const {
    static char buf[64];
    snprintf(buf, sizeof(buf), "IntHolder(value=%d)", value_);
    return buf;
}

// DoubleHolder implementation
DoubleHolder::DoubleHolder(double value) : value_(value) {}
DoubleHolder::~DoubleHolder() {}
double DoubleHolder::get() const { return value_; }
const char* DoubleHolder::describe() const {
    static char buf[64];
    snprintf(buf, sizeof(buf), "DoubleHolder(value=%.5f)", value_);
    return buf;
}

// StringHolder implementation
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
