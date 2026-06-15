CXX ?= g++
CXXFLAGS ?= -std=c++17 -O2 -Wall -Wextra -I.
OUT ?= /tmp/inheritance_multiple_make

$(OUT): inheritance_multiple.cpp main.cpp inheritance_multiple.h
$(CXX) $(CXXFLAGS) inheritance_multiple.cpp main.cpp -o $(OUT)

run: $(OUT)
$(OUT)

clean:
rm -f $(OUT)

.PHONY: run clean
