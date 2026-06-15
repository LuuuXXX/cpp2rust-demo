#pragma once

#include <functional>
#include <vector>

class CallbackWrapperImpl {
public:
    std::function<int(int)> callback;
    explicit CallbackWrapperImpl(int (*fn)(int));
    ~CallbackWrapperImpl();
    int invoke(int value);
    void set(int (*fn)(int));
};

class ProcessorImpl {
public:
    std::function<int(int)> callback;
    ProcessorImpl();
    ~ProcessorImpl();
    void set_callback(int (*cb)(int));
    int process(int value);
};

class MultiCallbackImpl {
public:
    std::vector<std::function<int(int)>> callbacks;
    MultiCallbackImpl();
    ~MultiCallbackImpl();
    void add(int (*cb)(int));
    void invoke_all(int value);
};

class AsyncProcessorImpl {
public:
    std::function<void(int, int)> completion_callback;
    std::function<void(int)> progress_callback;
    bool cancelled;
    AsyncProcessorImpl();
    ~AsyncProcessorImpl();
    void set_completion_callback(void (*cb)(int, int));
    void set_progress_callback(void (*cb)(int));
    void start(int value);
    void cancel();
};

struct CallbackWrapper {
    CallbackWrapperImpl* impl;
    explicit CallbackWrapper(int (*fn)(int));
    ~CallbackWrapper();
    int invoke(int value) { return impl->invoke(value); }
};

struct Processor {
    ProcessorImpl* impl;
    Processor();
    ~Processor();
    int process(int value) { return impl->process(value); }
};

struct MultiCallback {
    MultiCallbackImpl* impl;
    MultiCallback();
    ~MultiCallback();
    void invoke_all(int value) { impl->invoke_all(value); }
};

struct AsyncProcessor {
    AsyncProcessorImpl* impl;
    AsyncProcessor();
    ~AsyncProcessor();
    bool is_cancelled() const { return impl->cancelled; }
    void cancel() { impl->cancel(); }
};
