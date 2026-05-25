#include "functional_bind.h"
#include <iostream>
#include <functional>
#include <string>

// AdderImpl implementation
AdderImpl::AdderImpl(int base) : base_value(base) {}
AdderImpl::~AdderImpl() {}
int AdderImpl::add(int value) {
    return base_value + value;
}

// Adder implementation
Adder::Adder(int base_value) : impl(new AdderImpl(base_value)) {}
Adder::~Adder() { delete impl; }

// MultiplierImpl implementation
MultiplierImpl::MultiplierImpl(int f) : factor(f) {}
MultiplierImpl::~MultiplierImpl() {}
int MultiplierImpl::multiply(int value) {
    return factor * value;
}

// Multiplier implementation
Multiplier::Multiplier(int factor) : impl(new MultiplierImpl(factor)) {}
Multiplier::~Multiplier() { delete impl; }

// StringProcessorImpl implementation
StringProcessorImpl::StringProcessorImpl() {}
StringProcessorImpl::~StringProcessorImpl() {}
void StringProcessorImpl::set_target(const char* t) {
    target = t;
}
int StringProcessorImpl::count_char(char ch) {
    int count = 0;
    for (char c : target) {
        if (c == ch) count++;
    }
    return count;
}

// StringProcessor implementation
StringProcessor::StringProcessor() : impl(new StringProcessorImpl()) {}
StringProcessor::~StringProcessor() { delete impl; }

struct Adder* adder_new(int base_value) {
    return new Adder(base_value);
}

void adder_delete(struct Adder* self) {
    delete self;
}

int adder_add(const struct Adder* self, int value) {
    if (self) {
        return self->impl->add(value);
    }
    return value;
}

// Bound functions
int add_five_impl(int a, int b) {
    std::cout << "add_five called: " << a << " + 5 = " << (a + 5) << std::endl;
    return a + 5;
}

int add_ten_impl(int a, int b) {
    std::cout << "add_ten called: " << a << " + 10 = " << (a + 10) << std::endl;
    return a + 10;
}

int add_five(int a) {
    return add_five_impl(a, 5);
}

int add_ten(int a) {
    return add_ten_impl(a, 10);
}

// Multiplier C API implementation
struct Multiplier* multiplier_new(int factor) {
    return new Multiplier(factor);
}

void multiplier_delete(struct Multiplier* self) {
    delete self;
}

int multiply(const struct Multiplier* self, int value) {
    if (self) {
        return self->impl->multiply(value);
    }
    return value;
}

// StringProcessor C API implementation
struct StringProcessor* string_processor_new(void) {
    return new StringProcessor();
}

void string_processor_delete(struct StringProcessor* self) {
    delete self;
}

void string_processor_set_target(struct StringProcessor* self, const char* target) {
    if (self) {
        self->impl->set_target(target);
    }
}

int string_processor_count_char(const struct StringProcessor* self, char ch) {
    if (self) {
        return self->impl->count_char(ch);
    }
    return 0;
}
