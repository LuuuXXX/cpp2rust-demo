#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"
g++ -std=c++17 -O2 -Wall -Wextra -I. \
    default_args.cpp main.cpp \
    -o /tmp/default_args_standalone
/tmp/default_args_standalone
