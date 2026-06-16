#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"
g++ -std=c++17 -O2 -Wall -Wextra -I. \
    mutable_member.cpp main.cpp \
    -o /tmp/mutable_member_standalone
/tmp/mutable_member_standalone
