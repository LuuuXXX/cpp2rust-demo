#pragma once

#include <map>
#include <string>

namespace foo {
namespace bar { namespace config {

// ConfigManager：直接持有 std::map<std::string, int>，演示嵌套命名空间类绑定。
// hicc 直出无需 extern-C 不透明指针 + *_delete，析构由 Rust Drop 自动完成。
class ConfigManager {
    std::map<std::string, int> values_;
public:
    ConfigManager() = default;

    void set_value(const char* key, int value) { values_[key ? key : ""] = value; }
    int get_value(const char* key) const {
        auto it = values_.find(key ? key : "");
        return it == values_.end() ? -1 : it->second;
    }
    int size() const { return static_cast<int>(values_.size()); }
};

}} // namespace bar::config

namespace baz {

class DataProcessor {
    int multiplier_;
public:
    DataProcessor();
    int process(int input) const { return input * multiplier_; }
};

} // namespace baz

const char* get_version();
int get_build_number();

// 锚点：本单元可链接的非模板符号。
int namespace_nested_anchor();

} // namespace foo
