//! Core infrastructure crate for rapidjson-rs.
//!
//! This crate provides the foundational runtime support for
//! higher-level features such as DOM, SAX, Pointer and Schema.

#![forbid(unsafe_code)]

pub mod error;
pub mod internal;
pub mod memory;
pub mod stream;
