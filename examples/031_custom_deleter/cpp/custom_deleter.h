#pragma once

#include <string>
#include <memory>

namespace custom_deleter_ns {

// 自定义删除器被调用的累计次数（演示删除器确实被触发）。
int cleanup_count();

// 自定义删除器：释放负载时记录一次清理，模拟带计数/日志的资源回收策略。
struct LoggingDeleter {
    void operator()(std::string* p) const;
};

// ManagedResource：用 unique_ptr<T, 自定义删除器> 持有负载，演示 RAII 自定义删除策略。
// hicc 直出无需手写 *_delete；Rust Drop 触发析构时内部 unique_ptr 会调用自定义删除器。
class ManagedResource {
    std::unique_ptr<std::string, LoggingDeleter> res_;
public:
    explicit ManagedResource(const char* name)
        : res_(new std::string(name ? name : ""), LoggingDeleter{}) {}

    const char* name() const { return res_ ? res_->c_str() : ""; }

    // 是否已释放负载（类似已 reset 的 unique_ptr）。
    int released() const { return res_ ? 0 : 1; }

    // 主动释放负载，触发自定义删除器。
    void release() { res_.reset(); }
};

} // namespace custom_deleter_ns
