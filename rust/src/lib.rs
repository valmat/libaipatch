//! `libaipatch` — static library for applying codex-format patches.
//!
//! Provides a C ABI via [`ffi`] module (aipatch_check, aipatch_apply, etc.)
//! and a Rust-native API via [`engine`] (check, apply).

pub mod engine;
pub mod errors;
pub mod ffi;
pub mod parser;
pub mod paths;
pub(crate) mod seek_sequence;
pub mod write_ops;

#[cfg(test)]
pub(crate) mod test_support;
