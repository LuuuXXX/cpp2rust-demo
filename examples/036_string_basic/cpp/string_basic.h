#pragma once

#ifdef __cplusplus
extern "C" {
#endif

// std::string 基本操作示例

#include <stddef.h>

// Forward declaration (opaque pointer)
struct String;

// Creation and destruction
struct String* string_new(void);
struct String* string_new_from(const char* str);
struct String* string_new_from_len(const char* str, size_t len);
void string_delete(struct String* self);

// Basic operations
size_t string_size(const struct String* self);
size_t string_length(const struct String* self);
int string_empty(const struct String* self);
const char* string_c_str(const struct String* self);

// Comparison operations
int string_compare(const struct String* self, const char* other);
int string_equals(const struct String* self, const char* other);
int string_starts_with(const struct String* self, const char* prefix);
int string_ends_with(const struct String* self, const char* suffix);

// Concatenation operations
struct String* string_concat(const struct String* self, const char* other);
void string_append(struct String* self, const char* other);
void string_append_len(struct String* self, const char* other, size_t len);

// Modification operations
void string_clear(struct String* self);
void string_trim(struct String* self);

// Substring operations
struct String* string_substr(const struct String* self, size_t pos, size_t len);
int string_find(const struct String* self, const char* substr);

// Case conversion
void string_to_upper(struct String* self);
void string_to_lower(struct String* self);

#ifdef __cplusplus
}

// Full class definition - for hicc code generation
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

#endif
