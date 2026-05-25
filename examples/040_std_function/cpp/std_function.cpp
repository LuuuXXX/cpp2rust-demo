#include "std_function.h"
#include <iostream>
#include <functional>
#include <vector>
#include <thread>
#include <chrono>

// CallbackWrapperImpl implementation
CallbackWrapperImpl::CallbackWrapperImpl(int (*fn)(int)) : callback(fn) {}
CallbackWrapperImpl::~CallbackWrapperImpl() {}
int CallbackWrapperImpl::invoke(int value) {
    if (callback) return callback(value);
    return value;
}
void CallbackWrapperImpl::set(int (*fn)(int)) {
    callback = fn;
}

// ProcessorImpl implementation
ProcessorImpl::ProcessorImpl() : callback(nullptr) {}
ProcessorImpl::~ProcessorImpl() {}
void ProcessorImpl::set_callback(int (*cb)(int)) {
    callback = cb;
}
int ProcessorImpl::process(int value) {
    if (callback) return callback(value);
    return value;
}

// MultiCallbackImpl implementation
MultiCallbackImpl::MultiCallbackImpl() {}
MultiCallbackImpl::~MultiCallbackImpl() {}
void MultiCallbackImpl::add(int (*cb)(int)) {
    callbacks.push_back(cb);
}
void MultiCallbackImpl::invoke_all(int value) {
    for (auto& cb : callbacks) {
        cb(value);
    }
}

// AsyncProcessorImpl implementation
AsyncProcessorImpl::AsyncProcessorImpl() : cancelled(false) {}
AsyncProcessorImpl::~AsyncProcessorImpl() {}
void AsyncProcessorImpl::set_completion_callback(void (*cb)(int, int)) {
    completion_callback = cb;
}
void AsyncProcessorImpl::set_progress_callback(void (*cb)(int)) {
    progress_callback = cb;
}
void AsyncProcessorImpl::start(int value) {
    cancelled = false;
    // Simulate async processing
    for (int i = 0; i <= 100; i += 20) {
        if (cancelled) break;
        if (progress_callback) progress_callback(i);
        std::this_thread::sleep_for(std::chrono::milliseconds(100));
    }
    if (completion_callback) completion_callback(value, value * 2);
}
void AsyncProcessorImpl::cancel() {
    cancelled = true;
}

// CallbackWrapper implementation
CallbackWrapper::CallbackWrapper(int (*fn)(int)) : impl(new CallbackWrapperImpl(fn)) {}
CallbackWrapper::~CallbackWrapper() { delete impl; }

// Processor implementation
Processor::Processor() : impl(new ProcessorImpl()) {}
Processor::~Processor() { delete impl; }

// MultiCallback implementation
MultiCallback::MultiCallback() : impl(new MultiCallbackImpl()) {}
MultiCallback::~MultiCallback() { delete impl; }

// AsyncProcessor implementation
AsyncProcessor::AsyncProcessor() : impl(new AsyncProcessorImpl()) {}
AsyncProcessor::~AsyncProcessor() { delete impl; }

// CallbackWrapper C API implementation
struct CallbackWrapper* callback_wrapper_new(int (*fn)(int)) {
    return new CallbackWrapper(fn);
}

void callback_wrapper_delete(struct CallbackWrapper* self) {
    delete self;
}

int callback_wrapper_invoke(const struct CallbackWrapper* self, int value) {
    if (self) {
        return self->impl->invoke(value);
    }
    return value;
}

void callback_wrapper_set(struct CallbackWrapper* self, int (*fn)(int)) {
    if (self) {
        self->impl->set(fn);
    }
}

// Processor C API implementation
struct Processor* processor_new(void) {
    return new Processor();
}

void processor_delete(struct Processor* self) {
    delete self;
}

void processor_set_callback(struct Processor* self, int (*cb)(int)) {
    if (self) {
        self->impl->set_callback(cb);
    }
}

int processor_process(const struct Processor* self, int value) {
    if (self) {
        return self->impl->process(value);
    }
    return value;
}

// MultiCallback C API implementation
struct MultiCallback* multi_callback_new(void) {
    return new MultiCallback();
}

void multi_callback_delete(struct MultiCallback* self) {
    delete self;
}

void multi_callback_add(struct MultiCallback* self, int (*cb)(int)) {
    if (self) {
        self->impl->add(cb);
    }
}

void multi_callback_invoke_all(struct MultiCallback* self, int value) {
    if (self) {
        self->impl->invoke_all(value);
    }
}

// AsyncProcessor C API implementation
struct AsyncProcessor* async_processor_new(void) {
    return new AsyncProcessor();
}

void async_processor_delete(struct AsyncProcessor* self) {
    delete self;
}

void async_processor_set_callback(struct AsyncProcessor* self, void (*cb)(int, int)) {
    if (self) {
        self->impl->set_completion_callback(cb);
    }
}

void async_processor_set_progress_callback(struct AsyncProcessor* self, void (*cb)(int)) {
    if (self) {
        self->impl->set_progress_callback(cb);
    }
}

void async_processor_start(struct AsyncProcessor* self, int value) {
    if (self) {
        self->impl->start(value);
    }
}

void async_processor_cancel(struct AsyncProcessor* self) {
    if (self) {
        self->impl->cancel();
    }
}
