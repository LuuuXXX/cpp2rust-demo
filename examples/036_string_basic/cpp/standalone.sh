#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"
g++ -std=c++17 -O2 -Wall -Wextra -I. \
    string_basic.cpp main.cpp \
    -o /tmp/string_basic_standalone
/tmp/string_basic_standalone
