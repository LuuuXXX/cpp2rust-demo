CXX ?= g++
CXXFLAGS ?= -std=c++17 -O2 -Wall -Wextra -I.
OUT ?= /tmp/class_volatile_make

$(OUT): class_volatile.cpp main.cpp class_volatile.h
$(CXX) $(CXXFLAGS) class_volatile.cpp main.cpp -o $(OUT)

run: $(OUT)
$(OUT)

clean:
rm -f $(OUT)

.PHONY: run clean
