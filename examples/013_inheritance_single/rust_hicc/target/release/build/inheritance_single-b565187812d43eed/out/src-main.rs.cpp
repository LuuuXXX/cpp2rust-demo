#include <hicc/hicc.hpp>
#line 0 R"(src/main.rs)"
#line 2
#include <string>
    #include <iostream>

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

    Animal* animal_new(const char* name) {
        return new Animal(name);
    }

    void animal_delete(Animal* self) {
        delete self;
    }

    Dog* dog_new(const char* name) {
        return new Dog(name);
    }

    void dog_delete(Dog* self) {
        delete self;
    }
#line 65
 struct Animal_65;
#line 65
namespace hicc { template<> struct MethodsType<Animal, void> { typedef Animal_65 methods_type; }; }
#line 65
 struct Animal_65 {
#line 65
typedef Animal Self; typedef void SelfContainer; typedef Animal_65 SelfMethods;
#line 67
static void _hicc_test_67() { const char* (Self::* _67)() const = &Self::getName; (void)_67; }
#line 67
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((const char* (Self::*)() const)&Self::getName));
#line 70
static void _hicc_test_70() { void (Self::* _70)() const = &Self::speak; (void)_70; }
#line 70
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((void (Self::*)() const)&Self::speak));
#line 65
};
#line 76
 struct Dog_76;
#line 76
namespace hicc { template<> struct MethodsType<Dog, void> { typedef Dog_76 methods_type; }; }
#line 76
 struct Dog_76 {
#line 76
typedef Dog Self; typedef void SelfContainer; typedef Dog_76 SelfMethods;
#line 78
static void _hicc_test_78() { const char* (Self::* _78)() const = &Self::getName; (void)_78; }
#line 78
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((const char* (Self::*)() const)&Self::getName));
#line 81
static void _hicc_test_81() { void (Self::* _81)() const = &Self::speak; (void)_81; }
#line 81
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((void (Self::*)() const)&Self::speak));
#line 84
static void _hicc_test_84() { void (Self::* _84)() const = &Self::bark; (void)_84; }
#line 84
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((void (Self::*)() const)&Self::bark));
#line 76
};
#line 90
EXPORT_METHODS_BEG(inheritance_single) {
#line 95
static void _hicc_test_95() { Animal* (* _95)(const char* name) = &animal_new; (void)_95; }
#line 95
EXPORT_METHOD_IN(void, ExportMethods, ((Animal* (*)(const char* name))&animal_new));
#line 98
static void _hicc_test_98() { void (* _98)(Animal* self) = &animal_delete; (void)_98; }
#line 98
EXPORT_METHOD_IN(void, ExportMethods, ((void (*)(Animal* self))&animal_delete));
#line 101
static void _hicc_test_101() { Dog* (* _101)(const char* name) = &dog_new; (void)_101; }
#line 101
EXPORT_METHOD_IN(void, ExportMethods, ((Dog* (*)(const char* name))&dog_new));
#line 104
static void _hicc_test_104() { void (* _104)(Dog* self) = &dog_delete; (void)_104; }
#line 104
EXPORT_METHOD_IN(void, ExportMethods, ((void (*)(Dog* self))&dog_delete));
#line 90
} EXPORT_METHODS_END();

