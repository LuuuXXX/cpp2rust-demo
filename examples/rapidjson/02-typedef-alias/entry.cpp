// entry.cpp — rapidjson-02-typedef-alias
//
// This synthetic translation unit demonstrates how typedef/using aliases
// register template specialisations in cpp2rust-demo's AliasRegistry and
// thereby unlock subsequent template class extraction.
//
// Scene: AliasRegistry two-map lookup; bare_template_name() correctness.

// -------------------------------------------------------------------
// Mimic of rapidjson's typedef-heavy header structure
// (no RapidJSON install required)
// -------------------------------------------------------------------

namespace rjson {

// Encoding "policies" (mimic rapidjson::UTF8 / rapidjson::ASCII).
template <typename CharType = char>
struct UTF8 {
    typedef CharType Ch;
};

template <typename CharType = char>
struct ASCII {
    typedef CharType Ch;
};

// Allocator policy.
struct CrtAllocator {};
template <typename BaseAllocator = CrtAllocator>
struct MemoryPoolAllocator {};

// -------------------------------------------------------------------
// Core template types (mirrors rapidjson::GenericValue / GenericDocument)
// -------------------------------------------------------------------

template <typename Encoding, typename Allocator = MemoryPoolAllocator<CrtAllocator>>
class GenericValue {
public:
    int getType() const;
    bool isNull() const;
    bool isString() const;
    bool isInt() const;
    int  getInt() const;
    const char* getString() const;
};

template <
    typename Encoding,
    typename Allocator = MemoryPoolAllocator<CrtAllocator>,
    typename StackAllocator = CrtAllocator>
class GenericDocument : public GenericValue<Encoding, Allocator> {
public:
    GenericDocument();
    bool parse(const char* json);
    bool hasParseError() const;
    int  getParseErrorOffset() const;
    bool isEmpty() const;
};

// -------------------------------------------------------------------
// Canonical typedef aliases — these are what cpp2rust-demo registers
// -------------------------------------------------------------------

/// The primary typedef alias that unlocks GenericValue extraction.
/// After seeing this, AliasRegistry maps: "GenericValue" → "Value"
typedef GenericValue<UTF8<char>> Value;

/// The primary typedef alias that unlocks GenericDocument extraction.
/// After seeing this, AliasRegistry maps: "GenericDocument" → "Document"
typedef GenericDocument<UTF8<char>> Document;

// Additional using-alias syntax (C++11 style) — also collected.
using ScopedDoc = GenericDocument<ASCII<char>>;

} // namespace rjson

// -------------------------------------------------------------------
// Free functions that use the aliased types in their signatures.
// These will pass the type gate because the aliases are registered.
// -------------------------------------------------------------------

/// Returns the string representation of a value's type.
const char* valueTypeName(const rjson::Value& val);

/// Checks whether a document was parsed without errors.
bool documentOk(const rjson::Document& doc);
