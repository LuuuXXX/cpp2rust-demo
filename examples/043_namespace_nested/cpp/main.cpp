#include "namespace_nested.h"
#include <iostream>

int main() {
    foo::bar::config::ConfigManager config;
    config.set_value("timeout", 30);
    config.set_value("retry", 3);
    config.set_value("port", 8080);

    std::cout << "size=" << config.size()
              << " timeout=" << config.get_value("timeout")
              << " retry=" << config.get_value("retry") << "\n";
    std::cout << "missing=" << config.get_value("missing") << "\n";

    foo::baz::DataProcessor processor;
    std::cout << "process(5)=" << processor.process(5) << "\n";
    std::cout << "version=" << foo::get_version()
              << " build_number=" << foo::get_build_number() << "\n";
    return 0;
}
