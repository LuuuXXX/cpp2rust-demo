#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"
g++ -std=c++17 -O2 -Wall -Wextra -I. \
    friend_function.cpp main.cpp \
    -o /tmp/friend_function_standalone
/tmp/friend_function_standalone
