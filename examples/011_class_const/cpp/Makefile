CXX ?= g++
CXXFLAGS ?= -std=c++17 -O2 -Wall -Wextra -I.
OUT ?= /tmp/class_const_make

$(OUT): class_const.cpp main.cpp class_const.h
$(CXX) $(CXXFLAGS) class_const.cpp main.cpp -o $(OUT)

run: $(OUT)
$(OUT)

clean:
rm -f $(OUT)

.PHONY: run clean
