// entry.cpp — guided/02-std-function
#include "event_emitter.hpp"

EventEmitter::EventEmitter() {}
EventEmitter::~EventEmitter() {}
void EventEmitter::on_event(std::function<void(int)> /*handler*/) {}
void EventEmitter::on_message(std::function<void(int, const char*)> /*handler*/) {}
void EventEmitter::emit(int /*event_id*/) const {}
int  EventEmitter::handler_count() const { return 0; }
