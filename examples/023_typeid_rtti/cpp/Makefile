CXX ?= g++
CXXFLAGS ?= -std=c++17 -O2 -Wall -Wextra -I.
OUT ?= /tmp/typeid_rtti_make

$(OUT): typeid_rtti.cpp main.cpp typeid_rtti.h
$(CXX) $(CXXFLAGS) typeid_rtti.cpp main.cpp -o $(OUT)

run: $(OUT)
$(OUT)

clean:
rm -f $(OUT)

.PHONY: run clean
