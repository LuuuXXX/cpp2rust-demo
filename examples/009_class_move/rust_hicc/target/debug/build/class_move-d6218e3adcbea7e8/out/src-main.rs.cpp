#include <hicc/hicc.hpp>
#line 0 R"(src/main.rs)"
#line 2
#include <iostream>
    #include <cstring>

    class UniqueVector {
        int* data;
        int size;
    public:
        UniqueVector() : data(nullptr), size(0) {}
        UniqueVector(int* data, int size) : size(size) {
            this->data = new int[size];
            std::memcpy(this->data, data, size * sizeof(int));
        }
        ~UniqueVector() {
            delete[] data;
        }
        UniqueVector(UniqueVector&& other) noexcept : data(other.data), size(other.size) {
            other.data = nullptr;
            other.size = 0;
        }
        UniqueVector& operator=(UniqueVector&& other) noexcept {
            if (this != &other) {
                delete[] data;
                data = other.data;
                size = other.size;
                other.data = nullptr;
                other.size = 0;
            }
            return *this;
        }
        int get(int index) const {
            if (index >= 0 && index < size) {
                return data[index];
            }
            return 0;
        }
        void set(int index, int value) {
            if (index >= 0 && index < size) {
                data[index] = value;
            }
        }
        int getSize() const {
            return size;
        }
        void moveFrom(UniqueVector& src) {
            delete[] data;
            data = src.data;
            size = src.size;
            src.data = nullptr;
            src.size = 0;
        }
    };

    UniqueVector* unique_vector_new() {
        return new UniqueVector();
    }

    UniqueVector* unique_vector_new_with_data(int* data, int size) {
        return new UniqueVector(data, size);
    }

    void unique_vector_delete(UniqueVector* self) {
        delete self;
    }

    void unique_vector_move(UniqueVector* dest, UniqueVector* src) {
        std::cout << "Moving UniqueVector: " << src->getSize() << " -> " << dest->getSize() << std::endl;
        dest->moveFrom(*src);
    }
#line 73
 struct UniqueVector_73;
#line 73
namespace hicc { template<> struct MethodsType<UniqueVector, void> { typedef UniqueVector_73 methods_type; }; }
#line 73
 struct UniqueVector_73 {
#line 73
typedef UniqueVector Self; typedef void SelfContainer; typedef UniqueVector_73 SelfMethods;
#line 75
static void _hicc_test_75() { int (Self::* _75)(int index) const = &Self::get; (void)_75; }
#line 75
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((int (Self::*)(int index) const)&Self::get));
#line 78
static void _hicc_test_78() { void (Self::* _78)(int index, int value) = &Self::set; (void)_78; }
#line 78
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((void (Self::*)(int index, int value))&Self::set));
#line 81
static void _hicc_test_81() { int (Self::* _81)() const = &Self::getSize; (void)_81; }
#line 81
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((int (Self::*)() const)&Self::getSize));
#line 73
};
#line 87
EXPORT_METHODS_BEG(class_move) {
#line 91
static void _hicc_test_91() { UniqueVector* (* _91)() = &unique_vector_new; (void)_91; }
#line 91
EXPORT_METHOD_IN(void, ExportMethods, ((UniqueVector* (*)())&unique_vector_new));
#line 94
static void _hicc_test_94() { UniqueVector* (* _94)(int* data, int size) = &unique_vector_new_with_data; (void)_94; }
#line 94
EXPORT_METHOD_IN(void, ExportMethods, ((UniqueVector* (*)(int* data, int size))&unique_vector_new_with_data));
#line 97
static void _hicc_test_97() { void (* _97)(UniqueVector* self) = &unique_vector_delete; (void)_97; }
#line 97
EXPORT_METHOD_IN(void, ExportMethods, ((void (*)(UniqueVector* self))&unique_vector_delete));
#line 100
static void _hicc_test_100() { void (* _100)(UniqueVector* dest, UniqueVector* src) = &unique_vector_move; (void)_100; }
#line 100
EXPORT_METHOD_IN(void, ExportMethods, ((void (*)(UniqueVector* dest, UniqueVector* src))&unique_vector_move));
#line 87
} EXPORT_METHODS_END();

