CXX ?= g++
CXXFLAGS ?= -std=c++17 -O2 -Wall -Wextra -I.
OUT ?= /tmp/hello_world_make

$(OUT): hello_world.cpp main.cpp hello_world.h
$(CXX) $(CXXFLAGS) hello_world.cpp main.cpp -o $(OUT)

run: $(OUT)
$(OUT)

clean:
rm -f $(OUT)

.PHONY: run clean
