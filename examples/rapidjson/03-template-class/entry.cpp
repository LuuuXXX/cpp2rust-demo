// entry.cpp — rapidjson-03-template-class
//
// This synthetic translation unit demonstrates how cpp2rust-demo extracts
// a typedef-aliased template specialisation as a concrete ClassIR and emits
// import_class! bindings with canonical_name, ctor, and method list.
//
// Scene: ClassTemplateSpecializationDecl → ClassIR (is_template_specialization=true,
//        canonical_name="Document") → import_class! with ctor.
//
// Requires Phase 1 bare_template_name() fix to extract correctly.

namespace rjdoc {

// -----------------------------------------------------------------------
// Supporting types (simplified RapidJSON policy types)
// -----------------------------------------------------------------------

template <typename CharType = char>
struct UTF8 { typedef CharType Ch; };

struct CrtAllocator {};

template <typename BaseAllocator = CrtAllocator>
struct MemoryPoolAllocator {};

// -----------------------------------------------------------------------
// GenericValue — the core JSON value type
// -----------------------------------------------------------------------

template <typename Encoding, typename Allocator = MemoryPoolAllocator<CrtAllocator>>
class GenericValue {
public:
    // Type inspection
    bool isNull()   const;
    bool isBool()   const;
    bool isInt()    const;
    bool isString() const;
    bool isObject() const;
    bool isArray()  const;

    // Value accessors
    bool        getBool()   const;
    int         getInt()    const;
    const char* getString() const;

    // Mutators
    void setBool(bool b);
    void setInt(int n);
    void setString(const char* s, unsigned length);
};

/// Alias that unlocks GenericValue extraction.
typedef GenericValue<UTF8<char>> Value;

// -----------------------------------------------------------------------
// GenericDocument — extends GenericValue, adds parsing
// -----------------------------------------------------------------------

template <
    typename Encoding,
    typename Allocator = MemoryPoolAllocator<CrtAllocator>,
    typename StackAllocator = CrtAllocator>
class GenericDocument : public GenericValue<Encoding, Allocator> {
public:
    // Constructor (default)
    GenericDocument();

    // Parse from string
    bool parse(const char* json);
    bool parseInsitu(char* json);

    // Parse status
    bool hasParseError() const;
    int  getParseErrorOffset() const;

    // Document state
    bool isEmpty() const;
    void setNull();

    // Member access (non-operator form, for binding)
    bool hasMember(const char* name) const;
    Value& getMember(const char* name);

    // Array operations
    int  size() const;
    void clear();
};

/// Alias that unlocks GenericDocument extraction.
typedef GenericDocument<UTF8<char>> Document;

} // namespace rjdoc
