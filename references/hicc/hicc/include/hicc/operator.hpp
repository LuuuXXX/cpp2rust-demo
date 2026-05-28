#ifndef HICC_OPERATOR_H
#define HICC_OPERATOR_H

namespace hicc {

template <class T>
static inline T make_clone(const T& val)
{
    return val;
}

template <class T, class... ArgTypes>
static inline T make_constructor(ArgTypes&&... args)
{
    return T(std::forward<ArgTypes>(args)...);
}

template <class T1, class T2>
static inline void make_assign(T1& self, const T2& val)
{
    self = val;
}

template <class T1, class T2>
static inline bool make_eq(const T1& v1, const T2& v2)
{
    return v1 == v2;
}

template <class T1, class T2>
static inline bool make_neq(const T1& v1, const T2& v2)
{
    return v1 != v2;
}

template <class T1, class T2>
static inline bool make_lt(const T1& v1, const T2& v2)
{
    return v1 < v2;
}

template <class T1, class T2>
static inline bool make_le(const T1& v1, const T2& v2)
{
    return v1 <= v2;
}

template <class T1, class T2>
static inline bool make_gt(const T1& v1, const T2& v2)
{
    return v1 > v2;
}

template <class T1, class T2>
static inline bool make_ge(const T1& v1, const T2& v2)
{
    return v1 >= v2;
}

template <class T1>
static inline bool make_not(const T1& val)
{
    return !val;
}

}    // namespace hicc

#endif
