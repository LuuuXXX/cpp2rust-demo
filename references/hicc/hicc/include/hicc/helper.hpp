#ifndef HICC_HELPER_H
#define HICC_HELPER_H

#include "exception.hpp"
#include "export.hpp"
#include "functional.hpp"
#include "types.hpp"

#define CONCAT_(n, m) n##m
#define CONCAT(n, m) CONCAT_(n, m)

#define METHOD_NAME_(n, m) CONCAT(n, m)
#define METHOD_NAME(n) METHOD_NAME_(_hicc_, n)

#define EXPORT_METHODS_BEG(libname)                                                    \
    namespace CONCAT(_hicc_ns_, libname)                                               \
    {                                                                                  \
        struct _hicc_export_methods;                                                   \
        struct ExportMethods {                                                         \
            static const struct _hicc_export_methods g_hicc_export_methods;            \
        };                                                                             \
        extern "C" {                                                                   \
        const struct _hicc_export_methods* CONCAT(_hicc_export_methods_lib, libname)() \
        {                                                                              \
            return &ExportMethods::g_hicc_export_methods;                              \
        }                                                                              \
        }                                                                              \
        struct _hicc_export_methods

#define EXPORT_METHOD(...) EXPORT_NAMED_METHOD_IN(CONCAT(fname, __LINE__), void, ExportMethods, (__VA_ARGS__))

#define EXPORT_METHOD_IN(M, T, ...) EXPORT_NAMED_METHOD_IN(CONCAT(fname_, __LINE__), M, T, (__VA_ARGS__))

#define EXPORT_NAMED_METHOD(name, ...) EXPORT_NAMED_METHOD_IN(name, void, ExportMethods, (__VA_ARGS__))

#define EXPORT_NAMED_METHOD_IN(name, M, T, ...) \
    const void* name = (const void*)(::hicc::export_method<__LINE__, M, T>((__VA_ARGS__)))

#define EXPORT_DATA(...) EXPORT_NAMED_DATA_IN(CONCAT(fname, __LINE__), void, ExportMethods, (__VA_ARGS__))

#define EXPORT_DATA_IN(M, T, ...) EXPORT_NAMED_DATA_IN(CONCAT(fname_, __LINE__), M, T, (__VA_ARGS__))

#define EXPORT_NAMED_DATA(name, ...) EXPORT_NAMED_DATA_IN(name, void, ExportMethods, (__VA_ARGS__))

#define EXPORT_NAMED_DATA_IN(name, M, T, ...) \
    const void* name = (const void*)(::hicc::export_data<__LINE__, M, T>((__VA_ARGS__)))

#define EXPORT_CONST_DATA(...) EXPORT_NAMED_CONST_DATA_IN(CONCAT(fname, __LINE__), void, ExportMethods, (__VA_ARGS__))

#define EXPORT_CONST_DATA_IN(M, T, ...) EXPORT_NAMED_CONST_DATA_IN(CONCAT(fname_, __LINE__), M, T, (__VA_ARGS__))

#define EXPORT_NAMED_CONST_DATA(name, ...) EXPORT_NAMED_CONST_DATA_IN(name, void, ExportMethods, (__VA_ARGS__))

#define EXPORT_NAMED_CONST_DATA_IN(name, M, T, ...) \
    const void* name = (const void*)(::hicc::export_const_data<__LINE__, M, T>((__VA_ARGS__)))

#define EXPORT_VARIADIC(...) EXPORT_METHOD((__VA_ARGS__))

#define EXPORT_NAMED_VARIADIC(name, ...) EXPORT_NAMEDMETHOD(name, (__VA_ARGS__))

#define EXPORT_EXCEPT_METHOD(...) EXPORT_EXCEPT_METHOD_IN(void, ExportMethods, (__VA_ARGS__))

#define EXPORT_EXCEPT_METHOD_IN(M, T, ...) EXPORT_NAMED_EXCEPT_METHOD_IN(CONCAT(fname_, __LINE__), M, T, (__VA_ARGS__))

#define EXPORT_NAMED_EXCEPT_METHOD(name, ...) EXPORT_NAMED_EXCEPT_METHOD_IN(name, void, ExportMethods, (__VA_ARGS__))

#define EXPORT_NAMED_EXCEPT_METHOD_IN(name, M, T, ...) \
    const void* name = (const void*)::hicc::export_except_method<__LINE__, M, T>((__VA_ARGS__))

#define EXPORT_DYNAMIC(T1, T2) EXPORT_NAMED_DYNAMIC(CONCAT(fname_, __LINE__), T1, T2)

// 导出类的成员方法

#define EXPORT_MEMBER_METHOD(...) EXPORT_MEMBER_METHOD_IN(void, (__VA_ARGS__))

#define EXPORT_MEMBER_METHOD_IN(M, ...) EXPORT_NAMED_MEMBER_METHOD_IN(CONCAT(fname_, __LINE__), M, (__VA_ARGS__))

#define EXPORT_NAMED_MEMBER_METHOD(name, ...) EXPORT_NAMED_MEMBER_METHOD_IN(name, void, (__VA_ARGS__))

#define EXPORT_NAMED_MEMBER_METHOD_IN(name, M, ...) \
    const void* name = (const void*)(::hicc::export_method<__LINE__, M>((__VA_ARGS__)))

#define EXPORT_MEMBER_DATA(...) EXPORT_MEMBER_DATA_IN(void, (__VA_ARGS__))

#define EXPORT_MEMBER_DATA_IN(M, ...) EXPORT_NAMED_MEMBER_DATA_IN(CONCAT(fname_, __LINE__), M, (__VA_ARGS__))

#define EXPORT_NAMED_MEMBER_DATA(name, ...) EXPORT_NAMED_MEMBER_DATA_IN(name, void, (__VA_ARGS__))

#define EXPORT_NAMED_MEMBER_DATA_IN(name, M, ...) \
    const void* name = (const void*)(::hicc::export_data<__LINE__, M>((__VA_ARGS__)))

#define EXPORT_MEMBER_CONST_DATA(...) EXPORT_MEMBER_CONST_DATA_IN(void, (__VA_ARGS__))

#define EXPORT_MEMBER_CONST_DATA_IN(M, ...) \
    EXPORT_NAMED_MEMBER_CONST_DATA_IN(CONCAT(fname_, __LINE__), M, (__VA_ARGS__))

#define EXPORT_NAMED_MEMBER_CONST_DATA(name, ...) EXPORT_NAMED_MEMBER_CONST_DATA_IN(name, void, (__VA_ARGS__))

#define EXPORT_NAMED_MEMBER_CONST_DATA_IN(name, M, ...) \
    const void* name = (const void*)(::hicc::export_const_data<__LINE__, M>((__VA_ARGS__)))

#define EXPORT_EXCEPT_MEMBER_METHOD(...) EXPORT_EXCEPT_MEMBER_METHOD_IN(void, (__VA_ARGS__))

#define EXPORT_EXCEPT_MEMBER_METHOD_IN(M, ...) \
    EXPORT_NAMED_EXCEPT_MEMBER_METHOD_IN(CONCAT(fname_, __LINE__), M, (__VA_ARGS__))

#define EXPORT_NAMED_EXCEPT_MEMBER_METHOD(name, ...) EXPORT_NAMED_EXCEPT_MEMBER_METHOD_IN(name, void, (__VA_ARGS__))

#define EXPORT_NAMED_EXCEPT_MEMBER_METHOD_IN(name, M, ...) \
    const void* name = (const void*)::hicc::export_except_method<__LINE__, M>((__VA_ARGS__))

#define EXPORT_DYNAMIC(T1, T2) EXPORT_NAMED_DYNAMIC(CONCAT(fname_, __LINE__), T1, T2)

#define EXPORT_NAMED_DYNAMIC(name, T1, T2) const void* name = (const void*)::hicc::export_dynamic<T1, T2>()

#define EXPORT_DYNAMIC_MOVE(T1, T2) EXPORT_NAMED_DYNAMIC_MOVE(CONCAT(fname_, __LINE__), T1, T2)

#define EXPORT_NAMED_DYNAMIC_MOVE(name, T1, T2) const void* name = (const void*)::hicc::export_dynamic_move<T1, T2>()

#define EXPORT_METHODS_END()                                                \
    ;                                                                       \
    const struct _hicc_export_methods ExportMethods::g_hicc_export_methods; \
    }

