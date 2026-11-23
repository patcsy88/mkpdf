//! Core document model for mkpdf.
//!
//! This crate is `forbid(unsafe_code)`. It contains no I/O and no global state, so
//! it can be compiled for WebAssembly and driven from any binding.
//!
//! Serialisation to PDF bytes is not implemented yet (see roadmap item MVP-001).

#![forbid(unsafe_code)]

mod error;
mod model;

pub use error::{Error, ErrorKind};
pub use model::{Document, MAX_PAGE_DIMENSION_PT, MAX_PAGES, Page};

/// Crate version as declared in `Cargo.toml`.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
