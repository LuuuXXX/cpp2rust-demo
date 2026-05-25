#pragma once

#ifdef __cplusplus
extern "C" {
#endif

// 模板偏特化示例：ValueHolder
// 通用版本和 char* 特化版本

// 通用版本 - 使用类声明
class IntHolder;
class DoubleHolder;

// 构造函数和析构函数
IntHolder* intholder_new(int value);
void intholder_delete(IntHolder* self);

// 访问器方法
int intholder_get(IntHolder* self);
const char* intholder_describe(IntHolder* self);

// DoubleHolder 类声明
DoubleHolder* doubleholder_new(double value);
void doubleholder_delete(DoubleHolder* self);
double doubleholder_get(DoubleHolder* self);
const char* doubleholder_describe(DoubleHolder* self);

// char* 特化版本 - StringHolder
class StringHolder;

StringHolder* stringholder_new(const char* value);
void stringholder_delete(StringHolder* self);
const char* stringholder_get(StringHolder* self);
const char* stringholder_describe(StringHolder* self);

#ifdef __cplusplus
}

// Full class definition - for hicc code generation
class IntHolder {
    int value_;
public:
    explicit IntHolder(int value);
    ~IntHolder();
    int get() const;
    const char* describe() const;
};

class DoubleHolder {
    double value_;
public:
    explicit DoubleHolder(double value);
    ~DoubleHolder();
    double get() const;
    const char* describe() const;
};

class StringHolder {
    char* value_;
    int length_;
public:
    explicit StringHolder(const char* value);
    ~StringHolder();
    const char* get() const;
    const char* describe() const;
};

#endif
