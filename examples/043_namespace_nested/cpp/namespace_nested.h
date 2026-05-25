#pragma once

#include <cstddef>
#include <cstring>

#ifdef __cplusplus
extern "C" {
#endif

// FFI opaque pointers using void*
void* config_manager_new(void);
void config_manager_delete(void* self);
void config_manager_set_value(void* self, const char* key, int value);
int config_manager_get_value(void* self, const char* key);

int string_length(const char* str);

void* data_processor_new(void);
void data_processor_delete(void* self);
int data_processor_process(void* self, int input);

const char* get_version(void);
int get_build_number(void);

#ifdef __cplusplus
}

// Full class definitions
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
