//! Coordinates the Studio studio analysis branch and keeps its child modules behind one documented reasoning-tree boundary.

pub(crate) use self::service::analyze_markdown;
pub(crate) use self::service::compile_markdown_nodes;

#[path = "markdown/mod.rs"]
mod markdown;
mod projection;
mod service;

#[cfg(test)]
#[path = "../../../tests/unit/gateway/studio/analysis.rs"]
mod tests;
