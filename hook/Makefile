CXX     ?= g++
TARGET   = libhook.so
CXXFLAGS = -Wall -fPIC -shared
LDFLAGS  = -ldl

all: $(TARGET)

$(TARGET): hook.cpp
	$(CXX) $(CXXFLAGS) -o $(TARGET) hook.cpp $(LDFLAGS)

clean:
	rm -f libhook.so

.PHONY: all clean
