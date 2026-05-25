CXX ?= g++
CXXFLAGS ?= -std=c++17 -Wall -Wextra -O2 -fPIC
TARGET = libcpp2rust_hook.so
SOURCES = hook.cpp

all: $(TARGET)

$(TARGET): $(SOURCES)
	$(CXX) $(CXXFLAGS) -shared -o $@ $(SOURCES)

clean:
	rm -f $(TARGET)

.PHONY: all clean
