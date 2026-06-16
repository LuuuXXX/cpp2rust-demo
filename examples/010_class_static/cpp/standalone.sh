#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"
g++ -std=c++17 -O2 -Wall -Wextra -I. \
    class_static.cpp main.cpp \
    -o /tmp/class_static_standalone
/tmp/class_static_standalone
