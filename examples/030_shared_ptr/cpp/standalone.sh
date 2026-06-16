#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"
g++ -std=c++17 -O2 -Wall -Wextra -I. \
    shared_ptr.cpp main.cpp \
    -o /tmp/shared_ptr_standalone
/tmp/shared_ptr_standalone
