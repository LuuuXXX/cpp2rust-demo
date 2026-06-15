CXX ?= g++
CXXFLAGS ?= -std=c++17 -O2 -Wall -Wextra -I.
OUT ?= /tmp/class_constructor_make

$(OUT): class_constructor.cpp main.cpp class_constructor.h
	$(CXX) $(CXXFLAGS) class_constructor.cpp main.cpp -o $(OUT)

run: $(OUT)
	$(OUT)

clean:
	rm -f $(OUT)

.PHONY: run clean
