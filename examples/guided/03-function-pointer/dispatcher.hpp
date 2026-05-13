#pragma once
// dispatcher.hpp — guided/03-function-pointer
//
// C++ class using function-pointer parameters for callbacks.
// Demonstrates the 🔧 guided workflow:
//   • Methods with function-pointer params are SKIPPED (hicc limitation).
//   • The interface report provides a pure-virtual interface skeleton;
//     the user converts the callback to an interface class + @make_proxy.

/// Signature of a raw C-style callback.
typedef void (*Callback)(int event_id, void* user_data);

/// Signature of a filter: returns non-zero to accept the event.
typedef int (*Filter)(int event_id);

class Dispatcher {
public:
    Dispatcher();
    ~Dispatcher();

    // ── Function-pointer params → SKIPPED ──────────────────────────────
    /// Register a callback invoked for every dispatched event.
    void register_callback(Callback cb, void* user_data);

    /// Set an optional filter; only accepted events reach callbacks.
    void set_filter(Filter filter);

    // ── POD methods → auto-extracted ✅ ────────────────────────────────
    /// Dispatch an event to all registered callbacks.
    void dispatch(int event_id) const;

    /// Remove all registered callbacks and reset the filter.
    void reset();

    /// Return the current number of registered callbacks.
    int callback_count() const;
};
