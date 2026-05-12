// entry-writer.cpp — rapidjson-08-multi-tu
//
// Translation unit 2: Writer / StringBuffer types.
// With real RapidJSON: #include "rapidjson/writer.h" + "rapidjson/stringbuffer.h"

namespace rapidjson {

// Forward declaration of Value (from entry-document.cpp)
template <typename E, typename A> class GenericValue;
template <typename C = char> struct UTF8 { typedef C Ch; };
struct CrtAllocator {};
template <typename B = CrtAllocator> struct MemoryPoolAllocator {};

/// String output buffer.
class StringBuffer {
public:
    StringBuffer();
    void        Clear();
    const char* GetString() const;
    size_t      GetSize() const;
    bool        Empty() const;
};

/// Writer that serialises JSON to an output stream.
template <typename OutputStream, typename SourceEncoding = UTF8<char>,
          typename TargetEncoding = UTF8<char>,
          typename StackAllocator = CrtAllocator>
class GenericWriter {
public:
    explicit GenericWriter(OutputStream& os);

    bool Null();
    bool Bool(bool b);
    bool Int(int i);
    bool String(const char* str, size_t length, bool copy = false);
    bool StartObject();
    bool EndObject(size_t member_count = 0);
    bool StartArray();
    bool EndArray(size_t element_count = 0);
    bool IsComplete() const;
    void Reset(OutputStream& os);
};

typedef GenericWriter<StringBuffer> Writer;

} // namespace rapidjson
