#ifndef HICC_EXPORT_H
#define HICC_EXPORT_H

#include <cassert>
#include <type_traits>
#include <functional>

#include "exception.hpp"
#include "export_type.hpp"
#include "types.hpp"

namespace hicc {

template <class T>
struct Unsupported;

template <class R, class... ArgTypes>
struct Unsupported<R (*)(ArgTypes..., ...)> {
    static const bool value = true;
};

template <int N, class T>
struct ExportId {
    static const void* id;
};

template <int N, class T>
const void* ExportId<N, T>::id = 0;

template <int N, class M, class T, class R, class... ArgTypes>
static inline typename ExportType<R (*)(ArgTypes...), M>::method_type export_method(std::function<R(ArgTypes...)> func)
{
    static auto ff = func;
#if __cplusplus >= 201703L
    static const void* id = (const void*)&ff;
    return ExportMethod<&id, std::function<R(ArgTypes...)>, M>::type::method;
#elif __cplusplus >= 199711L
    ExportId<N, T>::id = (const void*)&ff;
    return ExportMethod<&ExportId<N, T>::id, std::function<R(ArgTypes...)>, M>::type::method;
#else
#error hicc needs at least a C++11 compliant compiler
#endif
}

template <int N, class M, class T, class R, class... ArgTypes>
static inline typename ExportType<R (*)(ArgTypes...), M>::except_method_type export_except_method(
    std::function<R(ArgTypes...)> func)
{
    static auto ff = func;
#if __cplusplus >= 201703L
    static const void* id = (const void*)&ff;
    return ExportExceptMethod<&id, std::function<R(ArgTypes...)>, M>::type::method;
#elif __cplusplus >= 199711L
    ExportId<N, T>::id = (const void*)&ff;
    return ExportExceptMethod<&ExportId<N, T>::id, std::function<R(ArgTypes...)>, M>::type::method;
#else
#error hicc needs at least a C++11 compliant compiler
#endif
}

#define WHICH_M(M, T) typename std::conditional<std::is_void<M>::value, T, M>::type
template <int N, class MT, class T, class R, class... ArgTypes>
static inline typename ExportType<R (T::*)(ArgTypes...), WHICH_M(MT, T)>::method_type export_method(
    R (T::*mf)(ArgTypes...))
{
    typedef WHICH_M(MT, T) M;
#if __cplusplus >= 201703L
    static const void* id = member_addr(mf);
    return ExportMethod<&id, R (T::*)(ArgTypes...), M>::type::method;
#elif __cplusplus >= 199711L
    ExportId<N, T>::id = member_addr(mf);
    return ExportMethod<&ExportId<N, T>::id, R (T::*)(ArgTypes...), M>::type::method;
#else
#error hicc needs at least a C++11 compliant compiler
#endif
}

template <int N, class MT, class T, class R, class... ArgTypes>
static inline typename ExportType<R (T::*)(ArgTypes...), WHICH_M(MT, T)>::except_method_type export_except_method(
    R (T::*mf)(ArgTypes...))
{
    typedef WHICH_M(MT, T) M;
#if __cplusplus >= 201703L
    static const void* id = member_addr(mf);
    return ExportExceptMethod<&id, R (T::*)(ArgTypes...), M>::type::method;
#elif __cplusplus >= 199711L
    ExportId<N, T>::id = member_addr(mf);
    return ExportExceptMethod<&ExportId<N, T>::id, R (T::*)(ArgTypes...), M>::type::method;
#else
#error hicc needs at least a C++11 compliant compiler
#endif
}

template <int N, class M, class T, class R, class... ArgTypes>
static inline const void* export_method(R (T::*)(ArgTypes..., ...))
{
    static_assert(Unsupported<R (*)(ArgTypes..., ...)>::value);
    return 0;
}

template <int N, class M, class T, class R, class... ArgTypes>
static inline const void* export_except_method(R (T::*)(ArgTypes..., ...))
{
    static_assert(Unsupported<R (*)(ArgTypes..., ...)>::value);
    return 0;
}

template <int N, class M, class T, class R>
static inline const void* export_method(R (T::*)(va_list))
{
    static_assert(Unsupported<R (*)(...)>::value);
    return 0;
}

template <int N, class M, class T, class R>
static inline const void* export_except_method(R (T::*)(va_list))
{
    static_assert(Unsupported<R (*)(...)>::value);
    return 0;
}

template <int N, class MT, class T, class R, class... ArgTypes>
static inline typename ExportType<R (T::*)(ArgTypes...) volatile, WHICH_M(MT, T)>::method_type export_method(
    R (T::*mf)(ArgTypes...) volatile)
{
    typedef WHICH_M(MT, T) M;
#if __cplusplus >= 201703L
    static const void* id = member_addr(mf);
    return ExportMethod<&id, R (T::*)(ArgTypes...) volatile, M>::type::method;
#elif __cplusplus >= 199711L
    ExportId<N, T>::id = member_addr(mf);
    return ExportMethod<&ExportId<N, T>::id, R (T::*)(ArgTypes...) volatile, M>::type::method;
#else
#error hicc needs at least a C++11 compliant compiler
#endif
}

template <int N, class MT, class T, class R, class... ArgTypes>
static inline typename ExportType<R (T::*)(ArgTypes...) volatile, WHICH_M(MT, T)>::except_method_type export_except_method(
    R (T::*mf)(ArgTypes...) volatile)
{
    typedef WHICH_M(MT, T) M;
#if __cplusplus >= 201703L
    static const void* id = member_addr(mf);
    return ExportExceptMethod<&id, R (T::*)(ArgTypes...) volatile, M>::type::method;
#elif __cplusplus >= 199711L
    ExportId<N, T>::id = member_addr(mf);
    return ExportExceptMethod<&ExportId<N, T>::id, R (T::*)(ArgTypes...) volatile, M>::type::method;
#else
#error hicc needs at least a C++11 compliant compiler
#endif
}

template <int N, class M, class T, class R, class... ArgTypes>
static inline const void* export_method(R (T::*)(ArgTypes..., ...) volatile)
{
    static_assert(Unsupported<R (*)(ArgTypes..., ...)>::value);
    return 0;
}

template <int N, class M, class T, class R, class... ArgTypes>
static inline const void* export_except_method(R (T::*)(ArgTypes..., ...) volatile)
{
    static_assert(Unsupported<R (*)(ArgTypes..., ...)>::value);
    return 0;
}

template <int N, class M, class T, class R>
static inline const void* export_method(R (T::*)(va_list) volatile)
{
    static_assert(Unsupported<R (*)(...)>::value);
    return 0;
}

template <int N, class M, class T, class R>
static inline const void* export_except_method(R (T::*)(va_list) volatile)
{
    static_assert(Unsupported<R (*)(...)>::value);
    return 0;
}

template <int N, class MT, class T, class R, class... ArgTypes>
static inline typename ExportType<R (T::*)(ArgTypes...) const, WHICH_M(MT, T)>::method_type export_method(
    R (T::*mf)(ArgTypes...) const)
{
    typedef WHICH_M(MT, T) M;
#if __cplusplus >= 201703L
    static const void* id = member_addr(mf);
    return ExportMethod<&id, R (T::*)(ArgTypes...) const, M>::type::method;
#elif __cplusplus >= 199711L
    ExportId<N, T>::id = member_addr(mf);
    return ExportMethod<&ExportId<N, T>::id, R (T::*)(ArgTypes...) const, M>::type::method;
#else
#error hicc needs at least a C++11 compliant compiler
#endif
}

template <int N, class MT, class T, class R, class... ArgTypes>
static inline typename ExportType<R (T::*)(ArgTypes...) const, WHICH_M(MT, T)>::except_method_type export_except_method(
    R (T::*mf)(ArgTypes...) const)
{
    typedef WHICH_M(MT, T) M;
#if __cplusplus >= 201703L
    static const void* id = member_addr(mf);
    return ExportExceptMethod<&id, R (T::*)(ArgTypes...) const, M>::type::method;
#elif __cplusplus >= 199711L
    ExportId<N, T>::id = member_addr(mf);
    return ExportExceptMethod<&ExportId<N, T>::id, R (T::*)(ArgTypes...) const, M>::type::method;
#else
#error hicc needs at least a C++11 compliant compiler
#endif
}

template <int N, class M, class T, class R, class... ArgTypes>
static inline const void* export_method(R (T::*)(ArgTypes..., ...) const)
{
    static_assert(Unsupported<R (*)(ArgTypes..., ...)>::value);
    return 0;
}

template <int N, class M, class T, class R, class... ArgTypes>
static inline const void* export_except_method(R (T::*)(ArgTypes..., ...) const)
{
    static_assert(Unsupported<R (*)(ArgTypes..., ...)>::value);
    return 0;
}

template <int N, class M, class T, class R>
static inline const void* export_method(R (T::*)(va_list) const)
{
    static_assert(Unsupported<R (*)(...)>::value);
    return 0;
}

template <int N, class M, class T, class R>
static inline const void* export_except_method(R (T::*)(va_list) const)
{
    static_assert(Unsupported<R (*)(...)>::value);
    return 0;
}

template <int N, class MT, class T, class R, class... ArgTypes>
static inline typename ExportType<R (T::*)(ArgTypes...) const volatile, WHICH_M(MT, T)>::method_type export_method(
    R (T::*mf)(ArgTypes...) const volatile)
{
    typedef WHICH_M(MT, T) M;
#if __cplusplus >= 201703L
    static const void* id = member_addr(mf);
    return ExportMethod<&id, R (T::*)(ArgTypes...) const volatile, M>::type::method;
#elif __cplusplus >= 199711L
    ExportId<N, T>::id = member_addr(mf);
    return ExportMethod<&ExportId<N, T>::id, R (T::*)(ArgTypes...) const volatile, M>::type::method;
#else
#error hicc needs at least a C++11 compliant compiler
#endif
}

template <int N, class MT, class T, class R, class... ArgTypes>
static inline typename ExportType<R (T::*)(ArgTypes...) const volatile, WHICH_M(MT, T)>::except_method_type export_except_method(
    R (T::*mf)(ArgTypes...) const volatile)
{
    typedef WHICH_M(MT, T) M;
#if __cplusplus >= 201703L
    static const void* id = member_addr(mf);
    return ExportExceptMethod<&id, R (T::*)(ArgTypes...) const volatile, M>::type::method;
#elif __cplusplus >= 199711L
    ExportId<N, T>::id = member_addr(mf);
    return ExportExceptMethod<&ExportId<N, T>::id, R (T::*)(ArgTypes...) const volatile, M>::type::method;
#else
#error hicc needs at least a C++11 compliant compiler
#endif
}

template <int N, class M, class T, class R, class... ArgTypes>
static inline const void* export_method(R (T::*)(ArgTypes..., ...) const volatile)
{
    static_assert(Unsupported<R (*)(ArgTypes..., ...)>::value);
    return 0;
}

template <int N, class M, class T, class R, class... ArgTypes>
static inline const void* export_except_method(R (T::*)(ArgTypes..., ...) const volatile)
{
    static_assert(Unsupported<R (*)(ArgTypes..., ...)>::value);
    return 0;
}

template <int N, class M, class T, class R>
static inline const void* export_method(R (T::*)(va_list) const volatile)
{
    static_assert(Unsupported<R (*)(...)>::value);
    return 0;
}

template <int N, class M, class T, class R>
static inline const void* export_except_method(R (T::*)(va_list) const volatile)
{
    static_assert(Unsupported<R (*)(...)>::value);
    return 0;
}

template <int N, class MT, class T, class R, class... ArgTypes>
static inline typename ExportType<R (T::*)(ArgTypes...) &&, WHICH_M(MT, T)>::method_type export_method(
    R (T::*mf)(ArgTypes...) &&)
{
    typedef WHICH_M(MT, T) M;
#if __cplusplus >= 201703L
    static const void* addr = member_addr(mf);
    return ExportMethod<&addr, R (T::*)(ArgTypes...)&&, M>::type::method;
#elif __cplusplus >= 199711L
    ExportId<N, T>::id = member_addr(mf);
    return ExportMethod<&ExportId<N, T>::id, R (T::*)(ArgTypes...)&&, M>::type::method;
#else
#error hicc needs at least a C++11 compliant compiler
#endif
}

template <int N, class MT, class T, class R, class... ArgTypes>
static inline typename ExportType<R (T::*)(ArgTypes...) &&, WHICH_M(MT, T)>::except_method_type export_except_method(
    R (T::*mf)(ArgTypes...) &&)
{
    typedef WHICH_M(MT, T) M;
#if __cplusplus >= 201703L
    static const void* id = member_addr(mf);
    return ExportExceptMethod<&id, R (T::*)(ArgTypes...)&&, M>::type::method;
#elif __cplusplus >= 199711L
    ExportId<N, T>::id = member_addr(mf);
    return ExportExceptMethod<&ExportId<N, T>::id, R (T::*)(ArgTypes...)&&, M>::type::method;
#else
#error hicc needs at least a C++11 compliant compiler
#endif
}

template <int N, class M, class T, class R, class... ArgTypes>
static inline const void* export_method(R (T::*)(ArgTypes..., ...) &&)
{
    static_assert(Unsupported<R (*)(ArgTypes..., ...)>::value);
    return 0;
}

template <int N, class M, class T, class R, class... ArgTypes>
static inline const void* export_except_method(R (T::*)(ArgTypes..., ...) &&)
{
    static_assert(Unsupported<R (*)(ArgTypes..., ...)>::value);
    return 0;
}

template <int N, class M, class T, class R>
static inline const void* export_method(R (T::*)(va_list) &&)
{
    static_assert(Unsupported<R (*)(...)>::value);
    return 0;
}

template <int N, class M, class T, class R>
static inline const void* export_except_method(R (T::*)(va_list) &&)
{
    static_assert(Unsupported<R (*)(...)>::value);
    return 0;
}

template <const bool is_abi_function, int N, class T, class F, class M = void>
struct ExportFunction;

template <int N, class M, class T, class R, class... ArgTypes>
struct ExportFunction<false, N, T, R (*)(ArgTypes...), M> {
    static typename ExportType<R (*)(ArgTypes...), M>::method_type export_method(R (*gf)(ArgTypes...))
    {
#if __cplusplus >= 201703L
        static const void* id = (const void*)gf;
        return ExportMethod<&id, R (*)(ArgTypes...), M>::type::method;
#elif __cplusplus >= 199711L
        ExportId<N, T>::id = (const void*)gf;
        return ExportMethod<&ExportId<N, T>::id, R (*)(ArgTypes...), M>::type::method;
#else
#error hicc needs at least a C++11 compliant compiler
#endif
    }
};

template <int N, class M, class T, class R, class... ArgTypes>
struct ExportFunction<true, N, T, R (*)(ArgTypes...), M> {
    static typename ExportType<R (*)(ArgTypes...), M>::method_type export_method(R (*gf)(ArgTypes...))
    {
        return (typename ExportType<R (*)(ArgTypes...), M>::method_type)gf;
    }
};

template <int N, class M, class T, class R, class... ArgTypes>
static inline typename ExportType<R (*)(ArgTypes...), M>::method_type export_method(R (*gf)(ArgTypes...))
{
    typedef R (*fun_type)(ArgTypes...);
    return ExportFunction<is_abi_function<fun_type, M>::value, N, T, fun_type, M>::export_method(gf);
}

template <int N, class M, class T, class R, class... ArgTypes>
static inline typename ExportType<R (*)(ArgTypes..., ...)>::method_type export_method(R (*gf)(ArgTypes..., ...))
{
    static_assert(is_abi_function<R (*)(ArgTypes...)>::value);
    return gf;
}

template <int N, class M, class T, class R>
static inline const void* export_method(R (*)(va_list))
{
    static_assert(Unsupported<R (*)(...)>::value);
    return 0;
}

template <int N, class M, class T, class R, class... ArgTypes>
static inline typename ExportType<R (*)(ArgTypes...), M>::except_method_type export_except_method(R (*gf)(ArgTypes...))
{
#if __cplusplus >= 201703L
    static const void* id = (const void*)gf;
    return ExportExceptMethod<&id, R (*)(ArgTypes...), M>::type::method;
#elif __cplusplus >= 199711L
    ExportId<N, T>::id = (const void*)gf;
    return ExportExceptMethod<&ExportId<N, T>::id, R (*)(ArgTypes...), M>::type::method;
#else
#error hicc needs at least a C++11 compliant compiler
#endif
}

template <int N, class M, class T, class R, class... ArgTypes>
static inline const void* export_except_method(R (*)(ArgTypes..., ...))
{
    static_assert(Unsupported<R (*)(ArgTypes..., ...)>::value);
    return 0;
}

template <int N, class M, class T, class R>
static inline const void* export_except_method(R (*)(va_list))
{
    static_assert(Unsupported<R (*)(...)>::value);
    return 0;
}

template <int N, class M, class T, class D>
static inline typename ExportType<D T::*, M>::method_type export_data(D T::* field)
{
#if __cplusplus >= 201703L
    static const void* id = field_addr(field);
    return ExportMethod<&id, D T::*, M>::type::method;
#elif __cplusplus >= 199711L
    ExportId<N, T>::id = field_addr(field);
    return ExportMethod<&ExportId<N, T>::id, D T::*, M>::type::method;
#else
#error hicc needs at least a C++11 compliant compiler
#endif
}

template <int N, class M, class T, class D>
static inline typename ExportType<D*, M>::method_type export_data(D* data)
{
#if __cplusplus >= 201703L
    static const void* id = (const void*)data;
    return ExportMethod<&id, D*, M>::type::method;
#elif __cplusplus >= 199711L
    ExportId<N, T>::id = (const void*)data;
    return ExportMethod<&ExportId<N, T>::id, D*, M>::type::method;
#else
#error hicc needs at least a C++11 compliant compiler
#endif
}

template <int N, class M, class T, class D>
static inline typename ExportType<D T::*, M>::const_method_type export_const_data(D T::* field)
{
#if __cplusplus >= 201703L
    static const void* id = field_addr(field);
    return ExportMethod<&id, D T::*, M>::type::const_method;
#elif __cplusplus >= 199711L
    ExportId<N, T>::id = field_addr(field);
    return ExportMethod<&ExportId<N, T>::id, D T::*, M>::type::const_method;
#else
#error hicc needs at least a C++11 compliant compiler
#endif
}

template <int N, class M, class T, class D>
static inline typename ExportType<D*, M>::const_method_type export_const_data(D* data)
{
#if __cplusplus >= 201703L
    static const void* id = (const void*)data;
    return ExportMethod<&id, D*, M>::type::const_method;
#elif __cplusplus >= 199711L
    ExportId<N, T>::id = (const void*)data;
    return ExportMethod<&ExportId<N, T>::id, D*, M>::type::const_method;
#else
#error hicc needs at least a C++11 compliant compiler
#endif
}

template <class T1, class T2, class M = void>
struct ExportDynamic {
    typedef typename AbiValue<T1, M>::input_type input_type;
    typedef typename AbiValue<T2, M>::output_type output_type;
    typedef output_type (*method_type)(input_type*);
    static output_type method(input_type* obj) { return obj->template cast<T2, M>(); }
};

template <class T1, class T2, class M = void>
struct ExportDynamicMove {
    typedef typename AbiValue<T1, M>::input_type input_type;
    typedef typename AbiValue<T2, M>::output_type output_type;
    typedef output_type (*method_type)(input_type);
    static output_type method(input_type obj) { return obj.template cast_move<T2, M>(); }
};

template <class T1, class T2, class M = void>
static inline typename ExportDynamic<T1, T2, M>::method_type export_dynamic()
{
    return ExportDynamic<T1, T2, M>::method;
}

template <class T1, class T2, class M = void>
static inline typename ExportDynamicMove<T1, T2, M>::method_type export_dynamic_move()
{
    return ExportDynamicMove<T1, T2, M>::method;
}

template<class output, class Input> struct RemoteArg;

template<class T>
struct RemoteArg<T, T> {
        T val;
        RemoteArg(T val): val(val) {}
        T arg() const { return val; }
};

template<class T, class M> struct RemoteArg<AbiClass<T>, M*> {
        AbiClass<T> val;
        RemoteArg(AbiClass<T> val): val(val) {}
        M* arg() const { return (M*)&val; }
};

template<class T, class M> 
static inline RemoteArg<typename AbiValue<T, M>::output_type, typename AbiValue<T, M>::input_type> remote_arg(T&& val) {
        return RemoteArg<typename AbiValue<T, M>::output_type, typename AbiValue<T, M>::input_type>(AbiValue<T, M>::into(std::forward<T>(val)));
}

template <class T, class M = void>
struct RemoteProxy : public T {
    typedef typename AbiValue<T, M>::input_type _hicc_remote_type;
    _hicc_remote_type _hicc_remote;

