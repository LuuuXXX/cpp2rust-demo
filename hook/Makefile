CXX = g++
CXXFLAGS = -Wall -fPIC -shared
TARGET = libhook.so

all: $(TARGET)

$(TARGET): hook.cpp
	$(CXX) $(CXXFLAGS) -o $(TARGET) hook.cpp -ldl

clean:
	rm -f $(TARGET)

.PHONY: all clean
