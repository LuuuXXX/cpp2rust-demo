#include "nlohmann_json_direct.h"

#include <cstring>

static char g_buffer[4096];

extern "C" const char* nlohmann_json_parse_and_dump(const char* json_str) {
    try {
        nlohmann::json j = nlohmann::json::parse(json_str);
        std::string dumped = j.dump();
        size_t len = dumped.size();
        if (len >= sizeof(g_buffer)) len = sizeof(g_buffer) - 1;
        memcpy(g_buffer, dumped.c_str(), len);
        g_buffer[len] = '\0';
        return g_buffer;
    } catch (...) {
        const char* err = "parse error";
        memcpy(g_buffer, err, strlen(err) + 1);
        return g_buffer;
    }
}
