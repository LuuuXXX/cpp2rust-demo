#include <hicc/hicc.hpp>
#line 0 R"(src/main.rs)"
#line 2
#include <cstring>
    #include <new>





    struct SimpleValue {
        int value;
    };


    class Buffer {
        char* data_;
        size_t capacity_;
        size_t constructed_size_;
    public:
        explicit Buffer(size_t capacity) : data_(nullptr), capacity_(capacity), constructed_size_(0) {
            if (capacity_ > 0) {
                data_ = new char[capacity_];
                std::memset(data_, 0, capacity_);
            }
        }
        ~Buffer() {
            if (data_) {
                delete[] data_;
                data_ = nullptr;
            }
        }
        Buffer(const Buffer&) = delete;
        Buffer& operator=(const Buffer&) = delete;
        void* data() { return static_cast<void*>(data_); }
        size_t capacity() const { return capacity_; }
        size_t size() const { return constructed_size_; }
        void* construct(size_t offset) {
            if (offset < capacity_) {
                constructed_size_ = offset + sizeof(SimpleValue);
                return static_cast<void*>(data_ + offset);
            }
            return nullptr;
        }
    };


    class VectorBuffer {
        char* data_;
        size_t capacity_;
        size_t size_;
        size_t element_size_;
    public:
        explicit VectorBuffer(size_t capacity, size_t elem_size)
            : data_(nullptr), capacity_(capacity), size_(0), element_size_(elem_size) {
            if (capacity_ > 0) {
                data_ = new char[capacity_ * element_size_];
                std::memset(data_, 0, capacity_ * element_size_);
            }
        }
        ~VectorBuffer() {
            destroy_all();
            if (data_) {
                delete[] data_;
                data_ = nullptr;
            }
        }
        VectorBuffer(const VectorBuffer&) = delete;
        VectorBuffer& operator=(const VectorBuffer&) = delete;
        void* data() { return static_cast<void*>(data_); }
        size_t element_size() const { return element_size_; }
        void destroy_all() {
            size_ = 0;
            if (data_) {
                std::memset(data_, 0, capacity_ * element_size_);
            }
        }
    };


    Buffer* buffer_new(size_t capacity) {
        return new Buffer(capacity);
    }

    void buffer_delete(Buffer* self_) {
        if (self_) delete self_;
    }

    VectorBuffer* vector_buffer_new(size_t capacity) {
        return new VectorBuffer(capacity, sizeof(SimpleValue));
    }

    void vector_buffer_delete(VectorBuffer* self_) {
        if (self_) {
            self_->destroy_all();
            delete self_;
        }
    }
#line 100
 struct Buffer_100;
#line 100
namespace hicc { template<> struct MethodsType<Buffer, void> { typedef Buffer_100 methods_type; }; }
#line 115
 struct VectorBuffer_115;
#line 115
namespace hicc { template<> struct MethodsType<VectorBuffer, void> { typedef VectorBuffer_115 methods_type; }; }
#line 100
 struct Buffer_100 {
#line 100
typedef Buffer Self; typedef void SelfContainer; typedef Buffer_100 SelfMethods;
#line 102
static void _hicc_test_102() { void* (Self::* _102)() = &Self::data; (void)_102; }
#line 102
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((void* (Self::*)())&Self::data));
#line 105
static void _hicc_test_105() { size_t (Self::* _105)() const = &Self::capacity; (void)_105; }
#line 105
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((size_t (Self::*)() const)&Self::capacity));
#line 108
static void _hicc_test_108() { size_t (Self::* _108)() const = &Self::size; (void)_108; }
#line 108
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((size_t (Self::*)() const)&Self::size));
#line 111
static void _hicc_test_111() { void* (Self::* _111)(size_t offset) = &Self::construct; (void)_111; }
#line 111
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((void* (Self::*)(size_t offset))&Self::construct));
#line 100
};
#line 115
 struct VectorBuffer_115 {
#line 115
typedef VectorBuffer Self; typedef void SelfContainer; typedef VectorBuffer_115 SelfMethods;
#line 117
static void _hicc_test_117() { void* (Self::* _117)() = &Self::data; (void)_117; }
#line 117
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((void* (Self::*)())&Self::data));
#line 120
static void _hicc_test_120() { size_t (Self::* _120)() const = &Self::element_size; (void)_120; }
#line 120
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((size_t (Self::*)() const)&Self::element_size));
#line 123
static void _hicc_test_123() { void (Self::* _123)() = &Self::destroy_all; (void)_123; }
#line 123
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((void (Self::*)())&Self::destroy_all));
#line 115
};
#line 129
EXPORT_METHODS_BEG(placement_new) {
#line 132
static void _hicc_test_132() { Buffer* (* _132)(size_t capacity) = &buffer_new; (void)_132; }
#line 132
EXPORT_METHOD_IN(void, ExportMethods, ((Buffer* (*)(size_t capacity))&buffer_new));
#line 134
static void _hicc_test_134() { void (* _134)(Buffer* self_) = &buffer_delete; (void)_134; }
#line 134
EXPORT_METHOD_IN(void, ExportMethods, ((void (*)(Buffer* self_))&buffer_delete));
#line 138
static void _hicc_test_138() { VectorBuffer* (* _138)(size_t capacity) = &vector_buffer_new; (void)_138; }
#line 138
EXPORT_METHOD_IN(void, ExportMethods, ((VectorBuffer* (*)(size_t capacity))&vector_buffer_new));
#line 140
static void _hicc_test_140() { void (* _140)(VectorBuffer* self_) = &vector_buffer_delete; (void)_140; }
#line 140
EXPORT_METHOD_IN(void, ExportMethods, ((void (*)(VectorBuffer* self_))&vector_buffer_delete));
#line 129
} EXPORT_METHODS_END();

