#pragma once
#include <string>

namespace inheritance_single_ns {

// 基类：地道 C++ 命名空间类（无 extern-C、无 opaque 指针）
class Animal {
public:
    explicit Animal(std::string name);
    virtual ~Animal();

    const std::string& name() const;
    virtual std::string speak() const;

protected:
    std::string name_;
};

// 派生类：单继承 public Animal，覆写 speak()
class Dog : public Animal {
public:
    explicit Dog(std::string name);
    ~Dog() override;

    std::string bark() const;
    std::string speak() const override;
};

// 锚点：libclang 会把全局 C++ 函数误判为 extern-C，
// 以 _anchor 结尾的名字让 detect_idiomatic_mode 容忍并走直出路径。
int inheritance_single_anchor();

} // namespace inheritance_single_ns
