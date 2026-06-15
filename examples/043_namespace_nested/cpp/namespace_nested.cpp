#include "namespace_nested.h"
#include <iostream>
#include <cstring>

namespace foo {
    namespace bar {
        namespace config {

            ConfigManager::ConfigManager() : count_(0) {
                for (size_t i = 0; i < MAX_ENTRIES; ++i) {
                    keys_[i] = nullptr;
                    values_[i] = 0;
                }
            }
            ConfigManager::~ConfigManager() {
                for (size_t i = 0; i < count_; ++i) {
                    if (keys_[i]) delete[] keys_[i];
                }
            }
            void ConfigManager::set_value(const char* key, int value) {
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
            int ConfigManager::get_value(const char* key) const {
                if (!key) return 0;
                for (size_t i = 0; i < count_; ++i) {
                    if (keys_[i] && strcmp(keys_[i], key) == 0) return values_[i];
                }
                return 0;
            }

        }
    }

    namespace baz {

        DataProcessor::DataProcessor() : multiplier_(1) {}
        DataProcessor::~DataProcessor() {}
        int DataProcessor::process(int input) const {
            return input * multiplier_;
        }

    }
}
