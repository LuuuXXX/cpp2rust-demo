CXX ?= g++
CXXFLAGS ?= -std=c++17 -O2 -Wall -Wextra -I.
OUT ?= /tmp/class_static_make

$(OUT): class_static.cpp main.cpp class_static.h
$(CXX) $(CXXFLAGS) class_static.cpp main.cpp -o $(OUT)

run: $(OUT)
$(OUT)

clean:
rm -f $(OUT)

.PHONY: run clean
