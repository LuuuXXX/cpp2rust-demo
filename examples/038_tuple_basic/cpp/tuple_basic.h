#pragma once

#include <tuple>
#include <string>

class Tuple2Impl {
public:
    std::tuple<int, std::string> data;
    Tuple2Impl(int first, const char* second);
    ~Tuple2Impl();
};

class Tuple3Impl {
public:
    std::tuple<int, double, std::string> data;
    Tuple3Impl(int first, double second, const char* third);
    ~Tuple3Impl();
};

class Tuple4Impl {
public:
    std::tuple<int, double, std::string, int> data;
    Tuple4Impl(int first, double second, const char* third, int fourth);
    ~Tuple4Impl();
};

struct Tuple2 {
    Tuple2Impl* impl;
    explicit Tuple2(int first, const char* second);
    ~Tuple2();
    int get_first() const { return std::get<0>(impl->data); }
    const char* get_second() const { return std::get<1>(impl->data).c_str(); }
};

struct Tuple3 {
    Tuple3Impl* impl;
    explicit Tuple3(int first, double second, const char* third);
    ~Tuple3();
    int get_first() const { return std::get<0>(impl->data); }
    double get_second() const { return std::get<1>(impl->data); }
    const char* get_third() const { return std::get<2>(impl->data).c_str(); }
};

struct Tuple4 {
    Tuple4Impl* impl;
    explicit Tuple4(int first, double second, const char* third, int fourth);
    ~Tuple4();
    int get_first() const { return std::get<0>(impl->data); }
    double get_second() const { return std::get<1>(impl->data); }
    const char* get_third() const { return std::get<2>(impl->data).c_str(); }
    int get_fourth() const { return std::get<3>(impl->data); }
};
