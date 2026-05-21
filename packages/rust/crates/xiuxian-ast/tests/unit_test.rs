//! Cargo entry point for `xiuxian-ast` unit tests.

#[path = "unit/chunk.rs"]
mod chunk;
#[path = "unit/extract.rs"]
mod extract;
#[path = "unit/lang.rs"]
mod lang;
#[path = "unit/lib_policy.rs"]
mod lib_policy;
#[path = "unit/parser_registry.rs"]
mod parser_registry;
#[path = "unit/python.rs"]
mod python;
#[path = "unit/python_tree_sitter.rs"]
mod python_tree_sitter;
#[path = "unit/scan.rs"]
mod scan;
#[path = "unit/scan_decorator.rs"]
mod scan_decorator;
#[path = "unit/security.rs"]
mod security;
