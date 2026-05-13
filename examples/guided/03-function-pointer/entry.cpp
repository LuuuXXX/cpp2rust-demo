// entry.cpp — guided/03-function-pointer
#include "dispatcher.hpp"

Dispatcher::Dispatcher() {}
Dispatcher::~Dispatcher() {}
void Dispatcher::register_callback(Callback /*cb*/, void* /*user_data*/) {}
void Dispatcher::set_filter(Filter /*filter*/) {}
void Dispatcher::dispatch(int /*event_id*/) const {}
void Dispatcher::reset() {}
int  Dispatcher::callback_count() const { return 0; }
