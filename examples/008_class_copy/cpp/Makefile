CXX ?= g++
CXXFLAGS ?= -std=c++17 -O2 -Wall -Wextra -I.
OUT ?= /tmp/class_copy_make

$(OUT): class_copy.cpp main.cpp class_copy.h
	$(CXX) $(CXXFLAGS) class_copy.cpp main.cpp -o $(OUT)

run: $(OUT)
	$(OUT)

clean:
	rm -f $(OUT)

.PHONY: run clean
