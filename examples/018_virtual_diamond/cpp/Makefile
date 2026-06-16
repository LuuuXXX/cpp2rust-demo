CXX ?= g++
CXXFLAGS ?= -std=c++17 -O2 -Wall -Wextra -I.
OUT ?= /tmp/virtual_diamond_make

$(OUT): virtual_diamond.cpp main.cpp virtual_diamond.h
$(CXX) $(CXXFLAGS) virtual_diamond.cpp main.cpp -o $(OUT)

run: $(OUT)
$(OUT)

clean:
rm -f $(OUT)

.PHONY: run clean
