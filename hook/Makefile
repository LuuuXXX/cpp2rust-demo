CXX ?= g++
CXXFLAGS := -shared -fPIC -ldl -O2 -Wall
TARGET := libcpp2rust_hook.so

.PHONY: all clean

all: $(TARGET)

$(TARGET): hook.cpp
	$(CXX) $(CXXFLAGS) -o $@ $<
	@echo "Built $(TARGET)"

clean:
	rm -f $(TARGET)
