#pragma once

class UniqueVector {
    int* data;
    int size;
public:
    UniqueVector();
    UniqueVector(int* data, int size);
    ~UniqueVector();
    UniqueVector(UniqueVector&& other) noexcept;
    UniqueVector& operator=(UniqueVector&& other) noexcept;
    int get(int index) const;
    void set(int index, int value);
    int getSize() const;
    void moveFrom(UniqueVector& src);
};
