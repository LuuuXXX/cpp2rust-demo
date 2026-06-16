#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"
g++ -std=c++17 -O2 -Wall -Wextra -I. \
    constexpr_basic.cpp main.cpp \
    -o ./constexpr_basic_standalone
./constexpr_basic_standalone