#define EXPORT_CLASS_DECLARE(type, methods) \
    struct methods;                         \
    namespace hicc {                        \
    template <>                             \
    struct MethodsType<type> {              \
        typedef methods methods_type;       \
    };                                      \
    }

#define EXPORT_CLASS_DELETER(type, deleter)                \
    namespace hicc {                                       \
    template <>                                            \
    struct Deleter<type> {                                 \
        static void destroy(type* obj) { (deleter)(obj); } \
    };                                                     \
    }

#define EXPORT_CLASS_END()

#define CALL_REMOTE(type, method, abi_name, ...) CALL_REMOTE_METHOD((&type::method), abi_name, __VA_ARGS__)

#define CALL_REMOTE_METHOD(method, abi_name, ...)                                                            \
    return make_remote((const void*)_hicc_remote.get_methods().class_methods.abi_name, method, _hicc_remote) \
        .call(__VA_ARGS__);

#define CALL_REMOTE_BY_IDX(type, method, abi_idx, ...) CALL_REMOTE_METHOD_BY_IDX((&type::method), abi_idx, __VA_ARGS__)

#define CALL_REMOTE_METHOD_BY_IDX(method, abi_idx, ...)                                                            \
    return make_remote(((const void**)&_hicc_remote.get_methods().class_methods)[abi_idx], method, _hicc_remote) \
        .call(__VA_ARGS__);

#endif
