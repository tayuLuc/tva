#![forbid(unsafe_code)]

//! C FFI shared library for embedding tva-core.
//! Re-exports core's C API with cdylib crate-type.

pub use tva_core::*;
