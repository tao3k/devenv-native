pub(crate) use self::service::analyze_markdown;
pub(crate) use self::service::compile_markdown_nodes;

#[path = "markdown/mod.rs"]
mod markdown;
mod projection;
mod service;

#[cfg(test)]
#[path = "../../../tests/unit/gateway/studio/analysis.rs"]
mod tests;
