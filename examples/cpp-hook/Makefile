HOOK_SO = libhook.so
CXX = g++

all: $(HOOK_SO)

$(HOOK_SO): hook.cpp
	$(CXX) -shared -fPIC -o $@ $<

clean:
	rm -f $(HOOK_SO)

.PHONY: all clean
