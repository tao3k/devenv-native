use std::path::Path;

use globset::{Glob, GlobSet, GlobSetBuilder};

pub(crate) fn build_scope_matcher(patterns: &[String]) -> Option<GlobSet> {
    let mut builder = GlobSetBuilder::new();
    let mut has_pattern = false;
    for pattern in patterns {
        let Ok(glob) = Glob::new(pattern) else {
            continue;
        };
        builder.add(glob);
        has_pattern = true;
    }

    if !has_pattern {
        return None;
    }

    builder.build().ok()
}

pub(crate) fn normalize_match_path(project_root: &Path, path: &str) -> String {
    let path = Path::new(path);
    if path.is_absolute() {
        return path
            .strip_prefix(project_root)
            .map_or_else(|_| normalize_path(path), normalize_path);
    }

    normalize_path(path)
}

pub(crate) fn normalize_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

pub(crate) fn definition_match_score(
    base_score: f64,
    candidate_path: &str,
    preferred_parent: Option<&str>,
) -> f64 {
    base_score + definition_scope_bonus(candidate_path, preferred_parent)
}

pub(crate) fn definition_scope_bonus(candidate_path: &str, preferred_parent: Option<&str>) -> f64 {
    let Some(preferred_parent) = preferred_parent else {
        return 0.0;
    };
    let candidate_parent = Path::new(candidate_path)
        .parent()
        .map(normalize_path)
        .unwrap_or_default();
    if candidate_parent == preferred_parent {
        0.15
    } else {
        0.0
    }
}
