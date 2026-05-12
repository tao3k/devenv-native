//! Valkey-backed storage operations for knowledge entries.

mod core;
mod crud;
mod query;

pub use core::KnowledgeStorage;
pub use crud::KnowledgeStorageError;
