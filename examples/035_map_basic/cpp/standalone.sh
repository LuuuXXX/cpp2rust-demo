#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"
g++ -std=c++17 -O2 -Wall -Wextra -I. \
    map_basic.cpp main.cpp \
    -o ./map_basic_standalone
./map_basic_standalone
