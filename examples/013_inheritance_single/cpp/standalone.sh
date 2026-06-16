#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"
g++ -std=c++17 -O2 -Wall -Wextra -I. \
    inheritance_single.cpp main.cpp \
    -o /tmp/inheritance_single_standalone
/tmp/inheritance_single_standalone
