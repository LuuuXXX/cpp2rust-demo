#pragma once

#ifdef __cplusplus
extern "C" {
#endif

struct Animal;
struct Dog;

struct Animal* animal_new(const char* name);
void animal_delete(struct Animal* self);

struct Dog* dog_new(const char* name);
void dog_delete(struct Dog* self);

const char* animal_getName(struct Animal* self);
void animal_speak(struct Animal* self);

void dog_bark(struct Dog* self);
const char* dog_getName(struct Dog* self);
void dog_speak(struct Dog* self);

#ifdef __cplusplus
}

// Full class definition - for hicc code generation
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

#endif
