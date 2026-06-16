CXX ?= g++
CXXFLAGS ?= -std=c++17 -O2 -Wall -Wextra -I.
OUT ?= /tmp/function_overload_make

$(OUT): function_overload.cpp main.cpp function_overload.h
$(CXX) $(CXXFLAGS) function_overload.cpp main.cpp -o $(OUT)

run: $(OUT)
$(OUT)

clean:
rm -f $(OUT)

.PHONY: run clean
