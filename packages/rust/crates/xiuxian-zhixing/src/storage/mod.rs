//! Storage backend boundary for journals and agendas.

/// Markdown-based file storage logic.
pub mod markdown;
pub use markdown::MarkdownStorage;
