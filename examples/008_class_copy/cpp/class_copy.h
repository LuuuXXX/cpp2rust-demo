#pragma once

class Buffer {
    int* data;
    int size;
public:
    Buffer();
    explicit Buffer(int sz);
    Buffer(const Buffer& other);
    ~Buffer();
    void set(int index, int value);
    int get(int index) const;
    int getSize() const;
};
