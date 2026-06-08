UNAME_S := $(shell uname -s)
ifeq ($(UNAME_S),Darwin)
    # macOS：使用 Xcode 自带的 clang++（支持 arm64e，Homebrew LLVM 不保证）
    CXX     ?= xcrun clang++
    TARGET   = libhook.dylib
    UNAME_M := $(shell uname -m)
    ifeq ($(UNAME_M),arm64)
        # Apple Silicon：同时编译 arm64 和 arm64e，确保库能注入进
        # arm64（如 Homebrew 工具）和 arm64e（如 /bin/bash、/usr/bin/make）进程。
        CXXFLAGS = -Wall -dynamiclib -arch arm64 -arch arm64e
    else
        CXXFLAGS = -Wall -dynamiclib
    endif
else
    CXX     ?= g++
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
