#pragma once

#include <cstddef>
#include <cstring>

#ifdef __cplusplus

namespace foo {
    namespace bar {
        namespace config {

            class ConfigManager {
            public:
                static constexpr size_t MAX_ENTRIES = 10;
            private:
                int values_[MAX_ENTRIES];
                const char* keys_[MAX_ENTRIES];
                size_t count_;
            public:
                ConfigManager();
                ~ConfigManager();
                void set_value(const char* key, int value);
                int get_value(const char* key) const;
            };

        }
    }

    namespace baz {

        class DataProcessor {
        private:
            int multiplier_;
        public:
            DataProcessor();
            ~DataProcessor();
            int process(int input) const;
        };

    }
}

#endif
