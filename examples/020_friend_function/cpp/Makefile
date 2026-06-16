CXX ?= g++
CXXFLAGS ?= -std=c++17 -O2 -Wall -Wextra -I.
OUT ?= /tmp/friend_function_make

$(OUT): friend_function.cpp main.cpp friend_function.h
$(CXX) $(CXXFLAGS) friend_function.cpp main.cpp -o $(OUT)

run: $(OUT)
$(OUT)

clean:
rm -f $(OUT)

.PHONY: run clean
