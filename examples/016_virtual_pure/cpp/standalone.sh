#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"
g++ -std=c++17 -O2 -Wall -Wextra -I. \
    virtual_pure.cpp main.cpp \
    -o /tmp/virtual_pure_standalone
/tmp/virtual_pure_standalone
