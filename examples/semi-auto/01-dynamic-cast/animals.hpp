#pragma once
// animals.hpp — semi-auto/01-dynamic-cast
//
// Class hierarchy for demonstrating dynamic_cast semi-auto workflow.

class Animal {
public:
    virtual ~Animal() {}
    virtual const char* speak() const = 0;
    virtual const char* kind()  const = 0;
};

class Dog : public Animal {
public:
    explicit Dog(const char* name);
    ~Dog();

    const char* speak() const override;  // "Woof"
    const char* kind()  const override;  // "Dog"
    const char* name()  const;

    /// Fetch: Dog-specific behaviour (not on Animal).
    void fetch(const char* item) const;
};

class Cat : public Animal {
public:
    explicit Cat(const char* name);
    ~Cat();

    const char* speak() const override;  // "Meow"
    const char* kind()  const override;  // "Cat"
    const char* name()  const;

    /// Purr: Cat-specific behaviour (not on Animal).
    void purr() const;
};
