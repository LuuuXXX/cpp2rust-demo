CXX ?= g++
CXXFLAGS ?= -std=c++17 -O2 -Wall -Wextra -I.
OUT ?= /tmp/template_function_make

$(OUT): template_function.cpp main.cpp template_function.h
$(CXX) $(CXXFLAGS) template_function.cpp main.cpp -o $(OUT)

run: $(OUT)
$(OUT)

clean:
rm -f $(OUT)

.PHONY: run clean
