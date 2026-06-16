#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"
g++ -std=c++17 -O2 -Wall -Wextra -I. \
    array_basic.cpp main.cpp \
    -o /tmp/array_basic_standalone
/tmp/array_basic_standalone
