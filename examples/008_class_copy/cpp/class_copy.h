#pragma once

#ifdef __cplusplus
extern "C" {
#endif

struct Buffer;

// 构造函数
struct Buffer* buffer_new(void);
struct Buffer* buffer_newWithSize(int size);
struct Buffer* buffer_newCopy(const struct Buffer* other);

// 析构函数
void buffer_delete(struct Buffer* self);

// 操作
void buffer_set(struct Buffer* self, int index, int value);
int buffer_get(const struct Buffer* self, int index);
int buffer_size(const struct Buffer* self);

#ifdef __cplusplus
}

// Full class definition - for hicc code generation
class Buffer {
    int* data;
    int size;
public:
    Buffer();
    explicit Buffer(int sz);
    Buffer(const Buffer& other);
    ~Buffer();
    void set(int index, int value);
    int get(int index) const;
    int getSize() const;
};

#endif
