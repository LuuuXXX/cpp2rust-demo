// entry-errors.cpp — rapidjson-08-multi-tu
//
// Translation unit 3: ParseErrorCode enum.
// With real RapidJSON: #include "rapidjson/error/error.h"

/// Error codes returned by the JSON parser.
enum ParseErrorCode {
    kParseErrorNone = 0,
    kParseErrorDocumentEmpty,
    kParseErrorDocumentRootNotSingular,
    kParseErrorValueInvalid,
    kParseErrorObjectMissName,
    kParseErrorObjectMissColon,
    kParseErrorObjectMissCommaOrCurlyBracket,
    kParseErrorArrayMissCommaOrSquareBracket,
    kParseErrorTermination,
};

/// Return a human-readable description of a parse error.
const char* getParseErrorString(ParseErrorCode code);
