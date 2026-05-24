#ifndef RUST_ANY_HPP
#define RUST_ANY_HPP

#include <stddef.h>
#include <functional>
#include "types.hpp"

struct RustAny;

struct RustAnyMethods {
	void (*destroy)(const void*);
	const void* (*clone)(const void*);
	bool (*less)(const RustAny&, const RustAny&);
	bool (*equal)(const RustAny&, const RustAny&);
	size_t (*hash)(const RustAny&);
};

class RustAny {
	const RustAnyMethods* methods = 0;
	const void* val = 0;
public:
	~RustAny() {
		if (val && methods) {
			methods->destroy(val);
		}
	}
    RustAny(): methods(0), val(0) {}
	RustAny(const RustAny& other): methods(other.methods) {
		val = other.val ? other.methods->clone(other.val) : 0;
	}
	RustAny(RustAny&& other): methods(other.methods), val(other.val) {
		other.val = 0;
	}
	RustAny& operator=(const RustAny& other) {
		(*this).~RustAny();
		methods = other.methods;
		val = other.val ? other.methods->clone(other.val) : 0;
		return *this;
	}
	RustAny& operator=(RustAny&& other) {
		(*this).~RustAny();
		methods = other.methods;
		val = other.val;
		other.val = 0;
		return *this;
	}
	bool operator<(const RustAny& other) const {
		return methods->less(*this, other);
	}
	bool operator==(const RustAny& other) const {
		return methods->equal(*this, other);
	}
	size_t hash() const {
		return methods->hash(*this);
	}
};

template<> struct std::hash<RustAny> {
	size_t operator()(const RustAny& val) const {
		return val.hash();
	}
};

template<> struct hicc::is_pod<RustAny> {
	static const bool value = true;
};

#endif

