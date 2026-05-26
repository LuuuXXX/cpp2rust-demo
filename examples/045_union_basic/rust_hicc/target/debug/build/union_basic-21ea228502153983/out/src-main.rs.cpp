#include <hicc/hicc.hpp>
#line 0 R"(src/main.rs)"
#line 2
#include <cstring>
    #include <cstdint>


    const int VALUE_TYPE_INT = 0;
    const int VALUE_TYPE_FLOAT = 1;
    const int VALUE_TYPE_STRING = 2;


    class Variant {
        int type_;
        union {
            int int_value_;
            float float_value_;
            char string_buffer_[64];
        } data_;
    public:
        Variant();
        ~Variant();
        int get_type() const { return type_; }
        void set_int(int value);
        void set_float(float value);
        void set_string(const char* value);
        int get_int() const;
        float get_float() const;
        const char* get_string() const;
    };


    struct IntFloatUnion {
        union {
            int int_value;
            float float_value;
        } data;
    };


    Variant* variant_new_int(int value) {
        auto* v = new Variant();
        v->set_int(value);
        return v;
    }

    Variant* variant_new_float(float value) {
        auto* v = new Variant();
        v->set_float(value);
        return v;
    }

    Variant* variant_new_string(const char* value) {
        auto* v = new Variant();
        v->set_string(value);
        return v;
    }

    void variant_delete(Variant* self) {
        delete self;
    }

    int variant_get_type(const Variant* self) {
        if (self) return self->get_type();
        return VALUE_TYPE_INT;
    }

    int variant_get_int(const Variant* self) {
        if (self) return self->get_int();
        return 0;
    }

    float variant_get_float(const Variant* self) {
        if (self) return self->get_float();
        return 0.0f;
    }

    const char* variant_get_string(const Variant* self) {
        if (self) return self->get_string();
        return "";
    }

    void variant_set_int(Variant* self, int value) {
        if (self) self->set_int(value);
    }

    void variant_set_float(Variant* self, float value) {
        if (self) self->set_float(value);
    }

    void variant_set_string(Variant* self, const char* value) {
        if (self) self->set_string(value);
    }


    int union_get_int(const IntFloatUnion* u) {
        if (u) return u->data.int_value;
        return 0;
    }

    float union_get_float(const IntFloatUnion* u) {
        if (u) return u->data.float_value;
        return 0.0f;
    }

    void union_set_int(IntFloatUnion* u, int value) {
        if (u) u->data.int_value = value;
    }

    void union_set_float(IntFloatUnion* u, float value) {
        if (u) u->data.float_value = value;
    }
#line 114
 struct Variant_114;
