#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"
g++ -std=c++17 -O2 -Wall -Wextra -I. \
    virtual_diamond.cpp main.cpp \
    -o /tmp/virtual_diamond_standalone
/tmp/virtual_diamond_standalone
