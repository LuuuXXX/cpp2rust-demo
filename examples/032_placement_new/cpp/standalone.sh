#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"
g++ -std=c++17 -O2 -Wall -Wextra -I. \
    placement_new.cpp main.cpp \
    -o /tmp/placement_new_standalone
/tmp/placement_new_standalone
