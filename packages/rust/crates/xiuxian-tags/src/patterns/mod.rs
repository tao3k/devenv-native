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
    ALL_PATTERNS, GO_FN, GO_STRUCT, JAVA_CLASS, JAVA_METHOD, JS_CLASS, JS_CLASS_PATTERN, JS_FN,
    JS_FN_PATTERN, PYTHON_ASYNC_DEF, PYTHON_ASYNC_DEF_PATTERN, PYTHON_CLASS, PYTHON_CLASS_PATTERN,
    PYTHON_DEF, PYTHON_DEF_PATTERN, RUST_ENUM, RUST_ENUM_PATTERN, RUST_FN, RUST_FN_PATTERN,
    RUST_IMPL, RUST_IMPL_PATTERN, RUST_STRUCT, RUST_STRUCT_PATTERN, RUST_TRAIT, RUST_TRAIT_PATTERN,
    TS_INTERFACE, TS_INTERFACE_PATTERN,
};
