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
};

#endif
