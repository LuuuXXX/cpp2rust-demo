CXX ?= g++
CXXFLAGS ?= -std=c++17 -O2 -Wall -Wextra -I.
OUT ?= /tmp/mutable_member_make

$(OUT): mutable_member.cpp main.cpp mutable_member.h
$(CXX) $(CXXFLAGS) mutable_member.cpp main.cpp -o $(OUT)

run: $(OUT)
$(OUT)

clean:
rm -f $(OUT)

.PHONY: run clean
