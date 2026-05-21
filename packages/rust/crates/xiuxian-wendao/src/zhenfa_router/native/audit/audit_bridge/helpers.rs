//! Compatibility path boundary: this module preserves an established Wendao owner path while the API surface is being narrowed.
use std::collections::HashMap;
use std::hash::BuildHasher;

pub(crate) fn compute_hash(content: &str) -> String {
    blake3::hash(content.as_bytes()).to_hex().to_string()
}

pub(crate) fn resolve_file_content<'a, S: BuildHasher>(
    file_contents: &'a HashMap<String, String, S>,
    doc_path: &str,
) -> Option<&'a String> {
    if let Some(content) = file_contents.get(doc_path) {
        return Some(content);
    }

    let normalized_doc = doc_path.replace('\\', "/");
    let mut best_match: Option<(&String, usize)> = None;

    for (candidate, content) in file_contents {
        let normalized_candidate = candidate.replace('\\', "/");
        let is_match = normalized_candidate == normalized_doc
            || normalized_doc.ends_with(&normalized_candidate)
            || normalized_candidate.ends_with(&normalized_doc);

        if !is_match {
            continue;
        }

        let score = normalized_candidate.len();
        match best_match {
            Some((_, best_score)) if best_score >= score => {}
            _ => best_match = Some((content, score)),
        }
    }

    best_match.map(|(content, _)| content)
}
