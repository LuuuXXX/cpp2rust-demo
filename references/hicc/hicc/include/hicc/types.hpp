#ifndef HICC_TYPES_H
#define HICC_TYPES_H

#include <cassert>
#include <functional>
#include <iostream>
#include <tuple>
#include <type_traits>

namespace hicc {

template <class T>
struct Deleter;

template <class T, class M = void>
struct MethodsType;

template <class T, class M = void>
struct AbiValue;

template <class T, class M = void>
struct AbiClass;

template <class T, class M = void>
struct ClassType;

template <class T, class M, class V, class MM>
struct AbiType;

template <class T, class M = void>
struct PlainType;

template <class T>
struct PodType;

template <class T>
struct is_pointer;

template <class T>
struct InputType;

template <class T, class M = void>
struct is_abi_function;

template <class T, class M = void>
struct is_class;

template <class T>
struct is_pod;

template <class T>
struct AbiClassType;

template <class T>
struct UniqueArg;

template <class T, const bool is_assignable>
struct Assign;

template<class T>
struct ProxyType;

template<class T>
struct ProxyType {
    typedef void type;
};

template <class T>
struct Deleter {
    static void destroy(T* obj)
    {
        // 如果T的析构函数不是public, 如何释放资源，需要特例化Deleter
        delete obj;
    }
};

// 每个class都需要特例化MethodsType.
template <class T, class M>
struct MethodsType {
    typedef void methods_type;
};

template <class T, class M>
struct AbiClassMethods {
    typedef typename MethodsType<typename std::remove_cv<T>::type, M>::methods_type class_methods_type;
    void (*destroy)(AbiClass<T, M>);
    AbiClass<T, M> (*make_unique)(AbiClass<T, M>);
    AbiClass<T, M> (*make_ref)(T* obj);
    size_t (*size_of)();
    void (*write)(AbiClass<T, M>&, AbiClass<T, M>);
    class_methods_type class_methods;
    AbiClassMethods(void (*destroy)(AbiClass<T, M>), AbiClass<T, M> (*make_unique)(AbiClass<T, M>),
                    AbiClass<T, M> (*make_ref)(T* obj), size_t (*size_of)(),
                    void (*write)(AbiClass<T, M>&, AbiClass<T, M>))
        : destroy(destroy), make_unique(make_unique), make_ref(make_ref), size_of(size_of), write(write)
    {}
};

// C++:
// T, T*, T**, ...
// Rust:
// T, T*: ClassPtr<N = 1>
// T**: ClassPtr<N = 2>,
// T***: ClassPtr<N = 3>, ...
template <class T, class M>
struct AbiClass {
    template <const size_t N>
    struct NPtr;
    typedef typename is_pointer<T>::value_type value_type;
    typedef AbiClass<value_type, M> self_type;
    typedef typename is_pointer<T>::pointer_type pointer_type;
    typedef AbiClassMethods<value_type, M> methods_type;
    static const size_t N = is_pointer<pointer_type>::N - 1;
    AbiClass(const methods_type& methods, pointer_type obj) : methods(&methods)
    {
        static_assert(sizeof(void*) > N);
        set_obj(obj);
    }
    UniqueArg<AbiClass<T, M>> operator*() const { return UniqueArg<AbiClass<T, M>>(*this); }
    pointer_type get_obj() const
    {
        auto n = ((size_t)obj) & (sizeof(void*) - 1);
        assert(N == n);
        return (pointer_type)((size_t)obj & ~(sizeof(void*) - 1));
    }
    void set_obj(pointer_type obj) { this->obj = (pointer_type)(((size_t)obj) | N); }
    template <class Target, class TargetM = void>
    AbiClass<Target, TargetM> cast()
    {
        return AbiClass<Target, TargetM>(AbiClass<Target, TargetM>::no_destroy_methods(),
                                         dynamic_cast<Target*>(get_obj()));
    }
    template <class Target, class TargetM = void>
    AbiClass<Target, TargetM> cast_move()
    {
        if (methods != &no_destroy_methods()) {
            auto target = dynamic_cast<Target*>(get_obj());
            if (!target) {
                destroy();
            }
            if (methods == &destroy_methods()) {
                return AbiClass<Target, TargetM>(AbiClass<Target, TargetM>::destroy_methods(), target);
            } else {
                return AbiClass<Target, TargetM>(AbiClass<Target, TargetM>::dtor_methods(), target);
            }
        }
        return cast<Target, TargetM>();
    }
    T&& move_obj() { return std::move(*get_obj()); }
    const methods_type& get_methods() const { return *methods; }
    void set_methods(const methods_type& methods) { this->methods = &methods; }
    void destroy()
    {
        if (obj && N == 0) {
            methods->destroy(*(self_type*)this);
            obj = 0;
        }
    }
    static const methods_type& destroy_methods()
    {
        static methods_type methods(destroy, make_unique, make_ref, size_of, write);
        return methods;
    }
    static const methods_type& no_destroy_methods()
    {
        static methods_type methods(no_destroy, make_unique, make_ref, size_of, write);
        return methods;
    }
    static const methods_type& dtor_methods()
    {
        static methods_type methods(dtor, make_unique, make_ref, size_of, write);
        return methods;
    }

private:
    static void no_destroy(self_type) {}
    static void destroy(self_type input)
    {
        auto obj = input.get_obj();
        if (obj) Deleter<value_type>::destroy(obj);
    }
    static void dtor(self_type input)
    {
        auto obj = input.get_obj();
        if (obj) obj->~value_type();
    }
    static self_type make_unique(self_type input)
    {
        if (&input.get_methods() == &no_destroy_methods()) {
            input.set_methods(destroy_methods());
        }
        return input;
    }
    static void write(self_type& input, self_type value)
    {
        typedef typename std::remove_cv<value_type>::type type;
        Assign<type, std::is_assignable<type&, type>::value>::assign((type*)input.get_obj(), (type*)value.get_obj());
        value.destroy();
    }

