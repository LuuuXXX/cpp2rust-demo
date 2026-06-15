#include "inheritance_single.h"
#include <iostream>
#include <cstring>
#include <string>

Animal::Animal(const char* n) : name(n) {}

Animal::~Animal() {}

const char* Animal::getName() const {
    return name.c_str();
}

void Animal::speak() const {
    std::cout << name << " makes a sound" << std::endl;
}

Dog::Dog(const char* n) : Animal(n) {}

Dog::~Dog() {}

void Dog::bark() const {
    std::cout << name << " barks: Woof! Woof!" << std::endl;
}

void Dog::speak() const {
    std::cout << name << " barks: Woof! Woof!" << std::endl;
}
