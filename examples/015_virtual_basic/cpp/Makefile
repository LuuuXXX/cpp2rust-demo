CXX ?= g++
CXXFLAGS ?= -std=c++17 -O2 -Wall -Wextra -I.
OUT ?= /tmp/virtual_basic_make

$(OUT): virtual_basic.cpp main.cpp virtual_basic.h
$(CXX) $(CXXFLAGS) virtual_basic.cpp main.cpp -o $(OUT)

run: $(OUT)
$(OUT)

clean:
rm -f $(OUT)

.PHONY: run clean
