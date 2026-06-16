CXX ?= g++
CXXFLAGS ?= -std=c++17 -O2 -Wall -Wextra -I.
OUT ?= /tmp/explicit_ctor_make

$(OUT): explicit_ctor.cpp main.cpp explicit_ctor.h
$(CXX) $(CXXFLAGS) explicit_ctor.cpp main.cpp -o $(OUT)

run: $(OUT)
$(OUT)

clean:
rm -f $(OUT)

.PHONY: run clean
