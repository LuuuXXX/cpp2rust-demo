#pragma once

#include <string>

class Animal {
protected:
    std::string name;
public:
    Animal(const char* n);
    virtual ~Animal();
    const char* getName() const;
    virtual void speak() const;
};

class Dog : public Animal {
public:
    Dog(const char* n);
    ~Dog() override;
    void bark() const;
    void speak() const override;
};
