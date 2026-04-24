use regex::Regex;
use serde_yaml::Value;
use std::sync::LazyLock;

fn compile_regex(pattern: &str) -> Regex {
    match Regex::new(pattern) {
        Ok(regex) => regex,
        Err(_compile_err) => match Regex::new(r"$^") {
            Ok(fallback) => fallback,
            Err(fallback_err) => panic!("hardcoded fallback regex must compile: {fallback_err}"),
        },
    }
}

static FRONTMATTER_REGEX: LazyLock<Regex> =
    LazyLock::new(|| compile_regex(r"(?s)\A---\s*\n(.*?)\n(?:---|\.\.\.)\s*\n?"));

/// Borrowed raw frontmatter slice plus the remaining Markdown body.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RawFrontmatter<'a> {
    /// Raw YAML content without the surrounding fences.
    pub yaml: &'a str,
    /// Remaining Markdown body after the closing fence.
    pub body: &'a str,
}

/// Split one Markdown document into an optional borrowed raw YAML frontmatter
/// slice and the remaining body content.
#[must_use]
pub fn split_frontmatter_raw(content: &str) -> Option<RawFrontmatter<'_>> {
    let caps = FRONTMATTER_REGEX.captures(content)?;
    let yaml = caps.get(1)?.as_str();
    let body = caps.get(0).map_or(content, |m| &content[m.end()..]);
    Some(RawFrontmatter { yaml, body })
}

/// Split one Markdown document into an optional parsed YAML frontmatter value
/// and the remaining body content.
#[must_use]
pub fn split_frontmatter(content: &str) -> (Option<Value>, &str) {
    let Some(parts) = split_frontmatter_raw(content) else {
        return (None, content);
    };
    let parsed = serde_yaml::from_str::<Value>(parts.yaml).ok();
    (parsed, parts.body)
}
