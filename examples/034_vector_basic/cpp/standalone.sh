#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"
g++ -std=c++17 -O2 -Wall -Wextra -I. \
    vector_basic.cpp main.cpp \
    -o /tmp/vector_basic_standalone
/tmp/vector_basic_standalone
