#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"
g++ -std=c++17 -O2 -Wall -Wextra -I. \
    raii_pattern.cpp main.cpp \
    -o /tmp/raii_pattern_standalone
/tmp/raii_pattern_standalone
