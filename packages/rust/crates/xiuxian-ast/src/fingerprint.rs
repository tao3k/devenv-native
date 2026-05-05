//! Stable structural fingerprints for supported source languages.

use crate::{Lang, extract_skeleton};

/// Returns whether the language has a stable generic structural fingerprint
/// owner in `xiuxian-ast`.
#[must_use]
pub fn supports_semantic_fingerprint(lang: Lang) -> bool {
    matches!(
        lang,
        Lang::Python
            | Lang::Rust
            | Lang::JavaScript
            | Lang::TypeScript
            | Lang::Bash
            | Lang::Go
            | Lang::Java
            | Lang::C
            | Lang::Cpp
            | Lang::CSharp
            | Lang::Ruby
            | Lang::Swift
            | Lang::Kotlin
            | Lang::Lua
            | Lang::Php
            | Lang::Toml
    )
}

/// Builds a stable structural semantic fingerprint for one supported source
/// file.
#[must_use]
pub fn semantic_fingerprint(content: &str, lang: Lang) -> Option<String> {
    if !supports_semantic_fingerprint(lang) {
        return None;
    }

    let skeleton_source = extract_skeleton(content, lang);
    let skeleton = normalize_skeleton(skeleton_source.as_str());
    if skeleton.is_empty() {
        return None;
    }

    let mut hasher = blake3::Hasher::new();
    hasher.update(b"xiuxian_ast.semantic_fingerprint.v1\0");
    hasher.update(lang.as_str().as_bytes());
    hasher.update(b"\0");
    hasher.update(skeleton.as_bytes());
    Some(hasher.finalize().to_hex().to_string())
}

fn normalize_skeleton(skeleton: &str) -> String {
    skeleton
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}
