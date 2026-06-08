CXX ?= g++

UNAME_S := $(shell uname -s)
ifeq ($(UNAME_S),Darwin)
    TARGET   = libhook.dylib
    CXXFLAGS = -Wall -dynamiclib
else
    TARGET   = libhook.so
    CXXFLAGS = -Wall -fPIC -shared
    LDFLAGS  = -ldl
endif

all: $(TARGET)

$(TARGET): hook.cpp
	$(CXX) $(CXXFLAGS) -o $(TARGET) hook.cpp $(LDFLAGS)

clean:
	rm -f libhook.so libhook.dylib

.PHONY: all clean
