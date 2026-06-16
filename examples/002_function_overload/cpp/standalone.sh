#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"
g++ -std=c++17 -O2 -Wall -Wextra -I. \
    function_overload.cpp main.cpp \
    -o /tmp/function_overload_standalone
/tmp/function_overload_standalone
