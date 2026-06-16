#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"
g++ -std=c++17 -O2 -Wall -Wextra -I. \
    hello_world.cpp main.cpp \
    -o /tmp/hello_world_standalone
/tmp/hello_world_standalone