    template <class U>
    struct RemoteMethod;

    template <class R, class... ArgTypes>
    struct RemoteMethod<R (T::*)(ArgTypes...)> {
        typedef typename AbiValue<T, M>::input_type _hicc_remote;
        const void* method;
        _hicc_remote* remote;
        RemoteMethod(const void* method, _hicc_remote& remote) : method(method), remote(&remote) {}
        R call(ArgTypes... args)
        {
            typedef typename ExportType<R (T::*)(ArgTypes...), M>::method_type method_type;
            assert(method);
            method_type mf = (method_type)method;
            return AbiValue<R, M>::from_with(
                [&]() { return mf(remote, remote_arg<ArgTypes, M>(std::move(args)).arg() ...); });
        }
    };

    template <class R, class... ArgTypes>
    struct RemoteMethod<R (T::*)(ArgTypes...) const> {
        typedef typename AbiValue<T, M>::input_type _hicc_remote;
        const void* method;
        const _hicc_remote* remote;
        RemoteMethod(const void* method, const _hicc_remote& remote) : method(method), remote(&remote) {}
        R call(ArgTypes... args)
        {
            typedef typename ExportType<R (T::*)(ArgTypes...) const, M>::method_type method_type;
            assert(method);
            method_type mf = (method_type)method;
            return AbiValue<R, M>::from_with(
                [&]() { return mf(remote, remote_arg<ArgTypes, M>(std::move(args)).arg() ...); });
        }
    };

