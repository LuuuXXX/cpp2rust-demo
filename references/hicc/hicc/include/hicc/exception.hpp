#ifndef HICC_EXCEPTION_H
#define HICC_EXCEPTION_H

#include <stdio.h>

#include <cstring>
#include <exception>
#include <functional>
#include <string>

namespace hicc {

template <class T>
struct ExceptValue {
    typedef T except_value_type;
    static except_value_type from(std::function<T()> f) { return f(); }
};

template <>
struct ExceptValue<void> {
    typedef int except_value_type;
    static except_value_type from(std::function<void()> f)
    {
        f();
        return 0;
    }
};

template <typename T>
class Exception {
public:
    union {
        char except[64];
        typename ExceptValue<T>::except_value_type value;
    };
    bool has_except;

public:
    Exception(std::function<T()> fun)
    {
        memset(except, 0, sizeof(except));
        has_except = true;
        try {
            value = ExceptValue<T>::from(fun);
            has_except = false;
        } catch (const std::exception& err) {
            snprintf(except, sizeof(except), "%s", err.what());
        } catch (const char* err) {
            snprintf(except, sizeof(except), "%s", err);
        } catch (const std::string& err) {
            snprintf(except, sizeof(except), "%s", err.c_str());
        } catch (int err) {
            snprintf(except, sizeof(except), "int(%d)", err);
        } catch (long err) {
            snprintf(except, sizeof(except), "long(%ld)", err);
        } catch (long long err) {
            snprintf(except, sizeof(except), "long long(%lld)", err);
        } catch (short err) {
            snprintf(except, sizeof(except), "short(%hd)", err);
        } catch (unsigned int err) {
            snprintf(except, sizeof(except), "unsigned int(%u)", err);
        } catch (unsigned long err) {
            snprintf(except, sizeof(except), "unsigned long(%lu)", err);
        } catch (unsigned long long err) {
            snprintf(except, sizeof(except), "unsigned long long(%llu)", err);
        } catch (unsigned short err) {
            snprintf(except, sizeof(except), "unsigned short(%hu)", err);
        } catch (...) {
            snprintf(except, sizeof(except), "%s", "unknown exception");
        }
    }

    const char* what() const noexcept(true) { return except; }

    T into() noexcept(false)
    {
        if (has_except) {
            throw this;
        }
        return value;
    }
};

}    // namespace hicc

#endif
