#include "inheritance_single.h"
#include <iostream>
#include <cstring>
#include <string>

// Animal class implementations
Animal::Animal(const char* n) : name(n) {}

Animal::~Animal() {}

const char* Animal::getName() const {
    return name.c_str();
}

void Animal::speak() const {
    std::cout << name << " makes a sound" << std::endl;
}

// Dog class implementations
Dog::Dog(const char* n) : Animal(n) {}

Dog::~Dog() {}

void Dog::bark() const {
    std::cout << name << " barks: Woof! Woof!" << std::endl;
}

void Dog::speak() const {
    std::cout << name << " barks: Woof! Woof!" << std::endl;
}

// FFI wrapper functions
struct Animal* animal_new(const char* name) {
    return new Animal(name);
}

void animal_delete(struct Animal* self) {
    delete self;
}

struct Dog* dog_new(const char* name) {
    return new Dog(name);
}

void dog_delete(struct Dog* self) {
    delete self;
}

const char* animal_getName(struct Animal* self) {
    return self->getName();
}

void animal_speak(struct Animal* self) {
    self->speak();
}

void dog_bark(struct Dog* self) {
    self->bark();
}

const char* dog_getName(struct Dog* self) {
    return self->getName();
}

void dog_speak(struct Dog* self) {
    self->speak();
}
