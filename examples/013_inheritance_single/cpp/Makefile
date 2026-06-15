CXX ?= g++
CXXFLAGS ?= -std=c++17 -O2 -Wall -Wextra -I.
OUT ?= /tmp/inheritance_single_make

$(OUT): inheritance_single.cpp main.cpp inheritance_single.h
$(CXX) $(CXXFLAGS) inheritance_single.cpp main.cpp -o $(OUT)

run: $(OUT)
$(OUT)

clean:
rm -f $(OUT)

.PHONY: run clean
