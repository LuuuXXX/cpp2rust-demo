#include <hicc/hicc.hpp>
#line 0 R"(src/main.rs)"
#line 2
#include <iostream>
    #include <memory>
    #include <cstring>


    class UniqueBuffer {
        std::string data;
    public:
        UniqueBuffer(int sz) : data(sz, '\0') {}
        ~UniqueBuffer() {}
        int getSize() const { return static_cast<int>(data.size()); }
        char* getData() { return data.data(); }
        int useCount() const { return 1; }
    };

    class Processor {
        std::string buffer;
    public:
        Processor() : buffer() {}
        ~Processor() {}
        char* process(const char* input) {
            if (input) {
                buffer = std::string(input) + " [processed]";
            }
            return const_cast<char*>(buffer.c_str());
        }
    };


    UniqueBuffer* uniquebuffer_new(int size) {
        return new UniqueBuffer(size);
    }

    void uniquebuffer_delete(UniqueBuffer* self_) {
        delete self_;
    }

    Processor* processor_new() {
        return new Processor();
    }

    void processor_delete(Processor* self_) {
        delete self_;
    }

    char* processor_process(Processor* self_, const char* input) {
        return self_->process(input);
    }
#line 53
 struct UniqueBuffer_53;
#line 53
namespace hicc { template<> struct MethodsType<UniqueBuffer, void> { typedef UniqueBuffer_53 methods_type; }; }
#line 65
 struct Processor_65;
#line 65
namespace hicc { template<> struct MethodsType<Processor, void> { typedef Processor_65 methods_type; }; }
#line 53
 struct UniqueBuffer_53 {
#line 53
typedef UniqueBuffer Self; typedef void SelfContainer; typedef UniqueBuffer_53 SelfMethods;
#line 55
static void _hicc_test_55() { int (Self::* _55)() const = &Self::getSize; (void)_55; }
#line 55
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((int (Self::*)() const)&Self::getSize));
#line 58
static void _hicc_test_58() { char* (Self::* _58)() = &Self::getData; (void)_58; }
#line 58
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((char* (Self::*)())&Self::getData));
#line 61
static void _hicc_test_61() { int (Self::* _61)() const = &Self::useCount; (void)_61; }
#line 61
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((int (Self::*)() const)&Self::useCount));
#line 53
};
#line 65
 struct Processor_65 {
#line 65
typedef Processor Self; typedef void SelfContainer; typedef Processor_65 SelfMethods;
#line 67
static void _hicc_test_67() { char* (Self::* _67)(const char* input) = &Self::process; (void)_67; }
#line 67
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((char* (Self::*)(const char* input))&Self::process));
#line 65
};
#line 73
EXPORT_METHODS_BEG(unique_ptr) {
#line 76
static void _hicc_test_76() { UniqueBuffer* (* _76)(int size) = &uniquebuffer_new; (void)_76; }
#line 76
EXPORT_METHOD_IN(void, ExportMethods, ((UniqueBuffer* (*)(int size))&uniquebuffer_new));
#line 78
static void _hicc_test_78() { void (* _78)(UniqueBuffer* self_) = &uniquebuffer_delete; (void)_78; }
#line 78
EXPORT_METHOD_IN(void, ExportMethods, ((void (*)(UniqueBuffer* self_))&uniquebuffer_delete));
#line 82
static void _hicc_test_82() { Processor* (* _82)() = &processor_new; (void)_82; }
#line 82
EXPORT_METHOD_IN(void, ExportMethods, ((Processor* (*)())&processor_new));
#line 84
static void _hicc_test_84() { void (* _84)(Processor* self_) = &processor_delete; (void)_84; }
#line 84
EXPORT_METHOD_IN(void, ExportMethods, ((void (*)(Processor* self_))&processor_delete));
#line 73
} EXPORT_METHODS_END();

