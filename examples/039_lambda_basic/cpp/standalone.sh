#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"
g++ -std=c++17 -O2 -Wall -Wextra -I. \
    lambda_basic.cpp main.cpp \
    -o /tmp/lambda_basic_standalone
/tmp/lambda_basic_standalone
