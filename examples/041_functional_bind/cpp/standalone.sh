#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"
g++ -std=c++17 -O2 -Wall -Wextra -I. \
    functional_bind.cpp main.cpp \
    -o ./functional_bind_standalone
./functional_bind_standalone
rm -f ./functional_bind_standalone
