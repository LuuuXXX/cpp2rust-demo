// Safe binding wrapper for bigintegertest_ffi.h
// The file name is intentionally aligned with bigintegertest_ffi.cpp/.h
// to satisfy the 1:1 C++ <-> Rust convention.

#[allow(clippy::all)]
mod ffi {
    include!(concat!(env!("OUT_DIR"), "/ffi_bigintegertest_bindings.rs"));
}

use ffi::*;
use std::ffi::{CStr, CString};

pub struct CxxBigInteger {
    inner: *mut RapidJsonBigIntegerHandle,
}

impl CxxBigInteger {
    pub fn new() -> Self {
        let inner = unsafe { rapidjson_biginteger_new() };
        assert!(!inner.is_null(), "rapidjson_biginteger_new returned null");
        Self { inner }
    }

    pub fn from_decimal_literal(lit: &str) -> Self {
        let mut this = Self::new();
        let c = CString::new(lit).expect("decimal literal must not contain NUL");
        let ok = unsafe { rapidjson_biginteger_from_decimal_literal(this.inner, c.as_ptr()) };
        assert_ne!(ok, 0, "rapidjson_biginteger_from_decimal_literal failed");
        this
    }

    pub fn add_u64(&mut self, value: u64) {
        unsafe { rapidjson_biginteger_add_u64(self.inner, value as u64) };
    }

    pub fn mul_u64(&mut self, value: u64) {
        unsafe { rapidjson_biginteger_mul_u64(self.inner, value as u64) };
    }

    pub fn mul_u32(&mut self, value: u32) {
        unsafe { rapidjson_biginteger_mul_u32(self.inner, value as u32) };
    }

    pub fn shl_bits(&mut self, shift: u32) {
        unsafe { rapidjson_biginteger_shl(self.inner, shift as u32) };
    }

    pub fn compare(&self, other: &CxxBigInteger) -> i32 {
        unsafe { rapidjson_biginteger_compare(self.inner, other.inner) }
    }

    pub fn to_string(&self) -> String {
        // Simple fixed-size buffer; can be adjusted if needed.
        let mut buf = [0i8; 256];
        let ok = unsafe {
            rapidjson_biginteger_to_string(self.inner, buf.as_mut_ptr(), buf.len() as ::std::os::raw::c_ulong)
        };
        if ok == 0 {
            return String::new();
        }
        unsafe { CStr::from_ptr(buf.as_ptr()).to_string_lossy().into_owned() }
    }
}

impl Drop for CxxBigInteger {
    fn drop(&mut self) {
        unsafe { rapidjson_biginteger_free(self.inner) };
    }
}
