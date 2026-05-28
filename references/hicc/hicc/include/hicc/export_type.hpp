#ifndef HICC_EXPORT_TYPE_H
#define HICC_EXPORT_TYPE_H

#include <cstdarg>
#include <type_traits>
#include <functional>

#include "types.hpp"

namespace hicc {

template <class T, class M = void>
struct ExportType;

template <const void** f, class T, class M = void>
struct ExportMethod;

template <const void** f, class T, class M = void>
struct ExportExceptMethod;

template <const void** f, class T, class M = void>
struct Method;

template <const void** f, class T, class M = void>
struct ExceptMethod;

template <class T, class M = void>
struct MethodType;

template <const void** f, class T, class M = void>
struct VMethod;

template <const void** f, class T, class M = void>
struct VExceptMethod;

template <class T, class M = void>
struct VMethodType;

template <class T, class P = void>
struct HasVType;

template <class P, class A, class F>
struct VFunction;

template <class T, class P>
struct HasVType {
    static const bool value = false;
    typedef void type;
    typedef void vtype;
};

template <class R, class A, class P>
struct HasVType<R (*)(A, va_list), P> {
    static const bool value = true;
    typedef typename VFunction<P, A, R (*)()>::type type;
    typedef typename VFunction<P, A, R (*)()>::vtype vtype;
};

template <class P, class R, class A1, class A2, class... ArgTypes>
struct HasVType<R (*)(A1, A2, ArgTypes...), P> {
    typedef std::tuple<P, A1> tuple;
    typedef HasVType<R (*)(A2, ArgTypes...), tuple> _vtype;
    typedef typename _vtype::type type;
    typedef typename _vtype::vtype vtype;
    static const bool value = _vtype::value;
};

template <class T, class R, class A, class P>
struct HasVType<R (T::*)(A, va_list), P> {
    static const bool value = true;
    typedef typename VFunction<P, A, R (T::*)()>::type type;
    typedef typename VFunction<P, A, R (T::*)()>::vtype vtype;
};

template <class T, class P, class R, class A1, class A2, class... ArgTypes>
struct HasVType<R (T::*)(A1, A2, ArgTypes...), P> {
    typedef std::tuple<P, A1> tuple;
    typedef HasVType<R (T::*)(A2, ArgTypes...), tuple> _vtype;
    typedef typename _vtype::type type;
    typedef typename _vtype::vtype vtype;
    static const bool value = _vtype::value;
};

template <class T, class R, class A, class P>
struct HasVType<R (T::*)(A, va_list) volatile, P> {
    static const bool value = true;
    typedef typename VFunction<P, A, R (T::*)() volatile>::type type;
    typedef typename VFunction<P, A, R (T::*)() volatile>::vtype vtype;
};

template <class T, class P, class R, class A1, class A2, class... ArgTypes>
struct HasVType<R (T::*)(A1, A2, ArgTypes...) volatile, P> {
    typedef std::tuple<P, A1> tuple;
    typedef HasVType<R (T::*)(A2, ArgTypes...) volatile, tuple> _vtype;
    typedef typename _vtype::type type;
    typedef typename _vtype::vtype vtype;
    static const bool value = _vtype::value;
};

template <class T, class R, class A, class P>
struct HasVType<R (T::*)(A, va_list) const, P> {
    static const bool value = true;
    typedef typename VFunction<P, A, R (T::*)() const>::type type;
    typedef typename VFunction<P, A, R (T::*)() const>::vtype vtype;
};

template <class T, class P, class R, class A1, class A2, class... ArgTypes>
struct HasVType<R (T::*)(A1, A2, ArgTypes...) const, P> {
    typedef std::tuple<P, A1> tuple;
    typedef HasVType<R (T::*)(A2, ArgTypes...) const, tuple> _vtype;
    typedef typename _vtype::type type;
    typedef typename _vtype::vtype vtype;
    static const bool value = _vtype::value;
};

template <class T, class R, class A, class P>
struct HasVType<R (T::*)(A, va_list) const volatile, P> {
    static const bool value = true;
    typedef typename VFunction<P, A, R (T::*)() const volatile>::type type;
    typedef typename VFunction<P, A, R (T::*)() const volatile>::vtype vtype;
};

template <class T, class P, class R, class A1, class A2, class... ArgTypes>
struct HasVType<R (T::*)(A1, A2, ArgTypes...) const volatile, P> {
    typedef std::tuple<P, A1> tuple;
    typedef HasVType<R (T::*)(A2, ArgTypes...) const volatile, tuple> _vtype;
    typedef typename _vtype::type type;
    typedef typename _vtype::vtype vtype;
    static const bool value = _vtype::value;
};

template <class T, class R, class A, class P>
struct HasVType<R (T::*)(A, va_list)&&, P> {
    static const bool value = true;
    typedef typename VFunction<P, A, R (T::*)() &&>::type type;
    typedef typename VFunction<P, A, R (T::*)() &&>::vtype vtype;
};

template <class T, class P, class R, class A1, class A2, class... ArgTypes>
struct HasVType<R (T::*)(A1, A2, ArgTypes...)&&, P> {
    typedef std::tuple<P, A1> tuple;
    typedef HasVType<R (T::*)(A2, ArgTypes...)&&, tuple> _vtype;
    typedef typename _vtype::type type;
    typedef typename _vtype::vtype vtype;
    static const bool value = _vtype::value;
};

template <class A, class R, class... ArgTypes>
struct VFunction<void, A, R (*)(ArgTypes...)> {
    typedef R (*type)(A, ArgTypes..., ...);
    typedef R (*vtype)(ArgTypes..., A, ...);
};

template <class A, class P, class R, class... ArgTypes>
struct VFunction<std::tuple<void, P>, A, R (*)(ArgTypes...)> {
    typedef R (*type)(A, P, ArgTypes..., ...);
    typedef R (*vtype)(P, ArgTypes..., A, ...);
};

template <class A, class P1, class P2, class R, class... ArgTypes>
struct VFunction<std::tuple<P1, P2>, A, R (*)(ArgTypes...)> {
    typedef VFunction<P1, A, R (*)(P2, ArgTypes...)> _vtype;
    typedef typename _vtype::type type;
    typedef typename _vtype::vtype vtype;
};

template <class A, class T, class R, class... ArgTypes>
struct VFunction<void, A, R (T::*)(ArgTypes...)> {
    typedef R (T::*type)(A, ArgTypes..., ...);
    typedef R (T::*vtype)(ArgTypes..., A, ...);
};

template <class A, class T, class P, class R, class... ArgTypes>
struct VFunction<std::tuple<void, P>, A, R (T::*)(ArgTypes...)> {
    typedef R (T::*type)(A, P, ArgTypes..., ...);
    typedef R (T::*vtype)(P, ArgTypes..., A, ...);
};

template <class A, class T, class P1, class P2, class R, class... ArgTypes>
struct VFunction<std::tuple<P1, P2>, A, R (T::*)(ArgTypes...)> {
    typedef VFunction<P1, A, R (T::*)(P2, ArgTypes...)> _vtype;
    typedef typename _vtype::type type;
    typedef typename _vtype::vtype vtype;
};

template <class A, class T, class R, class... ArgTypes>
struct VFunction<void, A, R (T::*)(ArgTypes...) const> {
    typedef R (T::*type)(A, ArgTypes..., ...) const;
    typedef R (T::*vtype)(ArgTypes..., A, ...) const;
};

template <class A, class T, class P, class R, class... ArgTypes>
struct VFunction<std::tuple<void, P>, A, R (T::*)(ArgTypes...) const> {
    typedef R (T::*type)(A, P, ArgTypes..., ...) const;
    typedef R (T::*vtype)(P, ArgTypes..., A, ...) const;
};

template <class A, class T, class P1, class P2, class R, class... ArgTypes>
struct VFunction<std::tuple<P1, P2>, A, R (T::*)(ArgTypes...) const> {
    typedef VFunction<P1, A, R (T::*)(P2, ArgTypes...) const> _vtype;
    typedef typename _vtype::type type;
    typedef typename _vtype::vtype vtype;
};

template <class A, class T, class R, class... ArgTypes>
struct VFunction<void, A, R (T::*)(ArgTypes...) volatile> {
    typedef R (T::*type)(A, ArgTypes..., ...) volatile;
    typedef R (T::*vtype)(ArgTypes..., A, ...) volatile;
};

template <class A, class T, class P, class R, class... ArgTypes>
struct VFunction<std::tuple<void, P>, A, R (T::*)(ArgTypes...) volatile> {
    typedef R (T::*type)(A, P, ArgTypes..., ...) volatile;
    typedef R (T::*vtype)(P, ArgTypes..., A, ...) volatile;
};

template <class A, class T, class P1, class P2, class R, class... ArgTypes>
struct VFunction<std::tuple<P1, P2>, A, R (T::*)(ArgTypes...) volatile> {
    typedef VFunction<P1, A, R (T::*)(P2, ArgTypes...) volatile> _vtype;
    typedef typename _vtype::type type;
    typedef typename _vtype::vtype vtype;
};

template <class A, class T, class R, class... ArgTypes>
struct VFunction<void, A, R (T::*)(ArgTypes...) const volatile> {
    typedef R (T::*type)(A, ArgTypes..., ...) const volatile;
    typedef R (T::*vtype)(ArgTypes..., A, ...) const volatile;
};

template <class A, class T, class P, class R, class... ArgTypes>
struct VFunction<std::tuple<void, P>, A, R (T::*)(ArgTypes...) const volatile> {
    typedef R (T::*type)(A, P, ArgTypes..., ...) const volatile;
    typedef R (T::*vtype)(P, ArgTypes..., A, ...) const volatile;
};

template <class A, class T, class P1, class P2, class R, class... ArgTypes>
struct VFunction<std::tuple<P1, P2>, A, R (T::*)(ArgTypes...) const volatile> {
    typedef VFunction<P1, A, R (T::*)(P2, ArgTypes...) const volatile> _vtype;
    typedef typename _vtype::type type;
    typedef typename _vtype::vtype vtype;
};

template <class A, class T, class R, class... ArgTypes>
struct VFunction<void, A, R (T::*)(ArgTypes...) &&> {
    typedef R (T::*type)(A, ArgTypes..., ...) &&;
    typedef R (T::*vtype)(ArgTypes..., A, ...) &&;
};

template <class A, class T, class P, class R, class... ArgTypes>
struct VFunction<std::tuple<void, P>, A, R (T::*)(ArgTypes...) &&> {
    typedef R (T::*type)(A, P, ArgTypes..., ...) &&;
    typedef R (T::*vtype)(P, ArgTypes..., A, ...) &&;
};

template <class A, class T, class P1, class P2, class R, class... ArgTypes>
struct VFunction<std::tuple<P1, P2>, A, R (T::*)(ArgTypes...) &&> {
    typedef VFunction<P1, A, R (T::*)(P2, ArgTypes...) &&> _vtype;
    typedef typename _vtype::type type;
    typedef typename _vtype::vtype vtype;
};

namespace {
template <class T, class R, class... Args>
union MemberFunction {
    const void* addr;
    R (T::*pm)(Args...);
    R (T::*pm_const)(Args...) const;
    R (T::*pm_volatile)(Args...) volatile;
    R (T::*pm_const_volatile)(Args...) const volatile;
    R (T::*pm_rr)(Args...) &&;
};

template <class T, class R, class... Args>
static inline MemberFunction<T, R, Args...> void_2_member(const void* addr)
{
    MemberFunction<T, R, Args...> mf = { 0 };
    mf.addr = addr;
    return mf;
}

template<class T, class D>
union MemberField {
    const void* addr;
    D T::* pm;
};

template<class T, class D>
static inline MemberField<T, D> void_2_field(const void* addr)
{
        MemberField<T, D> field = { 0 };
        field.addr = addr;
        return field;
}

}    // namespace

template <class T, class R, class... Args>
static inline const void* member_addr(R (T::*member)(Args...))
{
    MemberFunction<T, R, Args...> mf = { 0 };
    mf.pm = member;
    return mf.addr;
}

template <class T, class R, class... Args>
static inline const void* member_addr(R (T::*member)(Args...) const)
{
    MemberFunction<T, R, Args...> mf = { 0 };
    mf.pm_const = member;
    return mf.addr;
}

template <class T, class R, class... Args>
static inline const void* member_addr(R (T::*member)(Args...) volatile)
{
    MemberFunction<T, R, Args...> mf = { 0 };
    mf.pm_volatile = member;
    return mf.addr;
}

template <class T, class R, class... Args>
static inline const void* member_addr(R (T::*member)(Args...) const volatile)
{
    MemberFunction<T, R, Args...> mf = { 0 };
    mf.pm_const_volatile = member;
    return mf.addr;
}

template <class T, class R, class... Args>
static inline const void* member_addr(R (T::*member)(Args...) &&)
{
    MemberFunction<T, R, Args...> mf = { 0 };
    mf.pm_rr = member;
    return mf.addr;
}

template<class T, class D>
static inline const void* field_addr(D T::* field)
{
    MemberField<T, D> mf = { 0 };
    mf.pm = field;
    return mf.addr;
}

template <class M, class R, class... ArgTypes>
struct MethodType<R (*)(ArgTypes..., ...), M> {
    typedef R (*method_type)(ArgTypes..., ...);
    typedef void* except_method_type;
    typedef void const_method_type;
};

template <class M, class R, class... ArgTypes>
struct MethodType<R (*)(ArgTypes...), M> {
    typedef typename AbiValue<R, M>::output_type (*method_type)(typename AbiValue<ArgTypes, M>::input_type...);
    typedef Exception<typename AbiValue<R, M>::output_type> (*except_method_type)(
        typename AbiValue<ArgTypes, M>::input_type...);
    typedef void const_method_type;
};

template <class M, class T, class R, class... ArgTypes>
struct MethodType<R (T::*)(ArgTypes...), M> {
    typedef typename AbiValue<R, M>::output_type (*method_type)(typename AbiValue<T, M>::input_type*,
                                                                typename AbiValue<ArgTypes, M>::input_type...);
    typedef Exception<typename AbiValue<R, M>::output_type> (*except_method_type)(
        typename AbiValue<T, M>::input_type*, typename AbiValue<ArgTypes, M>::input_type...);
    typedef void const_method_type;
};

template <class M, class T, class R, class... ArgTypes>
struct MethodType<R (T::*)(ArgTypes...) volatile, M> {
    typedef typename AbiValue<R, M>::output_type (*method_type)(typename AbiValue<T, M>::input_type*,
                                                                typename AbiValue<ArgTypes, M>::input_type...);
    typedef Exception<typename AbiValue<R, M>::output_type> (*except_method_type)(
        typename AbiValue<T, M>::input_type*, typename AbiValue<ArgTypes, M>::input_type...);
    typedef void const_method_type;
};

template <class M, class T, class R, class... ArgTypes>
struct MethodType<R (T::*)(ArgTypes...) const, M> {
    typedef typename AbiValue<R, M>::output_type (*method_type)(const typename AbiValue<T, M>::input_type*,
                                                                typename AbiValue<ArgTypes, M>::input_type...);
    typedef Exception<typename AbiValue<R, M>::output_type> (*except_method_type)(
        const typename AbiValue<T, M>::input_type*, typename AbiValue<ArgTypes, M>::input_type...);
    typedef void const_method_type;
};

template <class M, class T, class R, class... ArgTypes>
struct MethodType<R (T::*)(ArgTypes...) const volatile, M> {
    typedef typename AbiValue<R, M>::output_type (*method_type)(const typename AbiValue<T, M>::input_type*,
                                                                typename AbiValue<ArgTypes, M>::input_type...);
    typedef Exception<typename AbiValue<R, M>::output_type> (*except_method_type)(
        const typename AbiValue<T, M>::input_type*, typename AbiValue<ArgTypes, M>::input_type...);
    typedef void const_method_type;
};

template <class M, class T, class R, class... ArgTypes>
struct MethodType<R (T::*)(ArgTypes...)&&, M> {
    typedef typename AbiValue<R, M>::output_type (*method_type)(typename AbiValue<T, M>::input_type,
                                                                typename AbiValue<ArgTypes, M>::input_type...);
    typedef Exception<typename AbiValue<R, M>::output_type> (*except_method_type)(
        typename AbiValue<T, M>::input_type, typename AbiValue<ArgTypes, M>::input_type...);
    typedef void const_method_type;
};

template <const void** f, class M, class R, class... ArgTypes>
struct Method<f, R (*)(ArgTypes...), M> {
    static typename AbiValue<R, M>::output_type method(typename AbiValue<ArgTypes, M>::input_type... args)
    {
        typedef R (*fun_type)(ArgTypes...);
        return AbiValue<R, M>::into_with(
            [&]() -> R { return ((fun_type)*f)((AbiValue<ArgTypes, M>::from(std::move(args)))...); });
    }
};

template <const void** f, class M, class R, class... ArgTypes>
struct ExceptMethod<f, R (*)(ArgTypes...), M> {
    typedef Exception<typename AbiValue<R, M>::output_type> output_type;
    static output_type method(typename AbiValue<ArgTypes, M>::input_type... args)
    {
        typedef R (*fun_type)(ArgTypes...);
        return output_type([&]() -> typename AbiValue<R, M>::output_type {
            return AbiValue<R, M>::into_with(
                [&]() -> R { return ((fun_type)*f)((AbiValue<ArgTypes, M>::from(std::move(args)))...); });
        });
    }
};

template <const void** f, class M, class R, class... ArgTypes>
struct Method<f, std::function<R(ArgTypes...)>, M> {
    static typename AbiValue<R, M>::output_type method(typename AbiValue<ArgTypes, M>::input_type... args)
    {
        auto func = (std::function<R(ArgTypes...)>*)*f;
        return AbiValue<R, M>::into_with(
            [&]() -> R { return (*func)((AbiValue<ArgTypes, M>::from(std::move(args)))...); });
    }
};

template <const void** f, class M, class R, class... ArgTypes>
struct ExceptMethod<f, std::function<R(ArgTypes...)>, M> {
    typedef Exception<typename AbiValue<R, M>::output_type> output_type;
    static output_type method(typename AbiValue<ArgTypes, M>::input_type... args)
    {
        auto func = (std::function<R(ArgTypes...)>*)*f;
        return output_type([&]() -> typename AbiValue<R, M>::output_type {
            return AbiValue<R, M>::into_with(
                [&]() -> R { return (*func)((AbiValue<ArgTypes, M>::from(std::move(args)))...); });
        });
    }
};

template <const void** f, class M, class T, class R, class... ArgTypes>
struct Method<f, R (T::*)(ArgTypes...), M> {
    static typename AbiValue<R, M>::output_type method(typename AbiValue<T, M>::input_type* obj,
                                                       typename AbiValue<ArgTypes, M>::input_type... args)
    {
        auto mf = void_2_member<T, R, ArgTypes...>(*f).pm;
        return AbiValue<R, M>::into_with(
            [&]() -> R { return (obj->get_obj()->*mf)((AbiValue<ArgTypes, M>::from(std::move(args)))...); });
    }
};

template <const void** f, class M, class T, class R, class... ArgTypes>
struct ExceptMethod<f, R (T::*)(ArgTypes...), M> {
    typedef Exception<typename AbiValue<R, M>::output_type> output_type;
    static output_type method(typename AbiValue<T, M>::input_type* obj,
                              typename AbiValue<ArgTypes, M>::input_type... args)
    {
        auto mf = void_2_member<T, R, ArgTypes...>(*f).pm;
        return output_type([&]() -> typename AbiValue<R, M>::output_type {
            return AbiValue<R, M>::into_with(
                [&]() -> R { return (obj->get_obj()->*mf)((AbiValue<ArgTypes, M>::from(std::move(args)))...); });
        });
    }
};

template <const void** f, class M, class T, class R, class... ArgTypes>
struct Method<f, R (T::*)(ArgTypes...) const, M> {
    static typename AbiValue<R, M>::output_type method(const typename AbiValue<T, M>::input_type* obj,
                                                       typename AbiValue<ArgTypes, M>::input_type... args)
    {
        auto mf = void_2_member<T, R, ArgTypes...>(*f).pm_const;
        return AbiValue<R, M>::into_with(
            [&]() -> R { return (obj->get_obj()->*mf)((AbiValue<ArgTypes, M>::from(std::move(args)))...); });
    }
};

template <const void** f, class M, class T, class R, class... ArgTypes>
struct ExceptMethod<f, R (T::*)(ArgTypes...) const, M> {
    typedef Exception<typename AbiValue<R, M>::output_type> output_type;
    static output_type method(const typename AbiValue<T, M>::input_type* obj,
                              typename AbiValue<ArgTypes, M>::input_type... args)
    {
        auto mf = void_2_member<T, R, ArgTypes...>(*f).pm_const;
        return output_type([&]() -> typename AbiValue<R, M>::output_type {
            return AbiValue<R, M>::into_with(
                [&]() -> R { return (obj->get_obj()->*mf)((AbiValue<ArgTypes, M>::from(std::move(args)))...); });
        });
    }
};

template <const void** f, class M, class T, class R, class... ArgTypes>
struct Method<f, R (T::*)(ArgTypes...) volatile, M> {
    static typename AbiValue<R, M>::output_type method(typename AbiValue<T, M>::input_type* obj,
                                                       typename AbiValue<ArgTypes, M>::input_type... args)
    {
        auto mf = void_2_member<T, R, ArgTypes...>(*f).pm_volatile;
        return AbiValue<R, M>::into_with(
            [&]() -> R { return (obj->get_obj()->*mf)((AbiValue<ArgTypes, M>::from(std::move(args)))...); });
    }
};

template <const void** f, class M, class T, class R, class... ArgTypes>
struct ExceptMethod<f, R (T::*)(ArgTypes...) volatile, M> {
    typedef Exception<typename AbiValue<R, M>::output_type> output_type;
    static output_type method(typename AbiValue<T, M>::input_type* obj,
                              typename AbiValue<ArgTypes, M>::input_type... args)
    {
        auto mf = void_2_member<T, R, ArgTypes...>(*f).pm_volatile;
        return output_type([&]() -> typename AbiValue<R, M>::output_type {
            return AbiValue<R, M>::into_with(
                [&]() -> R { return (obj->get_obj()->*mf)((AbiValue<ArgTypes, M>::from(std::move(args)))...); });
        });
    }
};

template <const void** f, class M, class T, class R, class... ArgTypes>
struct Method<f, R (T::*)(ArgTypes...) const volatile, M> {
    static typename AbiValue<R, M>::output_type method(const typename AbiValue<T, M>::input_type* obj,
                                                       typename AbiValue<ArgTypes, M>::input_type... args)
    {
        auto mf = void_2_member<T, R, ArgTypes...>(*f).pm_const_volatile;
        return AbiValue<R, M>::into_with(
            [&]() -> R { return (obj->get_obj()->*mf)((AbiValue<ArgTypes, M>::from(std::move(args)))...); });
    }
};

template <const void** f, class M, class T, class R, class... ArgTypes>
struct ExceptMethod<f, R (T::*)(ArgTypes...) const volatile, M> {
    typedef Exception<typename AbiValue<R, M>::output_type> output_type;
    static output_type method(const typename AbiValue<T, M>::input_type* obj,
                              typename AbiValue<ArgTypes, M>::input_type... args)
    {
        auto mf = void_2_member<T, R, ArgTypes...>(*f).pm_const_volatile;
        return output_type([&]() -> typename AbiValue<R, M>::output_type {
            return AbiValue<R, M>::into_with(
                [&]() -> R { return (obj->get_obj()->*mf)((AbiValue<ArgTypes, M>::from(std::move(args)))...); });
        });
    }
};

template <const void** f, class M, class T, class R, class... ArgTypes>
struct Method<f, R (T::*)(ArgTypes...)&&, M> {
    static typename AbiValue<R, M>::output_type method(typename AbiValue<T, M>::input_type obj,
                                                       typename AbiValue<ArgTypes, M>::input_type... args)
    {
        auto mf = void_2_member<T, R, ArgTypes...>(*f).pm_rr;
        return AbiValue<R, M>::into_with(
            [&]() -> R { return ((**obj).*mf)((AbiValue<ArgTypes, M>::from(std::move(args)))...); });
    }
};

template <const void** f, class M, class T, class R, class... ArgTypes>
struct ExceptMethod<f, R (T::*)(ArgTypes...)&&, M> {
    typedef Exception<typename AbiValue<R, M>::output_type> output_type;
    static output_type method(typename AbiValue<T, M>::input_type obj,
                              typename AbiValue<ArgTypes, M>::input_type... args)
    {
        auto mf = void_2_member<T, R, ArgTypes...>(*f).pm_rr;
        return output_type([&]() -> typename AbiValue<R, M>::output_type {
            return AbiValue<R, M>::into_with(
                [&]() -> R { return ((**obj).*mf)((AbiValue<ArgTypes, M>::from(std::move(args)))...); });
        });
    }
};

template <class M, class T, class D>
struct MethodType<D T::*, M> {
        typedef typename AbiValue<D&, M>::output_type (*method_type)(typename AbiValue<T, M>::input_type*);
        typedef typename AbiValue<const D&, M>::output_type (*const_method_type)(const typename AbiValue<T, M>::input_type*);
        typedef void except_method_type;
};

template <const void** f, class M, class T, class D>
struct Method<f, D T::*, M> {
        static typename AbiValue<D&, M>::output_type method(typename AbiValue<T, M>::input_type* obj) {
                auto mf = void_2_field<T, D>(*f).pm;
                return AbiValue<D&, M>::into(obj->get_obj()->*mf);
        }
        static typename AbiValue<const D&, M>::output_type const_method(const typename AbiValue<T, M>::input_type* obj) {
                auto mf = void_2_field<T, D>(*f).pm;
                return AbiValue<const D&, M>::into(obj->get_obj()->*mf);
        }
};

/// 指针类型特殊处理
template <class M, class T, class D>
struct MethodType<D* T::*, M> {
        typedef typename AbiValue<D*, M>::output_type (*method_type)(typename AbiValue<T, M>::input_type*);
        typedef typename AbiValue<const D*, M>::output_type (*const_method_type)(const typename AbiValue<T, M>::input_type*);
        typedef void except_method_type;
};

template <const void** f, class M, class T, class D>
struct Method<f, D* T::*, M> {
        static typename AbiValue<D*, M>::output_type method(typename AbiValue<T, M>::input_type* obj) {
                auto mf = void_2_field<T, D*>(*f).pm;
                return AbiValue<D*, M>::into(std::move(obj->get_obj()->*mf));
        }
        static typename AbiValue<const D*, M>::output_type const_method(const typename AbiValue<T, M>::input_type* obj) {
                auto mf = void_2_field<T, const D*>(*f).pm;
                return AbiValue<const D*, M>::into(std::move(obj->get_obj()->*mf));
        }
};

template <class M, class T, class D, int N>
struct MethodType<D (T::*)[N], M> {
        typedef typename AbiValue<D*, M>::output_type (*method_type)(typename AbiValue<T, M>::input_type*);
        typedef typename AbiValue<const D*, M>::output_type (*const_method_type)(const typename AbiValue<T, M>::input_type*);
        typedef void except_method_type;
};

template <const void** f, class M, class T, class D, int N>
struct Method<f, D (T::*)[N], M> {
        static typename AbiValue<D*, M>::output_type method(typename AbiValue<T, M>::input_type* obj) {
                auto mf = void_2_field<T, D[N]>(*f).pm;
                return AbiValue<D*, M>::into(std::move(&(obj->get_obj()->*mf)[0]));
        }
        static typename AbiValue<const D*, M>::output_type const_method(const typename AbiValue<T, M>::input_type* obj) {
                auto mf = void_2_field<T, const D[N]>(*f).pm;
                return AbiValue<const D*, M>::into(std::move(&(obj->get_obj()->*mf)[0]));
        }
};

template<class M, class D>
struct MethodType<D*, M> {
        typedef typename AbiValue<D*, M>::output_type (*method_type)();
        typedef typename AbiValue<const D*, M>::output_type (*const_method_type)();
        typedef void except_method_type;
};

template <const void** f, class M, class D>
struct Method<f, D*, M> {
        static typename AbiValue<D&, M>::output_type method() {
                return AbiValue<D&, M>::into(*(D*)*f);
        }
        static typename AbiValue<const D&, M>::output_type const_method() {
                return AbiValue<const D&, M>::into(*(const D*)*f);
        }
};

template<class M, class D>
struct MethodType<D**, M> {
        typedef typename AbiValue<D*, M>::output_type (*method_type)();
        typedef typename AbiValue<const D*, M>::output_type (*const_method_type)();
        typedef void except_method_type;
};

template <const void** f, class M, class D>
struct Method<f, D**, M> {
        static typename AbiValue<D*, M>::output_type method() {
                return AbiValue<D*, M>::into(std::move(*(D**)*f));
        }
        static typename AbiValue<const D*, M>::output_type const_method() {
                return AbiValue<const D*, M>::into(std::move(*(const D**)*f));
        }
};

template<class M, class D, int N>
struct MethodType<D(*)[N], M> {
        typedef typename AbiValue<D*, M>::output_type (*method_type)();
        typedef typename AbiValue<const D*, M>::output_type (*const_method_type)();
        typedef void except_method_type;
};

template <const void** f, class M, class D, int N>
struct Method<f, D(*)[N], M> {
        static typename AbiValue<D*, M>::output_type method() {
            typedef D(*DN)[N];
                return AbiValue<D*, M>::into(std::move(&(*(DN)*f)[0]));
        }
        static typename AbiValue<const D*, M>::output_type const_method() {
            typedef const D(*DN)[N];
                return AbiValue<const D*, M>::into(std::move(&(*(DN)*f)[0]));
        }
};

template <class M, class R>
struct is_abi_function<R (*)(va_list), M> {
    static const bool value = false;
};

template <class M, class R, class... ArgTypes>
struct VMethodType<R (*)(ArgTypes..., ...), M> {
    typedef typename AbiValue<R, M>::output_type (*method_type)(typename AbiValue<ArgTypes, M>::input_type..., ...);
    typedef Exception<typename AbiValue<R, M>::output_type> (*except_method_type)(
        typename AbiValue<ArgTypes, M>::input_type..., ...);
    typedef void const_method_type;
};

template <class M, class R, class T, class... ArgTypes>
struct VMethodType<R (T::*)(ArgTypes..., ...), M> {
    typedef typename AbiValue<R, M>::output_type (*method_type)(typename AbiValue<T, M>::input_type*,
                                                                typename AbiValue<ArgTypes, M>::input_type..., ...);
    typedef Exception<typename AbiValue<R, M>::output_type> (*except_method_type)(
        typename AbiValue<T, M>::input_type*, typename AbiValue<ArgTypes, M>::input_type..., ...);
    typedef void const_method_type;
};

template <class M, class R, class T, class... ArgTypes>
struct VMethodType<R (T::*)(ArgTypes..., ...) volatile, M> {
    typedef typename AbiValue<R, M>::output_type (*method_type)(typename AbiValue<T, M>::input_type*,
                                                                typename AbiValue<ArgTypes, M>::input_type..., ...);
    typedef Exception<typename AbiValue<R, M>::output_type> (*except_method_type)(
        typename AbiValue<T, M>::input_type*, typename AbiValue<ArgTypes, M>::input_type..., ...);
    typedef void const_method_type;
};

template <class M, class R, class T, class... ArgTypes>
struct VMethodType<R (T::*)(ArgTypes..., ...) const, M> {
    typedef typename AbiValue<R, M>::output_type (*method_type)(const typename AbiValue<T, M>::input_type*,
                                                                typename AbiValue<ArgTypes, M>::input_type..., ...);
    typedef Exception<typename AbiValue<R, M>::output_type> (*except_method_type)(
        const typename AbiValue<T, M>::input_type*, typename AbiValue<ArgTypes, M>::input_type..., ...);
    typedef void const_method_type;
};

template <class M, class R, class T, class... ArgTypes>
struct VMethodType<R (T::*)(ArgTypes..., ...) const volatile, M> {
    typedef typename AbiValue<R, M>::output_type (*method_type)(const typename AbiValue<T, M>::input_type*,
                                                                typename AbiValue<ArgTypes, M>::input_type..., ...);
    typedef Exception<typename AbiValue<R, M>::output_type> (*except_method_type)(
        const typename AbiValue<T, M>::input_type*, typename AbiValue<ArgTypes, M>::input_type..., ...);
    typedef void const_method_type;
};

template <class M, class R, class T, class... ArgTypes>
struct VMethodType<R (T::*)(ArgTypes..., ...)&&, M> {
    typedef typename AbiValue<R, M>::output_type (*method_type)(typename AbiValue<T, M>::input_type,
                                                                typename AbiValue<ArgTypes, M>::input_type..., ...);
    typedef Exception<typename AbiValue<R, M>::output_type> (*except_method_type)(
        typename AbiValue<T, M>::input_type, typename AbiValue<ArgTypes, M>::input_type..., ...);
    typedef void const_method_type;
};

struct VaList {
    va_list& ap;
    VaList(va_list& ap) : ap(ap) {}
    ~VaList() { va_end(ap); }
};

template <class M, const void** f, class R, class A, class... ArgTypes>
struct VMethod<f, R (*)(A, ArgTypes..., ...), M> {
    static typename AbiValue<R, M>::output_type method(typename AbiValue<ArgTypes, M>::input_type... args,
                                                       typename AbiValue<A, M>::input_type arg, ...)
    {
        va_list ap;
        va_start(ap, arg);
        typedef R (*fun_type)(ArgTypes..., A, va_list);
        return AbiValue<R, M>::into_with([&]() -> R {
            return ((fun_type)*f)(AbiValue<ArgTypes, M>::from(std::move(args))..., AbiValue<A, M>::from(std::move(arg)),
                                  VaList(ap).ap);
        });
    }
};

template <class M, const void** f, class R, class A, class... ArgTypes>
struct VExceptMethod<f, R (*)(A, ArgTypes..., ...), M> {
    typedef Exception<typename AbiValue<R, M>::output_type> output_type;
    static output_type method(typename AbiValue<ArgTypes, M>::input_type... args,
                              typename AbiValue<A, M>::input_type arg, ...)
    {
        va_list ap;
        va_start(ap, arg);
        typedef R (*fun_type)(ArgTypes..., A, va_list);
        return output_type([&]() -> typename AbiValue<R, M>::output_type {
            return AbiValue<R, M>::into_with([&]() -> R {
                return ((fun_type)*f)(AbiValue<ArgTypes, M>::from(std::move(args))...,
                                      AbiValue<A, M>::from(std::move(arg)), VaList(ap).ap);
            });
        });
    }
};

template <class M, const void** f, class T, class R, class A, class... ArgTypes>
struct VMethod<f, R (T::*)(A, ArgTypes..., ...), M> {
    static typename AbiValue<R, M>::output_type method(typename AbiValue<T, M>::input_type* obj,
                                                       typename AbiValue<ArgTypes, M>::input_type... args,
                                                       typename AbiValue<A, M>::input_type arg, ...)
    {
        va_list ap;
        va_start(ap, arg);
        auto mf = void_2_member<T, R, ArgTypes..., A, va_list>(*f).pm;
        return AbiValue<R, M>::into_with([&]() -> R {
            return (obj->get_obj()->*mf)(AbiValue<ArgTypes, M>::from(std::move(args))...,
                                         AbiValue<A, M>::from(std::move(arg)), VaList(ap).ap);
        });
    }
};

template <class M, const void** f, class T, class R, class A, class... ArgTypes>
struct VExceptMethod<f, R (T::*)(A, ArgTypes..., ...), M> {
    typedef Exception<typename AbiValue<R, M>::output_type> output_type;
    static output_type method(typename AbiValue<T, M>::input_type* obj,
                              typename AbiValue<ArgTypes, M>::input_type... args,
                              typename AbiValue<A, M>::input_type arg, ...)
    {
        va_list ap;
        va_start(ap, arg);
        auto mf = void_2_member<T, R, ArgTypes..., A, va_list>(*f).pm;
        return output_type([&]() -> typename AbiValue<R, M>::output_type {
            return AbiValue<R, M>::into_with([&]() -> R {
                return (obj->get_obj()->*mf)(AbiValue<ArgTypes, M>::from(std::move(args))...,
                                             AbiValue<A, M>::from(std::move(arg)), VaList(ap).ap);
            });
        });
    }
};

template <class M, const void** f, class T, class R, class A, class... ArgTypes>
struct VMethod<f, R (T::*)(A, ArgTypes..., ...) volatile, M> {
    static typename AbiValue<R, M>::output_type method(typename AbiValue<T, M>::input_type* obj,
                                                       typename AbiValue<ArgTypes, M>::input_type... args,
                                                       typename AbiValue<A, M>::input_type arg, ...)
    {
        va_list ap;
        va_start(ap, arg);
        auto mf = void_2_member<T, R, ArgTypes..., A, va_list>(*f).pm_volatile;
        return AbiValue<R, M>::into_with([&]() -> R {
            return (obj->get_obj()->*mf)(AbiValue<ArgTypes, M>::from(std::move(args))...,
                                         AbiValue<A, M>::from(std::move(arg)), VaList(ap).ap);
        });
    }
};

template <class M, const void** f, class T, class R, class A, class... ArgTypes>
struct VExceptMethod<f, R (T::*)(A, ArgTypes..., ...) volatile, M> {
    typedef Exception<typename AbiValue<R, M>::output_type> output_type;
    static output_type method(typename AbiValue<T, M>::input_type* obj,
                              typename AbiValue<ArgTypes, M>::input_type... args,
                              typename AbiValue<A, M>::input_type arg, ...)
    {
        va_list ap;
        va_start(ap, arg);
        auto mf = void_2_member<T, R, ArgTypes..., A, va_list>(*f).pm_volatile;
        return output_type([&]() -> typename AbiValue<R, M>::output_type {
            return AbiValue<R, M>::into_with([&]() -> R {
                return (obj->get_obj()->*mf)(AbiValue<ArgTypes, M>::from(std::move(args))...,
                                             AbiValue<A, M>::from(std::move(arg)), VaList(ap).ap);
            });
        });
    }
};

template <class M, const void** f, class T, class R, class A, class... ArgTypes>
struct VMethod<f, R (T::*)(A, ArgTypes..., ...) const, M> {
    static typename AbiValue<R, M>::output_type method(const typename AbiValue<T, M>::input_type* obj,
                                                       typename AbiValue<ArgTypes, M>::input_type... args,
                                                       typename AbiValue<A, M>::input_type arg, ...)
    {
        va_list ap;
        va_start(ap, arg);
        auto mf = void_2_member<T, R, ArgTypes..., A, va_list>(*f).pm_const;
        return AbiValue<R, M>::into_with([&]() -> R {
            return (obj->get_obj()->*mf)(AbiValue<ArgTypes, M>::from(std::move(args))...,
                                         AbiValue<A, M>::from(std::move(arg)), VaList(ap).ap);
        });
    }
};

template <class M, const void** f, class T, class R, class A, class... ArgTypes>
struct VExceptMethod<f, R (T::*)(A, ArgTypes..., ...) const, M> {
    typedef Exception<typename AbiValue<R, M>::output_type> output_type;
    static output_type method(const typename AbiValue<T, M>::input_type* obj,
                              typename AbiValue<ArgTypes, M>::input_type... args,
                              typename AbiValue<A, M>::input_type arg, ...)
    {
        va_list ap;
        va_start(ap, arg);
        auto mf = void_2_member<T, R, ArgTypes..., A, va_list>(*f).pm_const;
        return output_type([&]() -> typename AbiValue<R, M>::output_type {
            return AbiValue<R, M>::into_with([&]() -> R {
                return (obj->get_obj()->*mf)(AbiValue<ArgTypes, M>::from(std::move(args))...,
                                             AbiValue<A, M>::from(std::move(arg)), VaList(ap).ap);
            });
        });
    }
};

template <class M, const void** f, class T, class R, class A, class... ArgTypes>
struct VMethod<f, R (T::*)(A, ArgTypes..., ...) const volatile, M> {
    static typename AbiValue<R, M>::output_type method(const typename AbiValue<T, M>::input_type* obj,
                                                       typename AbiValue<ArgTypes, M>::input_type... args,
                                                       typename AbiValue<A, M>::input_type arg, ...)
    {
        va_list ap;
        va_start(ap, arg);
        auto mf = void_2_member<T, R, ArgTypes..., A, va_list>(*f).pm_const_volatile;
        return AbiValue<R, M>::into_with([&]() -> R {
            return (obj->get_obj()->*mf)(AbiValue<ArgTypes, M>::from(std::move(args))...,
                                         AbiValue<A, M>::from(std::move(arg)), VaList(ap).ap);
        });
    }
};

template <class M, const void** f, class T, class R, class A, class... ArgTypes>
struct VExceptMethod<f, R (T::*)(A, ArgTypes..., ...) const volatile, M> {
    typedef Exception<typename AbiValue<R, M>::output_type> output_type;
    static output_type method(const typename AbiValue<T, M>::input_type* obj,
                              typename AbiValue<ArgTypes, M>::input_type... args,
                              typename AbiValue<A, M>::input_type arg, ...)
    {
        va_list ap;
        va_start(ap, arg);
        auto mf = void_2_member<T, R, ArgTypes..., A, va_list>(*f).pm_const_volatile;
        return output_type([&]() -> typename AbiValue<R, M>::output_type {
            return AbiValue<R, M>::into_with([&]() -> R {
                return (obj->get_obj()->*mf)(AbiValue<ArgTypes, M>::from(std::move(args))...,
                                             AbiValue<A, M>::from(std::move(arg)), VaList(ap).ap);
            });
        });
    }
};

template <class M, const void** f, class T, class R, class A, class... ArgTypes>
struct VMethod<f, R (T::*)(A, ArgTypes..., ...)&&, M> {
    static typename AbiValue<R, M>::output_type method(typename AbiValue<T, M>::input_type obj,
                                                       typename AbiValue<ArgTypes, M>::input_type... args,
                                                       typename AbiValue<A, M>::input_type arg, ...)
    {
        va_list ap;
        va_start(ap, arg);
        auto mf = void_2_member<T, R, ArgTypes..., A, va_list>(*f).pm_rr;
        return AbiValue<R, M>::into_with([&]() -> R {
            return ((**obj).*mf)(AbiValue<ArgTypes, M>::from(std::move(args))..., AbiValue<A, M>::from(std::move(arg)),
                                 VaList(ap).ap);
        });
    }
};

template <class M, const void** f, class T, class R, class A, class... ArgTypes>
struct VExceptMethod<f, R (T::*)(A, ArgTypes..., ...)&&, M> {
    typedef Exception<typename AbiValue<R, M>::output_type> output_type;
    static output_type method(typename AbiValue<T, M>::input_type obj,
                              typename AbiValue<ArgTypes, M>::input_type... args,
                              typename AbiValue<A, M>::input_type arg, ...)
    {
        va_list ap;
        va_start(ap, arg);
        auto mf = void_2_member<T, R, ArgTypes..., A, va_list>(*f).pm_rr;
        return output_type([&]() -> typename AbiValue<R, M>::output_type {
            return AbiValue<R, M>::into_with([&]() -> R {
                return ((**obj).*mf)(AbiValue<ArgTypes, M>::from(std::move(args))...,
                                     AbiValue<A, M>::from(std::move(arg)), VaList(ap).ap);
            });
        });
    }
};

template <class T, class M>
struct ExportType {
    typedef HasVType<T> vtype;
    typedef typename std::conditional<vtype::value, VMethodType<typename vtype::vtype, M>, MethodType<T, M>>::type type;
    typedef typename type::method_type method_type;
    typedef typename type::const_method_type const_method_type;
    typedef typename type::except_method_type except_method_type;
};

template <const void** f, class T, class M>
struct ExportMethod {
    typedef HasVType<T> vtype;
    typedef typename std::conditional<vtype::value, VMethod<f, typename vtype::type, M>, Method<f, T, M>>::type type;
};

template <const void** f, class R, class M, class ArgTypes>
struct ExportMethod<f, std::function<R(ArgTypes...)>, M> {
    typedef Method<f, std::function<R(ArgTypes...)>, M> type;
};

template <const void** f, class T, class M>
struct ExportExceptMethod {
    typedef HasVType<T> vtype;
    typedef
        typename std::conditional<vtype::value, VExceptMethod<f, typename vtype::type, M>, ExceptMethod<f, T, M>>::type
            type;
};

template <const void** f, class R, class M, class ArgTypes>
struct ExportExceptMethod<f, std::function<R(ArgTypes...)>, M> {
    typedef ExceptMethod<f, std::function<R(ArgTypes...)>, M> type;
};

}    // namespace hicc
#endif
