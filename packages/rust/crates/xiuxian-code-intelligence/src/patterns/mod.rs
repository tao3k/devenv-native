//! Language-specific ast-grep pattern constants and grouped pattern tables.

mod catalog;
/// Go symbol extraction patterns.
pub mod go;
/// Java symbol extraction patterns.
pub mod java;
/// JavaScript symbol extraction patterns.
pub mod javascript;
/// Python symbol extraction patterns.
pub mod python;
/// Rust symbol extraction patterns.
pub mod rust;
/// TypeScript symbol extraction patterns.
pub mod typescript;

pub use catalog::{
    ALL_PATTERNS, GO_FN, GO_STRUCT, JAVA_CLASS, JAVA_METHOD, JS_CLASS, JS_FN, PYTHON_ASYNC_DEF,
    PYTHON_CLASS, PYTHON_DEF, RUST_ENUM, RUST_FN, RUST_IMPL, RUST_STRUCT, RUST_TRAIT, TS_INTERFACE,
};
