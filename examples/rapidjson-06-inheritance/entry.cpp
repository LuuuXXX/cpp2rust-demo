// entry.cpp — rapidjson-06-inheritance
#include "writer_base.hpp"
#include <cstring>
#include <cstdio>

WriterBase::WriterBase(int indent) : indent_(indent) {}

int WriterBase::GetIndent() const { return indent_; }

PrettyWriterImpl::PrettyWriterImpl(int indent) : WriterBase(indent) {}

void PrettyWriterImpl::WriteString(const char* str, size_t len) {
    std::printf("\"%.*s\"", (int)len, str);
}

void PrettyWriterImpl::WriteInt(int value) {
    std::printf("%d", value);
}

void PrettyWriterImpl::WriteBool(bool value) {
    std::printf("%s", value ? "true" : "false");
}

void PrettyWriterImpl::Flush() {}

void PrettyWriterImpl::SetMaxDepth(int depth) { max_depth_ = depth; }
int  PrettyWriterImpl::GetMaxDepth() const { return max_depth_; }
bool PrettyWriterImpl::IsAtLineStart() const { return at_line_start_; }