    template <class R, class... ArgTypes>
    struct RemoteMethod<R (T::*)(ArgTypes...) &&> {
        typedef typename AbiValue<T, M>::input_type _hicc_remote;
        const void* method;
        _hicc_remote remote;
        RemoteMethod(const void* method, _hicc_remote& remote) : method(method), remote(remote) { remote.set_obj(0); }
        R call(ArgTypes... args)
        {
            typedef typename ExportType<R (T::*)(ArgTypes...)&&, M>::method_type method_type;
            assert(method);
            method_type mf = (method_type)method;
            return AbiValue<R, M>::from_with(
                [&]() { return mf(remote, remote_arg<ArgTypes, M>(std::move(args)).arg() ...); });
        }
    };

    template <class R, class... ArgTypes>
    static RemoteMethod<R (T::*)(ArgTypes...)> make_remote(const void* method, R (T::*)(ArgTypes...),
                                                           typename AbiValue<T, M>::input_type& remote)
    {
        return RemoteMethod<R (T::*)(ArgTypes...)>(method, remote);
    }

    template <class R, class... ArgTypes>
    static RemoteMethod<R (T::*)(ArgTypes...) const> make_remote(const void* method, R (T::*)(ArgTypes...) const,
                                                                 const typename AbiValue<T, M>::input_type& remote)
    {
        return RemoteMethod<R (T::*)(ArgTypes...) const>(method, remote);
    }

    template <class R, class... ArgTypes>
    static RemoteMethod<R (T::*)(ArgTypes...) &&> make_remote(const void* method, R (T::*)(ArgTypes...) &&,
                                                              typename AbiValue<T, M>::input_type& remote)
    {
        return RemoteMethod<R (T::*)(ArgTypes...) &&>(method, remote);
    }

    template <class R, class... ArgTypes>
    T& self(R (T::*)(ArgTypes...))
    {
        return *this;
    }
    template <class R, class... ArgTypes>
    const T& self(R (T::*)(ArgTypes...) const) const
    {
        return *this;
    }
    template <class R, class... ArgTypes>
    T&& self(R (T::*)(ArgTypes...) &&)
    {
        return std::move(*this);
    }
    template <class... ArgTypes>
    RemoteProxy(typename AbiValue<T, M>::input_type remote, ArgTypes&&... args)
        : T(std::forward<ArgTypes>(args)...), _hicc_remote(remote)
    {}
    ~RemoteProxy() { _hicc_remote.destroy(); }
};
}    // namespace hicc
#endif
