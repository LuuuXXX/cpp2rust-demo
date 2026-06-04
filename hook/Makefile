CXX = g++
UNAME_S := $(shell uname -s)

ifeq ($(UNAME_S),Darwin)
CXXFLAGS = -Wall -fPIC -dynamiclib
TARGET = libhook.dylib
LDLIBS =
else
CXXFLAGS = -Wall -fPIC -shared
TARGET = libhook.so
LDLIBS = -ldl
endif

all: $(TARGET)

$(TARGET): hook.cpp
	$(CXX) $(CXXFLAGS) -o $(TARGET) hook.cpp $(LDLIBS)

clean:
	rm -f libhook.so libhook.dylib

.PHONY: all clean
