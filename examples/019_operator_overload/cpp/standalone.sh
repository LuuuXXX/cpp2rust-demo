#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"
g++ -std=c++17 -O2 -Wall -Wextra -I. \
    operator_overload.cpp main.cpp \
    -o /tmp/operator_overload_standalone
/tmp/operator_overload_standalone
