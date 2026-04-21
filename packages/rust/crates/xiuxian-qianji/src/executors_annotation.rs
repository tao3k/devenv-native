//! Context annotation mechanism using Qianhuan.

#[path = "executors_annotation_context.rs"]
mod context;
#[path = "executors/annotation/persona_markdown.rs"]
mod persona_markdown;

pub use context::ContextAnnotator;
