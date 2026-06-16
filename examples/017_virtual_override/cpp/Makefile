CXX ?= g++
CXXFLAGS ?= -std=c++17 -O2 -Wall -Wextra -I.
OUT ?= /tmp/virtual_override_make

$(OUT): virtual_override.cpp main.cpp virtual_override.h
$(CXX) $(CXXFLAGS) virtual_override.cpp main.cpp -o $(OUT)

run: $(OUT)
$(OUT)

clean:
rm -f $(OUT)

.PHONY: run clean
