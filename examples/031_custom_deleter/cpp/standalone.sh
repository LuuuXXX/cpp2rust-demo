#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"
g++ -std=c++17 -O2 -Wall -Wextra -I. \
    custom_deleter.cpp main.cpp \
    -o /tmp/custom_deleter_standalone
/tmp/custom_deleter_standalone
