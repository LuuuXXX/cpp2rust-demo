// Example 10: STL Containers
// C++ features: std::vector, std::list, std::map, std::set, std::unordered_map, std::deque, std::pair

#include <vector>
#include <list>
#include <map>
#include <set>
#include <unordered_map>
#include <unordered_set>
#include <deque>
#include <string>

// Vector operations
int vector_sum(const std::vector<int>& v) {
    int sum = 0;
    for (int x : v) sum += x;
    return sum;
}

int vector_size(const std::vector<int>& v) {
    return static_cast<int>(v.size());
}

// List operations
int list_sum(const std::list<int>& l) {
    int sum = 0;
    for (int x : l) sum += x;
    return sum;
}

// Map operations
int map_get(const std::map<int, int>& m, int key) {
    auto it = m.find(key);
    if (it != m.end()) {
        return it->second;
    }
    return -1;
}

// Set operations
bool set_contains(const std::set<int>& s, int key) {
    return s.find(key) != s.end();
}

// Deque operations
int deque_get(const std::deque<int>& d, int idx) {
    if (idx >= 0 && idx < static_cast<int>(d.size())) {
        return d[idx];
    }
    return 0;
}

// Pair operations
std::pair<int, int> make_pair_int(int a, int b) {
    return std::make_pair(a, b);
}

int pair_first(const std::pair<int, int>& p) {
    return p.first;
}

int pair_second(const std::pair<int, int>& p) {
    return p.second;
}

// Unordered map
int umap_get(const std::unordered_map<int, int>& m, int key) {
    auto it = m.find(key);
    if (it != m.end()) {
        return it->second;
    }
    return -1;
}
