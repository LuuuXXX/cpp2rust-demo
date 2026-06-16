#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"
g++ -std=c++17 -O2 -Wall -Wextra -I. \
    class_basic.cpp main.cpp \
    -o /tmp/class_basic_standalone
/tmp/class_basic_standalone
