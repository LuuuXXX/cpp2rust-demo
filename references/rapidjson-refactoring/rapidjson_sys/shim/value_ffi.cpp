// FFI shim implementation for standalone rapidjson Value operations.

#include "value_ffi.h"

#include <new>
#include <cstring>
#include <stdint.h>

#include "rapidjson/document.h"
#include "rapidjson/allocators.h"

using namespace rapidjson;

typedef GenericValue<UTF8<>, CrtAllocator>    CrtValue;

struct RapidJsonValueHandle {
    CrtValue val;
    CrtAllocator alloc;
    RapidJsonValueHandle() : val(), alloc() {}
};

// ── Value iteration ───────────────────────────────────────────────────────────

int rapidjson_value_get_member_at(const RapidJsonValueHandle* h,
                                   size_t index,
                                   const char** out_key,
                                   size_t* out_key_len) {
    if (!h->val.IsObject()) return 0;
    if (index >= h->val.MemberCount()) return 0;
    auto it = h->val.MemberBegin() + (ptrdiff_t)index;
    *out_key = it->name.GetString();
    *out_key_len = it->name.GetStringLength();
    return 1;
}

// ── Remove member ─────────────────────────────────────────────────────────────

int rapidjson_value_remove_member(RapidJsonValueHandle* h, const char* key) {
    if (!h->val.IsObject() || !h->val.HasMember(key)) return 0;
    h->val.RemoveMember(key);
    return 1;
}

// ── Array operations ──────────────────────────────────────────────────────────

void rapidjson_value_pop_back(RapidJsonValueHandle* h) {
    if (!h->val.IsArray() || h->val.Empty()) return;
    h->val.PopBack();
}

int rapidjson_value_erase_index(RapidJsonValueHandle* h, size_t index) {
    if (!h->val.IsArray() || index >= h->val.Size()) return 0;
    h->val.Erase(h->val.Begin() + (ptrdiff_t)index);
    return 1;
}

void rapidjson_value_reserve(RapidJsonValueHandle* h, size_t cap) {
    if (!h->val.IsArray()) return;
    h->val.Reserve((SizeType)cap, h->alloc);
}

// ── Numeric conversion ────────────────────────────────────────────────────────

int rapidjson_value_is_lossless_double(const RapidJsonValueHandle* h) {
    return h->val.IsLosslessDouble() ? 1 : 0;
}
int rapidjson_value_is_lossless_float(const RapidJsonValueHandle* h) {
    return h->val.IsLosslessFloat() ? 1 : 0;
}
float rapidjson_value_get_float(const RapidJsonValueHandle* h) {
    return h->val.GetFloat();
}

// ── Clear ─────────────────────────────────────────────────────────────────────

void rapidjson_value_clear_members(RapidJsonValueHandle* h) {
    if (h->val.IsObject()) h->val.RemoveAllMembers();
}
void rapidjson_value_clear_elements(RapidJsonValueHandle* h) {
    if (h->val.IsArray()) h->val.Clear();
}
