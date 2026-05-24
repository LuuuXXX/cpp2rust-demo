// Example 11: Smart Pointers
// C++ features: std::unique_ptr, std::shared_ptr, std::weak_ptr, custom deleters

#include <memory>
#include <iostream>

class Resource {
private:
    int value;
public:
    Resource(int v) : value(v) {
        std::cout << "Resource(" << value << ") created" << std::endl;
    }
    ~Resource() {
        std::cout << "Resource(" << value << ") destroyed" << std::endl;
    }
    int get() const { return value; }
    void set(int v) { value = v; }
};

// Unique pointer factory
std::unique_ptr<Resource> create_unique(int value) {
    return std::make_unique<Resource>(value);
}

// Shared pointer factory
std::shared_ptr<Resource> create_shared(int value) {
    return std::make_shared<Resource>(value);
}

// Accept unique_ptr by value (transfer ownership)
void consume_unique(std::unique_ptr<Resource> p) {
    if (p) {
        std::cout << "Consuming unique: " << p->get() << std::endl;
    }
}

// Accept shared_ptr by reference (share ownership)
void use_shared(std::shared_ptr<Resource>& p) {
    if (p) {
        std::cout << "Using shared: " << p->get() << " (use_count=" << p.use_count() << ")" << std::endl;
    }
}

// Return shared_ptr reference
std::shared_ptr<Resource>& get_static_shared() {
    static std::shared_ptr<Resource> instance = create_shared(999);
    return instance;
}
