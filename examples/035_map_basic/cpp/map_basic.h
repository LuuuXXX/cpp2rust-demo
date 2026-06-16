#pragma once

#include <map>
#include <string>
#include <unordered_map>

namespace map_basic_ns {

// StringIntMap：直接持有 std::map<std::string, int>，演示有序映射的基本操作。
// hicc 直出无需 extern-C 不透明指针 + *_delete，析构由 Rust Drop 自动完成。
class StringIntMap {
    std::map<std::string, int> data_;
public:
    StringIntMap() = default;

    void insert(const char* key, int value) { data_[key ? key : ""] = value; }
    int get(const char* key) const {
        auto it = data_.find(key ? key : "");
        return it == data_.end() ? -1 : it->second;
    }
    int contains(const char* key) const { return data_.count(key ? key : "") ? 1 : 0; }
    int size() const { return static_cast<int>(data_.size()); }
    int erase(const char* key) { return static_cast<int>(data_.erase(key ? key : "")); }
    void clear() { data_.clear(); }
    const char* first_key() const { return data_.empty() ? "" : data_.begin()->first.c_str(); }
};

// Counter：直接持有 std::unordered_map<std::string, int>，演示词频计数。
class Counter {
    std::unordered_map<std::string, int> counts_;
    std::string last_;
public:
    Counter() = default;

    void add(const char* word) {
        last_ = word ? word : "";
        ++counts_[last_];
    }
    int count(const char* word) const {
        auto it = counts_.find(word ? word : "");
        return it == counts_.end() ? 0 : it->second;
    }
    int unique_words() const { return static_cast<int>(counts_.size()); }
    const char* last_word() const { return last_.c_str(); }
    void clear() { counts_.clear(); last_.clear(); }
};

// 锚点：本单元可链接的非模板符号。
int map_basic_anchor();

} // namespace map_basic_ns
