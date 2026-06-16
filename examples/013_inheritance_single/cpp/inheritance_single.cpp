#include "inheritance_single.h"

namespace inheritance_single_ns {

Animal::Animal(std::string name) : name_(std::move(name)) {}

Animal::~Animal() = default;

const std::string& Animal::name() const {
    return name_;
}

std::string Animal::speak() const {
    return name_ + " makes a sound";
}

Dog::Dog(std::string name) : Animal(std::move(name)) {}

Dog::~Dog() = default;

std::string Dog::bark() const {
    return name_ + " barks: Woof! Woof!";
}

std::string Dog::speak() const {
    return name_ + " barks: Woof! Woof!";
}

int inheritance_single_anchor() {
    return 0;
}

} // namespace inheritance_single_ns
