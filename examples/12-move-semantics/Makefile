CXX = g++
CXXFLAGS = -c -std=c++17 -Wall -fPIC
TARGET = libexample.so
SOURCES = main.cpp
OBJECTS = main.o

all: $(TARGET)

$(TARGET): $(OBJECTS)
	$(CXX) -shared -o $@ $^

main.o: main.cpp
	$(CXX) $(CXXFLAGS) -o $@ main.cpp

clean:
	rm -f $(TARGET) $(OBJECTS)

.PHONY: all clean
