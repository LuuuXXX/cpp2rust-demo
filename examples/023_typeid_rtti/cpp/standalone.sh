#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"
g++ -std=c++17 -O2 -Wall -Wextra -I. \
    typeid_rtti.cpp main.cpp \
    -o /tmp/typeid_rtti_standalone
/tmp/typeid_rtti_standalone
