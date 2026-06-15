CXX ?= g++
CXXFLAGS ?= -std=c++17 -O2 -Wall -Wextra -I.
OUT ?= /tmp/class_basic_make

$(OUT): class_basic.cpp main.cpp class_basic.h
	$(CXX) $(CXXFLAGS) class_basic.cpp main.cpp -o $(OUT)

run: $(OUT)
	$(OUT)

clean:
	rm -f $(OUT)

.PHONY: run clean
