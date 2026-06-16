#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"
g++ -std=c++17 -O2 -Wall -Wextra -I. \
    explicit_ctor.cpp main.cpp \
    -o /tmp/explicit_ctor_standalone
/tmp/explicit_ctor_standalone
