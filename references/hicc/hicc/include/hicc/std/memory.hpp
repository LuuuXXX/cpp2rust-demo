#ifndef HICC_STD_MEMORY_H
#define HICC_STD_MEMORY_H

#include <memory>

#include "../helper.hpp"

namespace hicc {

template <class T>
struct WeakPtrMethods {
    EXPORT_MEMBER_METHOD(&std::weak_ptr<T>::expired);
    EXPORT_MEMBER_METHOD(&std::weak_ptr<T>::lock);
};

template <class T>
struct MethodsType<std::weak_ptr<T>> {
    typedef WeakPtrMethods<T> methods_type;
};

template <class T>
struct SharedPtrMethods {
    EXPORT_MEMBER_METHOD(&std::shared_ptr<T>::get);
    static std::weak_ptr<T> weak(const std::shared_ptr<T>& shared) { return shared; }
    EXPORT_METHOD_IN(void, std::shared_ptr<T>, &SharedPtrMethods<T>::weak);
    static bool is_empty(const std::shared_ptr<T>& shared) { return !shared; }
    EXPORT_METHOD_IN(void, std::shared_ptr<T>, &SharedPtrMethods<T>::is_empty);
};

template <class T>
struct MethodsType<std::shared_ptr<T>> {
    typedef SharedPtrMethods<T> methods_type;
};

template <class T1, class T2, class... ArgTypes>
static inline std::unique_ptr<T2> make_unique_cast(ArgTypes&&... args)
{
    auto obj = new(std::nothrow) T1(std::forward<ArgTypes>(args)...);
    auto target = dynamic_cast<T2*>(obj);
    if (!target && obj) {
        Deleter<T1>::destroy(obj);
    }
    return std::unique_ptr<T2>(target);
}

template <class T, class... ArgTypes>
static inline std::unique_ptr<T> make_unique(ArgTypes&&... args)
{
    return std::unique_ptr<T>(new(std::nothrow) T(std::forward<ArgTypes>(args)...));
}

template <class T, class... ArgTypes>
static inline AbiClass<T> placement_new(void* buf, size_t len, ArgTypes&&... args)
{
    if (len < sizeof(T)) {
            return AbiClass<T>(AbiClass<T>::no_destroy_methods(), 0);
    }
    auto obj =  new(buf) T(std::forward<ArgTypes>(args)...);
    return AbiClass<T>(AbiClass<T>::dtor_methods(), obj);
}

template <class T>
static inline size_t size_of()
{
        return sizeof(T);
}

template <class T, class D>
struct UniquePtrMethods {
    EXPORT_MEMBER_METHOD(&std::unique_ptr<T, D>::get);
    static bool is_empty(const std::unique_ptr<T>& unique) { return !unique; }
    typedef std::unique_ptr<T, D> Self;
    EXPORT_METHOD_IN(void, Self, &UniquePtrMethods<T, D>::is_empty);
};

template <class T, class D>
struct MethodsType<std::unique_ptr<T, D>> {
    typedef UniquePtrMethods<T, D> methods_type;
};

// std::unique_ptr是常见的c++类型，针对性优化，对性能有利
// 如果T为class类型，则AbiClass<std::unique_ptr<T>>可以优化为AbiClass<T>
template <class T>
struct UniqueAbiType {
    typedef AbiClass<T> input_type;
    typedef AbiClass<T> output_type;
    static std::unique_ptr<T> from(input_type val)
    {
        // std::unique_ptr释放资源, 需要确保资源可以被释放
        assert(&val.get_methods() != &AbiClass<T>::no_destroy_methods());
        // std::unique_ptr释放资源，需要避免资源被重复释放
        val.set_methods(AbiClass<T>::no_destroy_methods());
        return std::unique_ptr<T>(make_unique_arg(val).arg.get_obj());
    }
    static output_type into(std::unique_ptr<T> val)
    {
        // 返回对象可以传递给std::unique_ptr<T>, 也可以直接传递给T, 这两者都代表需要c++端释放资源
        return output_type(AbiClass<T>::destroy_methods(), val.release());
    }
};

template <class T>
struct AbiValue<std::unique_ptr<T>> {
    typedef UniqueAbiType<T> unique_abi_type;
    typedef ClassType<std::unique_ptr<T>> common_abi_type;
    typedef
        typename std::conditional<is_class<T>::value && !is_pointer<T>::value, unique_abi_type, common_abi_type>::type
            abi_type;
    typedef typename abi_type::input_type input_type;
    typedef typename abi_type::output_type output_type;
    static std::unique_ptr<T> from(input_type arg) { return abi_type::from(arg); }
    static std::unique_ptr<T> from_with(std::function<input_type()> fun) { return from(fun()); }
    static output_type into(std::unique_ptr<T> val) { return abi_type::into(std::move(val)); }
    static output_type into_with(std::function<std::unique_ptr<T>()> fun) { return into(fun()); }
};

}    // namespace hicc

#endif
