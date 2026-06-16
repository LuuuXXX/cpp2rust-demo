#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"
g++ -std=c++17 -O2 -Wall -Wextra -I. \
    inheritance_multiple.cpp main.cpp \
    -o /tmp/inheritance_multiple_standalone
/tmp/inheritance_multiple_standalone
