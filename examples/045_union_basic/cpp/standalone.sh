#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"
g++ -std=c++17 -O2 -Wall -Wextra -I. \
    union_basic.cpp main.cpp \
    -o ./union_basic_standalone
trap 'rm -f ./union_basic_standalone' EXIT
./union_basic_standalone
