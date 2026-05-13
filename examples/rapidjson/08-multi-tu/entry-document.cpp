// entry-document.cpp — rapidjson-08-multi-tu
//
// Translation unit 1: Document / Value types.
// With real RapidJSON: #include "rapidjson/document.h"
// This synthetic version defines equivalent types.

namespace rapidjson {

struct CrtAllocator {};
template <typename B = CrtAllocator> struct MemoryPoolAllocator {};
template <typename C = char> struct UTF8 { typedef C Ch; };

template <typename E, typename A = MemoryPoolAllocator<CrtAllocator>>
class GenericValue {
public:
    bool isNull()   const;
    bool isInt()    const;
    bool isString() const;
    int  getInt()   const;
    const char* getString() const;
    void setInt(int n);
};

typedef GenericValue<UTF8<char>> Value;

template <typename E, typename A = MemoryPoolAllocator<CrtAllocator>, typename S = CrtAllocator>
class GenericDocument : public GenericValue<E, A> {
public:
    GenericDocument();
    bool parse(const char* json);
    bool hasParseError() const;
    bool hasMember(const char* name) const;
    int  size() const;
};

typedef GenericDocument<UTF8<char>> Document;

} // namespace rapidjson
