#pragma once

#include <string>
class UniqueBuffer {
    std::string data;
public:
    UniqueBuffer(int sz);
    ~UniqueBuffer();
    int getSize() const;
    char* getData();
    UniqueBuffer move();
    int useCount() const;
};

class Processor {
    std::string buffer;
public:
    Processor();
    ~Processor();
    char* process(const char* input);
};
