#pragma once

#include <string>
#include <unordered_map>

class SharedData {
    std::string name_;
public:
    int value;
    SharedData(const char* n);
    ~SharedData();
    int useCount() const;
    const char* getName() const;
    SharedData* clone() const;
    void reset();
};

class Cache {
    std::unordered_map<std::string, void*> data_;
public:
    Cache();
    ~Cache();
    SharedData* get(const char* name);
};
