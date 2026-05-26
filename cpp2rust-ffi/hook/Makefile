CXX ?= g++
CXXFLAGS = -O2 -fPIC -shared
LDFLAGS = -ldl

HOOK_LIB = libcpp2rust_hook.so
HOOK_SRC = hook.cpp

.PHONY: all clean

all: $(HOOK_LIB)

$(HOOK_LIB): $(HOOK_SRC)
	$(CXX) $(CXXFLAGS) -o $@ $< $(LDFLAGS)
	@echo "Built: $@"

clean:
	rm -f $(HOOK_LIB)

install: $(HOOK_LIB)
	cp $(HOOK_LIB) /usr/local/lib/
	@echo "Installed to /usr/local/lib/"

.PHONY: test
test: $(HOOK_LIB)
	@echo "Testing hook library..."
	@test -f $(HOOK_LIB) && echo "OK: $(HOOK_LIB) exists"
