CXX ?= g++
CXXFLAGS ?= -std=c++17 -O2 -Wall -Wextra -I.
OUT ?= /tmp/virtual_pure_make

$(OUT): virtual_pure.cpp main.cpp virtual_pure.h
$(CXX) $(CXXFLAGS) virtual_pure.cpp main.cpp -o $(OUT)

run: $(OUT)
$(OUT)

clean:
rm -f $(OUT)

.PHONY: run clean
