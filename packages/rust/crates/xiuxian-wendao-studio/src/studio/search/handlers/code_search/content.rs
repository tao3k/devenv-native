#[cfg(test)]
use std::collections::HashSet;
#[cfg(test)]
use std::path::Path;

#[cfg(test)]
pub(crate) const CODE_CONTENT_EXTENSIONS: [&str; 4] = ["jl", "julia", "mo", "modelica"];
#[cfg(test)]
pub(crate) const CODE_CONTENT_EXCLUDE_GLOBS: [&str; 7] = [
    ".git/**",
    ".cache/**",
    ".devenv/**",
    ".direnv/**",
    "node_modules/**",
    "target/**",
    "dist/**",
];

#[cfg(test)]
pub(crate) fn parse_content_search_line(line: &str) -> Option<(String, usize, String)> {
    let (path, remainder) = line.rsplit_once(':')?;
    let (path, line_number) = path.rsplit_once(':')?;
    Some((
        path.to_string(),
        line_number.parse().ok()?,
        remainder.to_string(),
    ))
}

#[cfg(test)]
pub(crate) fn truncate_content_search_snippet(value: &str, max_chars: usize) -> String {
    let truncated = value.chars().take(max_chars).collect::<String>();
    if value.chars().count() > max_chars {
        format!("{truncated}...")
    } else {
        truncated
    }
}

#[cfg(test)]
pub(crate) fn path_matches_language_filters(path: &str, filters: &HashSet<String>) -> bool {
    if filters.is_empty() {
        return true;
    }

    let extension = Path::new(path)
        .extension()
        .and_then(|ext| ext.to_str())
        .map(str::to_ascii_lowercase);
    filters.iter().any(|filter| match filter.as_str() {
        "julia" => matches!(extension.as_deref(), Some("jl" | "julia")),
        "modelica" => matches!(extension.as_deref(), Some("mo" | "modelica")),
        other => extension.as_deref() == Some(other),
    })
}

#[cfg(test)]
pub(crate) fn is_supported_code_extension(path: &str) -> bool {
    Path::new(path)
        .extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| {
            CODE_CONTENT_EXTENSIONS
                .iter()
                .any(|candidate| candidate.eq_ignore_ascii_case(ext))
        })
}
