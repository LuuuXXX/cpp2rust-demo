#pragma once
// event_emitter.hpp — guided/02-std-function
//
// C++ class using std::function<> for callbacks.
// Demonstrates the 🔧 guided workflow:
//   • Methods with std::function params are SKIPPED (hicc limitation).
//   • cpp2rust-demo generates a pure-virtual interface skeleton + @make_proxy
//     suggestion in the interface report; the user fills in the interface
//     class and implements the Rust trait.

#include <functional>

class EventEmitter {
public:
    EventEmitter();
    ~EventEmitter();

    // ── std::function params → SKIPPED ─────────────────────────────────
    /// Register a callback for all events.
    void on_event(std::function<void(int)> handler);

    /// Register a callback that receives a payload string.
    void on_message(std::function<void(int, const char*)> handler);

    // ── POD methods → auto-extracted ✅ ────────────────────────────────
    /// Emit an event with the given id.
    void emit(int event_id) const;

    /// Return the number of registered handlers.
    int handler_count() const;
};
