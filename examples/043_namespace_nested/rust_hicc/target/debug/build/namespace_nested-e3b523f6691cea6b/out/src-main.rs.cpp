#include <hicc/hicc.hpp>
#line 0 R"(src/main.rs)"
#line 5
#include <iostream>
    #include <cstring>

    namespace foo { namespace bar { namespace config {

        class ConfigManager {
        public:
            static constexpr size_t MAX_ENTRIES = 10;
        private:
            int values_[MAX_ENTRIES];
            const char* keys_[MAX_ENTRIES];
            size_t count_;
        public:
            ConfigManager() : count_(0) {
                for (size_t i = 0; i < MAX_ENTRIES; ++i) {
                    keys_[i] = nullptr;
                    values_[i] = 0;
                }
            }
            ~ConfigManager() {
                for (size_t i = 0; i < count_; ++i) {
                    if (keys_[i]) delete[] keys_[i];
                }
            }
            void set_value(const char* key, int value) {
                if (!key) return;
                for (size_t i = 0; i < count_; ++i) {
                    if (keys_[i] && strcmp(keys_[i], key) == 0) {
                        values_[i] = value;
                        return;
                    }
                }
                if (count_ < MAX_ENTRIES) {
                    size_t len = strlen(key);
                    char* new_key = new char[len + 1];
                    strcpy(new_key, key);
                    keys_[count_] = new_key;
                    values_[count_] = value;
                    ++count_;
                }
            }
            int get_value(const char* key) const {
                if (!key) return 0;
                for (size_t i = 0; i < count_; ++i) {
                    if (keys_[i] && strcmp(keys_[i], key) == 0) return values_[i];
                }
                return 0;
            }
        };

    }}}

    namespace foo { namespace baz {

        class DataProcessor {
        private:
            int multiplier_;
        public:
            DataProcessor() : multiplier_(1) {}
            ~DataProcessor() {}
            int process(int input) const {
                return input * multiplier_;
            }
        };

    }}

