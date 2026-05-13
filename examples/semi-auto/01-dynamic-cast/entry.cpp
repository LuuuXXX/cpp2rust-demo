// entry.cpp — semi-auto/01-dynamic-cast
//
// Compiled with clang to produce the capture file consumed by
// `cpp2rust-demo init`.  The implementations are stubs; only the
// declarations matter for AST extraction.
//
// dynamic_cast targets:
//   Animal* → Dog*    (downcast)
//   Animal* → Cat*    (downcast)

#include "animals.hpp"

#include <cstring>

Dog::Dog(const char* /*name*/) {}
Dog::~Dog() {}
const char* Dog::speak() const { return "Woof"; }
const char* Dog::kind()  const { return "Dog"; }
const char* Dog::name()  const { return "dog"; }
void Dog::fetch(const char* /*item*/) const {}

Cat::Cat(const char* /*name*/) {}
Cat::~Cat() {}
const char* Cat::speak() const { return "Meow"; }
const char* Cat::kind()  const { return "Cat"; }
const char* Cat::name()  const { return "cat"; }
void Cat::purr() const {}
