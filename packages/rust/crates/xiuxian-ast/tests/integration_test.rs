//! Cargo entry point for `xiuxian-ast` integration tests.

#[path = "integration/extract.rs"]
mod extract;
#[path = "integration/item.rs"]
mod item;
#[path = "integration/lang.rs"]
mod lang;
#[path = "integration/python_tree_sitter.rs"]
mod python_tree_sitter;
