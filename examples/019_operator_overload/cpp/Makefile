CXX ?= g++
CXXFLAGS ?= -std=c++17 -O2 -Wall -Wextra -I.
OUT ?= /tmp/operator_overload_make

$(OUT): operator_overload.cpp main.cpp operator_overload.h
$(CXX) $(CXXFLAGS) operator_overload.cpp main.cpp -o $(OUT)

run: $(OUT)
$(OUT)

clean:
rm -f $(OUT)

.PHONY: run clean
