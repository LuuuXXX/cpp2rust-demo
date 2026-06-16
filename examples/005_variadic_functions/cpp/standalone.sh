#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"
g++ -std=c++17 -O2 -Wall -Wextra -I. \
    variadic_functions.cpp main.cpp \
    -o /tmp/variadic_functions_standalone
/tmp/variadic_functions_standalone
