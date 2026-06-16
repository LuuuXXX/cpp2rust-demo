#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"
g++ -std=c++17 -O2 -Wall -Wextra -I. \
    inline_functions.cpp main.cpp \
    -o /tmp/inline_functions_standalone
/tmp/inline_functions_standalone
