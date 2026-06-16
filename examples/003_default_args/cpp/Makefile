CXX ?= g++
CXXFLAGS ?= -std=c++17 -O2 -Wall -Wextra -I.
OUT ?= /tmp/default_args_make

$(OUT): default_args.cpp main.cpp default_args.h
$(CXX) $(CXXFLAGS) default_args.cpp main.cpp -o $(OUT)

run: $(OUT)
$(OUT)

clean:
rm -f $(OUT)

.PHONY: run clean
