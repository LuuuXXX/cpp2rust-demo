#pragma once

#ifdef __cplusplus
extern "C" {
#endif

struct UniqueBuffer;

UniqueBuffer* uniquebuffer_new(int size);
void uniquebuffer_delete(UniqueBuffer* self);

int uniquebuffer_size(UniqueBuffer* self);
char* uniquebuffer_data(UniqueBuffer* self);

UniqueBuffer* uniquebuffer_move(UniqueBuffer* self);
int uniquebuffer_use_count(UniqueBuffer* self);

struct Processor;

Processor* processor_new(void);
void processor_delete(Processor* self);
char* processor_process(Processor* self, const char* input);

#ifdef __cplusplus
}

// Full class definition - for hicc code generation
#include <string>
class UniqueBuffer {
    std::string data;
public:
    UniqueBuffer(int sz);
    ~UniqueBuffer();
    int getSize() const;
    char* getData();
    UniqueBuffer move();
    int useCount() const;
};

class Processor {
    std::string buffer;
public:
    Processor();
    ~Processor();
    char* process(const char* input);
};

#endif
