// Example 16: Static Members
// C++ features: static member variables, static member functions, singleton pattern

#include <cstdint>

class Counter {
private:
    int count;
    static int total_count;
    static const int MAX_COUNT = 1000;

public:
    Counter() : count(0) {
        total_count++;
    }

    Counter(int initial) : count(initial) {
        total_count++;
    }

    ~Counter() {
        total_count--;
    }

    void increment() {
        if (count < MAX_COUNT) {
            count++;
        }
    }

    void decrement() {
        if (count > 0) {
            count--;
        }
    }

    int get() const {
        return count;
    }

    // Static member function
    static int get_total_count() {
        return total_count;
    }

    static int get_max_count() {
        return MAX_COUNT;
    }

    static void reset_total() {
        total_count = 0;
    }
};

int Counter::total_count = 0;

// Class with static factory
class Registry {
private:
    static Registry* instance;
    int id;

    Registry() : id(0) {}

public:
    static Registry* get_instance() {
        if (!instance) {
            instance = new Registry();
        }
        return instance;
    }

    static void delete_instance() {
        delete instance;
        instance = nullptr;
    }

    int get_id() const { return id; }
    void set_id(int i) { id = i; }
};

Registry* Registry::instance = nullptr;

// Class with static member in template
template<typename T>
class TemplateClass {
private:
    T value;
    static int instance_count;

public:
    TemplateClass(T v) : value(v) {
        instance_count++;
    }

    T get() const { return value; }

    static int get_count() {
        return instance_count;
    }
};

template<typename T>
int TemplateClass<T>::instance_count = 0;
