CXX ?= g++
CXXFLAGS ?= -std=c++17 -O2 -Wall -Wextra -I.
OUT ?= /tmp/variadic_functions_make

$(OUT): variadic_functions.cpp main.cpp variadic_functions.h
	$(CXX) $(CXXFLAGS) variadic_functions.cpp main.cpp -o $(OUT)

run: $(OUT)
	$(OUT)

clean:
	rm -f $(OUT)

.PHONY: run clean
