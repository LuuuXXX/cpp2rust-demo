#include <hicc/hicc.hpp>
#line 0 R"(src/main.rs)"
#line 2
#include <iostream>
    #include <cstring>

    class Buffer {
        int* data;
        int size;
    public:
        Buffer() : data(nullptr), size(0) {}
        explicit Buffer(int sz) : size(sz) {
            data = new int[sz];
            std::memset(data, 0, sz * sizeof(int));
        }
        Buffer(const Buffer& other) : size(other.size) {
            data = new int[other.size];
            std::memcpy(data, other.data, other.size * sizeof(int));
        }
        ~Buffer() {
            delete[] data;
        }
        void set(int index, int value) {
            if (index >= 0 && index < size) {
                data[index] = value;
            }
        }
        int get(int index) const {
            if (index >= 0 && index < size) {
                return data[index];
            }
            return 0;
        }
        int getSize() const {
            return size;
        }
    };

    Buffer* buffer_new() {
        return new Buffer();
    }

    Buffer* buffer_new_with_size(int size) {
        return new Buffer(size);
    }

    Buffer* buffer_new_copy(const Buffer* other) {
        return new Buffer(*other);
    }

    void buffer_delete(Buffer* self) {
        delete self;
    }
#line 55
 struct Buffer_55;
#line 55
namespace hicc { template<> struct MethodsType<Buffer, void> { typedef Buffer_55 methods_type; }; }
#line 55
 struct Buffer_55 {
#line 55
typedef Buffer Self; typedef void SelfContainer; typedef Buffer_55 SelfMethods;
#line 57
static void _hicc_test_57() { void (Self::* _57)(int index, int value) = &Self::set; (void)_57; }
#line 57
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((void (Self::*)(int index, int value))&Self::set));
#line 60
static void _hicc_test_60() { int (Self::* _60)(int index) const = &Self::get; (void)_60; }
#line 60
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((int (Self::*)(int index) const)&Self::get));
#line 63
static void _hicc_test_63() { int (Self::* _63)() const = &Self::getSize; (void)_63; }
#line 63
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((int (Self::*)() const)&Self::getSize));
#line 55
};
#line 69
EXPORT_METHODS_BEG(class_copy) {
#line 73
static void _hicc_test_73() { Buffer* (* _73)() = &buffer_new; (void)_73; }
#line 73
EXPORT_METHOD_IN(void, ExportMethods, ((Buffer* (*)())&buffer_new));
#line 76
static void _hicc_test_76() { Buffer* (* _76)(int size) = &buffer_new_with_size; (void)_76; }
#line 76
EXPORT_METHOD_IN(void, ExportMethods, ((Buffer* (*)(int size))&buffer_new_with_size));
#line 79
static void _hicc_test_79() { Buffer* (* _79)(const Buffer* other) = &buffer_new_copy; (void)_79; }
#line 79
EXPORT_METHOD_IN(void, ExportMethods, ((Buffer* (*)(const Buffer* other))&buffer_new_copy));
#line 82
static void _hicc_test_82() { void (* _82)(Buffer* self) = &buffer_delete; (void)_82; }
#line 82
EXPORT_METHOD_IN(void, ExportMethods, ((void (*)(Buffer* self))&buffer_delete));
#line 69
} EXPORT_METHODS_END();

