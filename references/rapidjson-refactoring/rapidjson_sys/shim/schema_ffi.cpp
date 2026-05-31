// FFI shim implementation for rapidjson SchemaDocument / SchemaValidator.

#include "schema_ffi.h"
#include "document_ffi.h"

#include <new>
#include <cstring>

#include "rapidjson/document.h"
#include "rapidjson/schema.h"
#include "rapidjson/pointer.h"
#include "rapidjson/stringbuffer.h"
#include "rapidjson/writer.h"

using namespace rapidjson;

typedef GenericDocument<UTF8<>, CrtAllocator> CrtDocument;
typedef SchemaDocument CrtSchemaDocument;

struct RapidJsonDocumentHandle {
    CrtDocument doc;
};

struct RapidJsonSchemaDocHandle {
    CrtSchemaDocument sd;
    RapidJsonSchemaDocHandle(const Document& d) : sd(d) {}
};

struct RapidJsonSchemaValidHandle {
    SchemaValidator sv;
    explicit RapidJsonSchemaValidHandle(const CrtSchemaDocument& sd) : sv(sd) {}
};

// Helper: serialize CrtDocument to JSON then re-parse into a standard Document
// (which uses MemoryPoolAllocator, required by SchemaDocument).
static bool build_schema_doc(const RapidJsonDocumentHandle* src,
                              RapidJsonSchemaDocHandle** out) {
    StringBuffer sb;
    Writer<StringBuffer> w(sb);
    src->doc.Accept(w);

    Document d;
    d.Parse(sb.GetString());
    if (d.HasParseError()) return false;

    *out = new (std::nothrow) RapidJsonSchemaDocHandle(d);
    return *out != nullptr;
}

RapidJsonSchemaDocHandle* rapidjson_schemadoc_new(const RapidJsonDocumentHandle* doc) {
    RapidJsonSchemaDocHandle* h = nullptr;
    if (!build_schema_doc(doc, &h)) return nullptr;
    return h;
}
void rapidjson_schemadoc_free(RapidJsonSchemaDocHandle* h) { delete h; }

RapidJsonSchemaValidHandle* rapidjson_schemavalidator_new(const RapidJsonSchemaDocHandle* sd) {
    return new (std::nothrow) RapidJsonSchemaValidHandle(sd->sd);
}
void rapidjson_schemavalidator_free(RapidJsonSchemaValidHandle* h) { delete h; }

int rapidjson_schemavalidator_validate(RapidJsonSchemaValidHandle* sv,
                                        const RapidJsonDocumentHandle* doc) {
    sv->sv.Reset();
    return doc->doc.Accept(sv->sv) ? 1 : 0;
}

int rapidjson_schemavalidator_is_valid(const RapidJsonSchemaValidHandle* sv) {
    return sv->sv.IsValid() ? 1 : 0;
}

size_t rapidjson_schemavalidator_get_invalid_schema_pointer(
            const RapidJsonSchemaValidHandle* sv, char* buf, size_t buf_len) {
    StringBuffer sb;
    sv->sv.GetInvalidSchemaPointer().Stringify(sb);
    size_t n = sb.GetSize() < buf_len - 1 ? sb.GetSize() : buf_len - 1;
    memcpy(buf, sb.GetString(), n);
    buf[n] = '\0';
    return n;
}

const char* rapidjson_schemavalidator_get_invalid_schema_keyword(
                const RapidJsonSchemaValidHandle* sv) {
    return sv->sv.GetInvalidSchemaKeyword();
}

size_t rapidjson_schemavalidator_get_invalid_document_pointer(
            const RapidJsonSchemaValidHandle* sv, char* buf, size_t buf_len) {
    StringBuffer sb;
    sv->sv.GetInvalidDocumentPointer().Stringify(sb);
    size_t n = sb.GetSize() < buf_len - 1 ? sb.GetSize() : buf_len - 1;
    memcpy(buf, sb.GetString(), n);
    buf[n] = '\0';
    return n;
}

void rapidjson_schemavalidator_reset(RapidJsonSchemaValidHandle* sv) {
    sv->sv.Reset();
}
