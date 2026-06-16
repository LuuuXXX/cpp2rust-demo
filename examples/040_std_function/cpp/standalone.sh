#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"
g++ -std=c++17 -O2 -Wall -Wextra -I. \
    std_function.cpp main.cpp \
    -o ./std_function_standalone
./std_function_standalone