#line 114
namespace hicc { template<> struct MethodsType<Variant, void> { typedef Variant_114 methods_type; }; }
#line 114
 struct Variant_114 {
#line 114
typedef Variant Self; typedef void SelfContainer; typedef Variant_114 SelfMethods;
#line 116
static void _hicc_test_116() { int (Self::* _116)() const = &Self::get_type; (void)_116; }
#line 116
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((int (Self::*)() const)&Self::get_type));
#line 119
static void _hicc_test_119() { void (Self::* _119)(int value) = &Self::set_int; (void)_119; }
#line 119
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((void (Self::*)(int value))&Self::set_int));
#line 122
static void _hicc_test_122() { void (Self::* _122)(float value) = &Self::set_float; (void)_122; }
#line 122
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((void (Self::*)(float value))&Self::set_float));
#line 125
static void _hicc_test_125() { void (Self::* _125)(const char* value) = &Self::set_string; (void)_125; }
#line 125
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((void (Self::*)(const char* value))&Self::set_string));
#line 128
static void _hicc_test_128() { int (Self::* _128)() const = &Self::get_int; (void)_128; }
#line 128
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((int (Self::*)() const)&Self::get_int));
#line 131
static void _hicc_test_131() { float (Self::* _131)() const = &Self::get_float; (void)_131; }
#line 131
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((float (Self::*)() const)&Self::get_float));
#line 134
static void _hicc_test_134() { const char* (Self::* _134)() const = &Self::get_string; (void)_134; }
#line 134
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((const char* (Self::*)() const)&Self::get_string));
#line 114
};
#line 140
EXPORT_METHODS_BEG(union_basic) {
#line 145
static void _hicc_test_145() { Variant* (* _145)(int value) = &variant_new_int; (void)_145; }
#line 145
EXPORT_METHOD_IN(void, ExportMethods, ((Variant* (*)(int value))&variant_new_int));
#line 148
static void _hicc_test_148() { Variant* (* _148)(float value) = &variant_new_float; (void)_148; }
#line 148
EXPORT_METHOD_IN(void, ExportMethods, ((Variant* (*)(float value))&variant_new_float));
#line 151
static void _hicc_test_151() { Variant* (* _151)(const char* value) = &variant_new_string; (void)_151; }
#line 151
EXPORT_METHOD_IN(void, ExportMethods, ((Variant* (*)(const char* value))&variant_new_string));
#line 154
static void _hicc_test_154() { void (* _154)(Variant* self) = &variant_delete; (void)_154; }
#line 154
EXPORT_METHOD_IN(void, ExportMethods, ((void (*)(Variant* self))&variant_delete));
#line 157
static void _hicc_test_157() { int (* _157)(const Variant* self) = &variant_get_type; (void)_157; }
#line 157
EXPORT_METHOD_IN(void, ExportMethods, ((int (*)(const Variant* self))&variant_get_type));
#line 160
static void _hicc_test_160() { int (* _160)(const Variant* self) = &variant_get_int; (void)_160; }
#line 160
EXPORT_METHOD_IN(void, ExportMethods, ((int (*)(const Variant* self))&variant_get_int));
#line 163
static void _hicc_test_163() { float (* _163)(const Variant* self) = &variant_get_float; (void)_163; }
#line 163
EXPORT_METHOD_IN(void, ExportMethods, ((float (*)(const Variant* self))&variant_get_float));
#line 166
static void _hicc_test_166() { const char* (* _166)(const Variant* self) = &variant_get_string; (void)_166; }
#line 166
EXPORT_METHOD_IN(void, ExportMethods, ((const char* (*)(const Variant* self))&variant_get_string));
#line 169
static void _hicc_test_169() { void (* _169)(Variant* self, int value) = &variant_set_int; (void)_169; }
#line 169
EXPORT_METHOD_IN(void, ExportMethods, ((void (*)(Variant* self, int value))&variant_set_int));
#line 172
static void _hicc_test_172() { void (* _172)(Variant* self, float value) = &variant_set_float; (void)_172; }
#line 172
EXPORT_METHOD_IN(void, ExportMethods, ((void (*)(Variant* self, float value))&variant_set_float));
#line 175
static void _hicc_test_175() { void (* _175)(Variant* self, const char* value) = &variant_set_string; (void)_175; }
#line 175
EXPORT_METHOD_IN(void, ExportMethods, ((void (*)(Variant* self, const char* value))&variant_set_string));
#line 178
static void _hicc_test_178() { int (* _178)(const IntFloatUnion* u) = &union_get_int; (void)_178; }
#line 178
EXPORT_METHOD_IN(void, ExportMethods, ((int (*)(const IntFloatUnion* u))&union_get_int));
#line 181
static void _hicc_test_181() { float (* _181)(const IntFloatUnion* u) = &union_get_float; (void)_181; }
#line 181
EXPORT_METHOD_IN(void, ExportMethods, ((float (*)(const IntFloatUnion* u))&union_get_float));
#line 184
static void _hicc_test_184() { void (* _184)(IntFloatUnion* u, int value) = &union_set_int; (void)_184; }
#line 184
EXPORT_METHOD_IN(void, ExportMethods, ((void (*)(IntFloatUnion* u, int value))&union_set_int));
#line 187
static void _hicc_test_187() { void (* _187)(IntFloatUnion* u, float value) = &union_set_float; (void)_187; }
#line 187
EXPORT_METHOD_IN(void, ExportMethods, ((void (*)(IntFloatUnion* u, float value))&union_set_float));
#line 140
} EXPORT_METHODS_END();

