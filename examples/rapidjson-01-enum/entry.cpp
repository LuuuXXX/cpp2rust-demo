// entry.cpp — rapidjson-01-enum
//
// This synthetic translation unit covers the RapidJSON ParseErrorCode enum
// scenario.  It directly defines equivalent enum types (no RapidJSON install
// required) so the example is fully self-contained.
//
// Scene: Extract C++ enum / enum class → Rust #[repr(C)] enum.

// -------------------------------------------------------------------
// Equivalent of rapidjson/error/error.h (standalone for this example)
// -------------------------------------------------------------------

/// Error codes returned by the RapidJSON parser.
/// (Mirrors rapidjson::ParseErrorCode)
enum ParseErrorCode {
    kParseErrorNone = 0,                //!< No error.

    kParseErrorDocumentEmpty,           //!< The document is empty.
    kParseErrorDocumentRootNotSingular, //!< The document root must not follow by other values.

    kParseErrorValueInvalid,            //!< Invalid value.

    kParseErrorObjectMissName,          //!< Missing a name for object member.
    kParseErrorObjectMissColon,         //!< Missing a colon after a name of object member.
    kParseErrorObjectMissCommaOrCurlyBracket, //!< Missing a comma or '}' after an object member.

    kParseErrorArrayMissCommaOrSquareBracket, //!< Missing a comma or ']' after an array element.

    kParseErrorStringUnicodeEscapeInvalidHex, //!< Incorrect hex digit after \\u escape in string.
    kParseErrorStringUnicodeSurrogateInvalid, //!< The surrogate pair in string is invalid.
    kParseErrorStringEscapeInvalid,     //!< Invalid escape character in string.
    kParseErrorStringMissQuotationMark, //!< Missing a closing quotation mark in string.
    kParseErrorStringInvalidEncoding,   //!< Invalid encoding in string.

    kParseErrorNumberTooBig,            //!< Number too big to be stored in double.
    kParseErrorNumberMissFraction,      //!< Miss fraction part in number.
    kParseErrorNumberMissExponent,      //!< Miss exponent in number.

    kParseErrorTermination,             //!< Parsing was terminated.
    kParseErrorUnspecificSyntaxError,   //!< Unspecific syntax error.
};

/// Scoped error code for write operations (enum class example).
/// (Illustrates how cpp2rust-demo handles C++11 enum class)
enum class WriteErrorCode {
    kWriteErrorNone = 0,       //!< No error.
    kWriteErrorInitFailed = 1, //!< Initialisation failed.
    kWriteErrorBufferFull = 2, //!< Output buffer is full.
};

/// Type of a JSON value.
/// (Mirrors rapidjson::Type)
enum Type {
    kNullType = 0,     //!< null
    kFalseType = 1,    //!< false
    kTrueType = 2,     //!< true
    kObjectType = 3,   //!< object
    kArrayType = 4,    //!< array
    kStringType = 5,   //!< string
    kNumberType = 6,   //!< number
};

// A simple helper so the TU has at least one function (avoids empty-file warnings).
const char* parseErrorName(ParseErrorCode code);
