// writer_base.hpp — rapidjson-06-inheritance
//
// Two-class hierarchy modelling the RapidJSON Writer extension pattern.
// WriterBase is a pure-virtual interface; PrettyWriterImpl is a concrete
// class that inherits from it and adds additional methods.
//
// Demonstrates: bases[] extraction, class Derived: Base syntax generation,
// mixed-class companion interface for WriterBase.

#pragma once
#include <cstddef>  // size_t

// -------------------------------------------------------------------
// Abstract base class — pure-virtual interface (→ #[interface])
// -------------------------------------------------------------------

/// Abstract writer interface (mirrors rapidjson::BaseReaderHandler pattern).
class WriterBase {
public:
    explicit WriterBase(int indent = 0);
    virtual ~WriterBase() {}

    /// Write a JSON string value.
    virtual void WriteString(const char* str, size_t len) = 0;

    /// Write a JSON integer value.
    virtual void WriteInt(int value) = 0;

    /// Write a JSON boolean value.
    virtual void WriteBool(bool value) = 0;

    /// Flush any buffered output.
    virtual void Flush() = 0;

    /// Get current indentation level (non-virtual accessor).
    int GetIndent() const;

protected:
    int indent_;
};

// -------------------------------------------------------------------
// Concrete derived class — inherits WriterBase, adds pretty-print
// -------------------------------------------------------------------

/// Pretty-printing writer that inherits from WriterBase.
class PrettyWriterImpl : public WriterBase {
public:
    /// Construct with the given indentation step.
    explicit PrettyWriterImpl(int indent = 4);

    // Override all pure-virtual methods.
    void WriteString(const char* str, size_t len) override;
    void WriteInt(int value) override;
    void WriteBool(bool value) override;
    void Flush() override;

    // Additional methods specific to pretty-printing.

    /// Set the maximum nesting depth for pretty output.
    void SetMaxDepth(int depth);

    /// Get the current maximum depth setting.
    int GetMaxDepth() const;

    /// Whether the output is currently at the beginning of a line.
    bool IsAtLineStart() const;

private:
    int max_depth_ = 64;
    bool at_line_start_ = true;
};
