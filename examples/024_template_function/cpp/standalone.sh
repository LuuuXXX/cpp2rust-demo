#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"
g++ -std=c++17 -O2 -Wall -Wextra -I. \
    template_function.cpp main.cpp \
    -o /tmp/template_function_standalone
/tmp/template_function_standalone
