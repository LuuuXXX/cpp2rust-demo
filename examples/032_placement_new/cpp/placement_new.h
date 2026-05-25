#pragma once

#ifdef __cplusplus
extern "C" {
#endif

// Placement new 示例
// 展示如何在预分配内存中构造对象

#include <stddef.h>

// 缓冲区结构
class Buffer;

// 创建预分配缓冲区
Buffer* buffer_new(size_t capacity);

// 销毁缓冲区（但不调用对象的析构函数）
void buffer_delete(Buffer* self);

// 获取缓冲区起始地址
void* buffer_data(Buffer* self);

// 获取缓冲区容量
size_t buffer_capacity(Buffer* self);

// 在缓冲区中构造对象
// 返回构造的对象指针
void* buffer_construct(Buffer* self, size_t offset);

// 获取已构造对象的大小（模拟）
size_t buffer_size(Buffer* self);

// 模拟 std::vector 风格的内存管理
class VectorBuffer;

// 创建 vector 缓冲区
VectorBuffer* vector_buffer_new(size_t capacity);

// 销毁 vector 缓冲区（调用所有对象的析构函数）
void vector_buffer_delete(VectorBuffer* self);

// 获取数据指针
void* vector_buffer_data(VectorBuffer* self);

// 获取元素大小
size_t vector_buffer_element_size(VectorBuffer* self);

#ifdef __cplusplus
}

// Full class definition - for hicc code generation
class Buffer {
    char* data_;
    size_t capacity_;
    size_t constructed_size_;
public:
    explicit Buffer(size_t capacity);
    ~Buffer();
    Buffer(const Buffer&) = delete;
    Buffer& operator=(const Buffer&) = delete;
    void* data();
    size_t capacity() const;
    size_t size() const;
    void* construct(size_t offset);
};

class VectorBuffer {
    char* data_;
    size_t capacity_;
    size_t size_;
    size_t element_size_;
public:
    explicit VectorBuffer(size_t capacity, size_t elem_size);
    ~VectorBuffer();
    VectorBuffer(const VectorBuffer&) = delete;
    VectorBuffer& operator=(const VectorBuffer&) = delete;
    void* data();
    size_t element_size() const;
    void destroy_all();
};

#endif
