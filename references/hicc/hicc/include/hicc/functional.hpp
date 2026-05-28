#ifndef HICC_FUNCTIONAL_H
#define HICC_FUNCTIONAL_H

#include <functional>
#include <memory>

#include "types.hpp"

namespace hicc {

struct AbiFunction {
    const void* call;
    const void* destroy;
    const void* ctx;
};

template <class M, class R, class... ArgTypes>
class CxxFunction {
    std::function<R(ArgTypes...)> func;
    CxxFunction() = delete;
    CxxFunction(const CxxFunction&) = delete;

public:
    CxxFunction(const std::function<R(ArgTypes...)>& func) : func(func) {}
    AbiFunction into() const
    {
        return AbiFunction{
            /* call */     (const void*)call,
            /* destroy */  (const void*)destroy,
            /* ctx */      (const void*)this
        };
    }

    static bool is_cxx_function(const AbiFunction& f) { return f.destroy == (const void*)destroy; }

    static std::function<R(ArgTypes...)> into(const AbiFunction& f)
    {
        assert(is_cxx_function(f));
        return ((CxxFunction<M, R, ArgTypes...>*)(f.ctx))->func;
    }

private:
    ~CxxFunction() {}
    static typename AbiValue<R, M>::output_type call(const void* ctx,
                                                     typename AbiValue<ArgTypes, M>::input_type... args)
    {
        CxxFunction<M, R, ArgTypes...>* func = (CxxFunction<M, R, ArgTypes...>*)ctx;
        return AbiValue<R, M>::into_with([&]() { return func->func(AbiValue<ArgTypes, M>::from(std::move(args))...); });
    }
    static void destroy(const void* ctx)
    {
        CxxFunction<M, R, ArgTypes...>* func = (CxxFunction<M, R, ArgTypes...>*)ctx;
        delete func;
    }
};

template <class M, class R, class... ArgTypes>
static inline AbiFunction make_abi_function(const std::function<R(ArgTypes...)>& func)
{
    auto f = new (std::nothrow) CxxFunction<M, R, ArgTypes...>(func);
    if (f) {
        return f->into();
    }
    return AbiFunction{
            /* call */     0,
            /* destroy */  0,
            /* ctx */      0
        };
}

template <class M, class R, class... ArgTypes>
class ExternFunction {
    ExternFunction(const ExternFunction&) = delete;
    ExternFunction() = delete;
    void (*destroy)(const void* ctx) = 0;

public:
    ExternFunction(AbiFunction func)
    {
        call = (R(*)(const void*, ArgTypes...))func.call;
        destroy = (void (*)(const void*))func.destroy;
        ctx = func.ctx;
    }
    ~ExternFunction()
    {
        if (destroy) {
            destroy(ctx);
        }
    }

public:
    R (*call)(const void* ctx, ArgTypes...) = 0;
    const void* ctx = 0;
};

template <class M, class R, class... ArgTypes>
static inline std::function<R(ArgTypes...)> make_cxx_function(AbiFunction func)
{
    if (CxxFunction<M, R, ArgTypes...>::is_cxx_function(func)) {
        return CxxFunction<M, R, ArgTypes...>::into(func);
    }
    auto shared = std::make_shared<ExternFunction<M, R, ArgTypes...>>(func);
    return [=](ArgTypes... args) -> R { return shared->call(shared->ctx, args...); };
}

template <class M, class R, class... ArgTypes>
struct PlainType<std::function<R(ArgTypes...)>, M> {
    typedef AbiFunction output_type;
    typedef AbiFunction input_type;
    static output_type into(std::function<R(ArgTypes...)> f) { return make_abi_function<M>(f); }
    static std::function<R(ArgTypes...)> from(input_type arg) { return make_cxx_function<M, R, ArgTypes...>(arg); }
};

// 如果是const引用参数，无需释放资源.
// 如果返回的是const引用，拷贝资源
// 如果是指针，一般只是在c++内部使用，不做任何特殊处理，其他语言只是存储指针不能调用std::function
template <class M, class R, class... ArgTypes>
struct AbiValue<const std::function<R(ArgTypes...)>&, M> {
    typedef AbiFunction* input_type;
    typedef AbiFunction output_type;
    static std::function<R(ArgTypes...)> from(input_type func)
    {
        if (CxxFunction<M, R, ArgTypes...>::is_cxx_function(*func)) {
            return CxxFunction<M, R, ArgTypes...>::into(*func);
        }
        return make_cxx_function<M, R, ArgTypes...>(AbiFunction{
            /* call */ func->call,
            /* destroy */  0,
            /* ctx */ func->ctx,
        });
    }
    static std::function<R(ArgTypes...)> from_with(std::function<input_type()> fun) { return from(fun()); }
    static output_type into(const std::function<R(ArgTypes...)>& f) { return make_abi_function<M>(f); }
    static output_type into_with(const std::function<const std::function<R(ArgTypes...)>&()>& fun)
    {
        return into(fun());
    }
};

}    // namespace hicc

#endif
