#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"
out="./exception_basic_standalone"
trap 'rm -f "$out"' EXIT
g++ -std=c++17 -O2 -Wall -Wextra -I. \
    exception_basic.cpp main.cpp \
    -o "$out"
"$out"