    static self_type make_ref(pointer_type obj) { return self_type(no_destroy_methods(), obj); }
    static size_t size_of() { return sizeof(value_type); }
    const methods_type* methods = 0;
    pointer_type obj = 0;
};

template <class T>
struct Assign<T, true> {
    static void assign(T* self, T* val) { *self = std::move(*val); }
};

template <class T>
struct Assign<T, false> {
    static void assign(T* , T* ) { std::cerr << "can't assign new value" << std::endl; }
};

template <class T, class M>
struct UniqueArg<AbiClass<T, M>> {
    AbiClass<T, M> arg;
    UniqueArg(AbiClass<T, M> arg) : arg(arg) {}
    T&& operator*() const { return std::move(*arg.get_obj()); }
    ~UniqueArg() { arg.destroy(); }
};

template <class T, class M>
struct AbiClassType<AbiClass<T, M>> {
    typedef typename std::conditional<is_class<T, M>::value, AbiClass<T, M>, AbiClass<T>>::type type;
};

template <class T, class M>
struct ClassType {
    typedef AbiClass<T, M> input_type;
    typedef AbiClass<T, M> output_type;
    typedef typename std::conditional<std::is_abstract<T>::value, const T&, T>::type return_type;
    static output_type into(T&& val)
    {
        return output_type(AbiClass<T, M>::destroy_methods(), new (std::nothrow) T(std::forward<T>(val)));
    }
    static return_type from(const input_type& arg) { return **arg; }
};

template <class T, class M>
struct ClassType<T*, M> {
    typedef AbiClass<T*, M>* input_type;
    typedef AbiClass<T*, M> output_type;
    typedef typename AbiClass<T*, M>::pointer_type pointer_type;
    static output_type into(T* val) { return output_type(AbiClass<T*, M>::no_destroy_methods(), (pointer_type)val); }
    static T* from(const input_type& arg) { return arg ? (T*)arg->get_obj() : 0; }
};

template <class T, class M>
struct PlainType {
    typedef typename std::remove_cv<typename is_pointer<T>::value_type>::type value_type;
    typedef typename std::conditional<is_pod<value_type>::value, PodType<T>, ClassType<T, M>>::type type;
    typedef typename type::input_type input_type;
    typedef typename type::output_type output_type;
    typedef typename std::conditional<std::is_abstract<T>::value, const T&, T>::type return_type;
    static output_type into(T&& val) { return type::into(std::forward<T>(val)); }
    static return_type from(input_type&& arg) { return type::from(std::forward<input_type>(arg)); }
};

template <class T>
struct PodType {
    typedef T input_type;
    typedef T output_type;
    static output_type into(const T& val) { return val; }
    static T from(const input_type& arg) { return arg; }
};

template <class T>
struct is_pointer {
    typedef T value_type;
    typedef T* pointer_type;
    static const bool value = false;
    static const size_t N = 0;
};

template <class T>
struct is_pointer<T&&> {
    typedef typename is_pointer<T>::value_type value_type;
    typedef typename is_pointer<T>::pointer_type pointer_type;
    static const bool value = is_pointer<T>::value;
    static const size_t N = is_pointer<T>::N;
};

template <class T>
struct is_pointer<T*> {
    typedef typename is_pointer<T>::value_type value_type;
    typedef typename is_pointer<T>::pointer_type pointer_type;
    static const bool value = true;
    static const size_t N = is_pointer<T>::N + 1;
};

template <class T>
struct is_pointer<T&> {
    typedef typename is_pointer<T>::value_type value_type;
    typedef typename is_pointer<T>::pointer_type pointer_type;
    static const bool value = true;
    static const size_t N = is_pointer<T>::N + 1;
};

template <class R, class... ArgTypes>
struct is_pointer<R (*)(ArgTypes...)> {
    typedef R (*value_type)(ArgTypes...);
    typedef value_type pointer_type;
    static const bool value = true;
    static const size_t N = 1;
};

template <class T, class R, class... ArgTypes>
struct is_pointer<R (T::*)(ArgTypes...)> {
    typedef R (T::*value_type)(ArgTypes...);
    typedef value_type pointer_type;
    static const bool value = true;
    static const size_t N = 1;
};

template <class T, class R, class... ArgTypes>
struct is_pointer<R (T::*)(ArgTypes...) const> {
    typedef R (T::*value_type)(ArgTypes...) const;
    typedef value_type pointer_type;
    static const bool value = true;
    static const size_t N = 1;
};

template <class T, class R, class... ArgTypes>
struct is_pointer<R (T::*)(ArgTypes...) &&> {
    typedef R (T::*value_type)(ArgTypes...) &&;
    typedef value_type pointer_type;
    static const bool value = true;
    static const size_t N = 1;
};

template <class T, class M>
struct is_pointer<M T::*> {
    typedef M T::*value_type;
    typedef value_type pointer_type;
    static const bool value = true;
    static const size_t N = 1;
};

template <class T>
struct InputType {
    typedef T input_type;
};

template <class T, int N>
struct InputType<T[N]> {
    typedef T* input_type;
};

template <class T, class M, class V>
struct AbiType<T, M, V, std::tuple<void, void>> {
    typedef typename std::conditional<is_class<V>::value, ClassType<T>, PlainType<T, M>>::type abi_type;
};

template <class T, class M, class V, class M1>
struct AbiType<T, M, V, std::tuple<M1>> {
    typedef typename std::conditional<is_class<V, M1>::value, ClassType<T, M1>, PlainType<T, M>>::type abi_type;
};

template <class T, class M, class V, class... MM>
struct AbiType<T, M, V, std::tuple<std::tuple<MM...>>> {
    typedef typename AbiType<T, M, V, std::tuple<MM...>>::abi_type abi_type;
};

template <class T, class M, class V, class M1, class M2, class... MM>
struct AbiType<T, M, V, std::tuple<M1, M2, MM...>> {
    typedef typename std::conditional<is_class<V, M1>::value, ClassType<T, M1>,
                                      typename AbiType<T, M, V, std::tuple<M2, MM...>>::abi_type>::type abi_type;
};

template <class T, class M>
struct AbiValue {
    typedef typename AbiType<T, M, typename std::remove_cv<typename is_pointer<T>::value_type>::type,
                             std::tuple<void, M>>::abi_type value_type;
    typedef typename value_type::input_type input_type;
    typedef typename value_type::output_type output_type;
    typedef typename std::conditional<std::is_abstract<T>::value, const T&, T>::type return_type;
    static return_type from(input_type&& val) { return value_type::from(std::forward<input_type>(val)); }
    static return_type from_with(std::function<input_type()> fun) { return from(fun()); }
    static output_type into(T&& val) { return value_type::into(std::forward<T>(val)); }
    static output_type into_with(std::function<return_type()> fun) { return into(fun()); }
};

template <class M>
struct AbiValue<void, M> {
    typedef void output_type;
    typedef void input_type;
    static void into_with(std::function<void()> fun) { fun(); };
    static void from_with(std::function<void()> fun) { fun(); };
};

template <class T, class M>
struct AbiValue<T&, M> {
    typedef AbiValue<T*, M> value_type;
    typedef typename value_type::input_type input_type;
    typedef typename value_type::output_type output_type;
    static T& from(input_type&& val) { return *value_type::from(std::forward<input_type>(val)); }
    static T& from_with(std::function<input_type()> fun) { return from(fun()); }
    static output_type into(T& val) { return value_type::into(&val); }
    static output_type into_with(std::function<T&()> fun) { return into(fun()); }
};

template <class T, class M>
struct AbiValue<T&&, M> {
    typedef AbiValue<T, M> value_type;
    typedef typename value_type::input_type input_type;
    typedef typename value_type::output_type output_type;
    typedef typename std::conditional<std::is_abstract<T>::value, const T&, T>::type return_type;
    static return_type from(input_type&& val) { return value_type::from(std::forward<input_type>(val)); }
    static return_type from_with(std::function<input_type()> fun) { return from(fun()); }
    static output_type into(T&& val) { return value_type::into(std::forward<T>(val)); }
    static output_type into_with(std::function<T && ()> fun) { return into(fun()); }
};

template <class T, class M>
struct is_class {
    typedef typename std::remove_cv<typename is_pointer<T>::value_type>::type value_type;
    typedef typename MethodsType<value_type, M>::methods_type methods_type;
    static const bool value = !std::is_void<methods_type>::value;
};

template <class T>
struct is_pod {
    static const bool value = std::is_pod<T>::value;
};

template<>
struct is_pod<void> {
    static const bool value = true;
};

template <class T, class U>
struct is_pod<AbiClass<T, U>> {
    static const bool value = true;
};

template <class R, class M>
struct is_abi_function<R (*)(), M> {
    static const bool value = std::is_same<typename AbiValue<R, M>::output_type, R>::value;
};

template <class M, class R, class A, class... ArgTypes>
struct is_abi_function<R (*)(A, ArgTypes...), M> {
    static const bool value =
        std::is_same<typename AbiValue<A, M>::input_type, A>::value && is_abi_function<R (*)(ArgTypes...), M>::value;
};
}    // namespace hicc

#endif
