#pragma once

#ifdef __cplusplus
extern "C" {
#endif

struct UniqueVector;

// 构造函数
struct UniqueVector* unique_vector_new(void);
struct UniqueVector* unique_vector_newWithData(int* data, int size);

// 析构函数
void unique_vector_delete(struct UniqueVector* self);

// 操作
int unique_vector_get(const struct UniqueVector* self, int index);
void unique_vector_set(struct UniqueVector* self, int index, int value);
int unique_vector_size(const struct UniqueVector* self);

// 移动语义
void unique_vector_move(struct UniqueVector* dest, struct UniqueVector* src);

#ifdef __cplusplus
}

// Full class definition - for hicc code generation
class UniqueVector {
    int* data;
    int size;
public:
    UniqueVector();
    UniqueVector(int* data, int size);
    ~UniqueVector();
    UniqueVector(UniqueVector&& other) noexcept;
    UniqueVector& operator=(UniqueVector&& other) noexcept;
    int get(int index) const;
    void set(int index, int value);
    int getSize() const;
    void moveFrom(UniqueVector& src);
};

#endif
