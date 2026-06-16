CXX ?= g++
CXXFLAGS ?= -std=c++17 -O2 -Wall -Wextra -I.
OUT ?= /tmp/inline_functions_make

$(OUT): inline_functions.cpp main.cpp inline_functions.h
$(CXX) $(CXXFLAGS) inline_functions.cpp main.cpp -o $(OUT)

run: $(OUT)
$(OUT)

clean:
rm -f $(OUT)

.PHONY: run clean
