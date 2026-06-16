#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"
g++ -std=c++17 -O2 -Wall -Wextra -I. \
    unique_ptr.cpp main.cpp \
    -o /tmp/unique_ptr_standalone
/tmp/unique_ptr_standalone
