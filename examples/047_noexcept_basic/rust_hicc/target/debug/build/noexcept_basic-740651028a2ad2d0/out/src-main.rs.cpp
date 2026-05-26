#include <hicc/hicc.hpp>
#line 0 R"(src/main.rs)"
#line 2
#include <utility>
    #include <cstdint>


    inline int noexcept_add(int a, int b) noexcept {
        return a + b;
    }

    inline int noexcept_multiply(int a, int b) noexcept {
        return a * b;
    }

    inline int conditional_abs(int value) noexcept {
        return value >= 0 ? value : -value;
    }


    class NoexceptMover {
        int value_;
    public:
        NoexceptMover(int value) : value_(value) {}
        ~NoexceptMover() {}
        NoexceptMover(NoexceptMover&& other) noexcept : value_(other.value_) {
            other.value_ = 0;
        }
        NoexceptMover& operator=(NoexceptMover&& other) noexcept {
            if (this != &other) {
                value_ = other.value_;
                other.value_ = 0;
            }
            return *this;
        }
        int get_value() const { return value_; }
        NoexceptMover(const NoexceptMover&) = delete;
        NoexceptMover& operator=(const NoexceptMover&) = delete;
    };


    NoexceptMover* noexcept_mover_new(int value) {
        return new NoexceptMover(value);
    }

    void noexcept_mover_delete(NoexceptMover* self) {
        delete self;
    }


    NoexceptMover* noexcept_mover_move(NoexceptMover* other) noexcept {
        if (other) {
            return new NoexceptMover(std::move(*other));
        }
        return nullptr;
    }
#line 58
 struct NoexceptMover_58;
#line 58
namespace hicc { template<> struct MethodsType<NoexceptMover, void> { typedef NoexceptMover_58 methods_type; }; }
#line 58
 struct NoexceptMover_58 {
#line 58
typedef NoexceptMover Self; typedef void SelfContainer; typedef NoexceptMover_58 SelfMethods;
#line 60
static void _hicc_test_60() { int (Self::* _60)() const = &Self::get_value; (void)_60; }
#line 60
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((int (Self::*)() const)&Self::get_value));
#line 58
};
#line 66
EXPORT_METHODS_BEG(noexcept_basic) {
#line 70
static void _hicc_test_70() { int (* _70)(int a, int b) noexcept = &noexcept_add; (void)_70; }
#line 70
EXPORT_METHOD_IN(void, ExportMethods, ((int (*)(int a, int b) noexcept)&noexcept_add));
#line 73
static void _hicc_test_73() { int (* _73)(int a, int b) noexcept = &noexcept_multiply; (void)_73; }
#line 73
EXPORT_METHOD_IN(void, ExportMethods, ((int (*)(int a, int b) noexcept)&noexcept_multiply));
#line 76
static void _hicc_test_76() { int (* _76)(int value) noexcept = &conditional_abs; (void)_76; }
#line 76
EXPORT_METHOD_IN(void, ExportMethods, ((int (*)(int value) noexcept)&conditional_abs));
#line 79
static void _hicc_test_79() { NoexceptMover* (* _79)(int value) = &noexcept_mover_new; (void)_79; }
#line 79
EXPORT_METHOD_IN(void, ExportMethods, ((NoexceptMover* (*)(int value))&noexcept_mover_new));
#line 82
static void _hicc_test_82() { void (* _82)(NoexceptMover* self) = &noexcept_mover_delete; (void)_82; }
#line 82
EXPORT_METHOD_IN(void, ExportMethods, ((void (*)(NoexceptMover* self))&noexcept_mover_delete));
#line 85
static void _hicc_test_85() { NoexceptMover* (* _85)(NoexceptMover* other) noexcept = &noexcept_mover_move; (void)_85; }
#line 85
EXPORT_METHOD_IN(void, ExportMethods, ((NoexceptMover* (*)(NoexceptMover* other) noexcept)&noexcept_mover_move));
#line 66
} EXPORT_METHODS_END();

