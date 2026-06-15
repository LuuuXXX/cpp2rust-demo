#pragma once

#include <string>

class StringImpl {
public:
    std::string data;
    StringImpl();
    explicit StringImpl(const char* str);
    explicit StringImpl(const char* str, size_t len);
    ~StringImpl();
};

struct String {
    StringImpl* impl;
    explicit String();
    explicit String(const char* str);
    explicit String(const char* str, size_t len);
    ~String();
    const char* c_str() const { return impl->data.c_str(); }
    size_t size() const { return impl->data.size(); }
    size_t length() const { return impl->data.length(); }
    bool empty() const { return impl->data.empty(); }
    int compare(const char* str) const { return impl->data.compare(str ? str : ""); }
    bool equals(const char* str) const { return impl->data == (str ? str : ""); }
    void append(const char* str) { if (str) impl->data += str; }
    void to_upper() { for (auto& c : impl->data) c = std::toupper((unsigned char)c); }
    void to_lower() { for (auto& c : impl->data) c = std::tolower((unsigned char)c); }
};
