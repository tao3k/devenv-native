use std::path::Path;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) enum FingerprintMode {
    PathOnly,
    Contents,
}

pub(crate) fn analysis_fingerprint_mode(
    relative_path: &str,
    plugin_ids: &[String],
) -> Option<FingerprintMode> {
    if is_git_internal_path(relative_path) {
        return None;
    }
    if plugin_ids.is_empty()
        || plugin_ids
            .iter()
            .any(|plugin_id| !matches!(plugin_id.as_str(), "julia-code-parser" | "modelica"))
    {
        return Some(FingerprintMode::Contents);
    }

    plugin_ids.iter().fold(None, |best, plugin_id| {
        let candidate = match plugin_id.as_str() {
            "julia-code-parser" => julia_fingerprint_mode(relative_path),
            "modelica" => modelica_fingerprint_mode(relative_path),
            _ => None,
        };
        strongest_mode(best, candidate)
    })
}

#[cfg(feature = "search-runtime")]
pub(crate) fn change_affects_analysis_identity(
    relative_path: &str,
    plugin_ids: &[String],
    changed_contents: bool,
) -> bool {
    match analysis_fingerprint_mode(relative_path, plugin_ids) {
        Some(FingerprintMode::Contents) => true,
        Some(FingerprintMode::PathOnly) => !changed_contents,
        None => false,
    }
}

fn strongest_mode(
    left: Option<FingerprintMode>,
    right: Option<FingerprintMode>,
) -> Option<FingerprintMode> {
    match (left, right) {
        (Some(left), Some(right)) => Some(left.max(right)),
        (Some(left), None) => Some(left),
        (None, Some(right)) => Some(right),
        (None, None) => None,
    }
}

fn julia_fingerprint_mode(relative_path: &str) -> Option<FingerprintMode> {
    if relative_path == "Project.toml"
        || is_under_directory_with_extension(relative_path, "src", "jl")
    {
        return Some(FingerprintMode::Contents);
    }
    if is_root_readme(relative_path)
        || is_under_directory_with_extension(relative_path, "docs", "md")
        || is_under_directory_with_extension(relative_path, "examples", "jl")
        || is_under_directory_with_extension(relative_path, "test", "jl")
    {
        return Some(FingerprintMode::PathOnly);
    }
    None
}

fn modelica_fingerprint_mode(relative_path: &str) -> Option<FingerprintMode> {
    if contains_hidden_component(relative_path) {
        return None;
    }
    if has_extension(relative_path, "mo")
        || relative_path == "package.order"
        || relative_path.ends_with("/package.order")
    {
        return Some(FingerprintMode::Contents);
    }
    if is_any_readme(relative_path) || is_supported_users_guide_text_doc(relative_path) {
        return Some(FingerprintMode::PathOnly);
    }
    None
}

fn is_root_readme(relative_path: &str) -> bool {
    !relative_path.contains('/') && is_readme_file_name(relative_path)
}

fn is_any_readme(relative_path: &str) -> bool {
    Path::new(relative_path)
        .file_name()
        .and_then(std::ffi::OsStr::to_str)
        .is_some_and(is_readme_file_name)
}

fn is_readme_file_name(name: &str) -> bool {
    name.to_ascii_uppercase().starts_with("README")
}

fn is_under_directory_with_extension(
    relative_path: &str,
    directory: &str,
    extension: &str,
) -> bool {
    if !relative_path.starts_with(directory) {
        return false;
    }
    let Some(remainder) = relative_path.strip_prefix(directory) else {
        return false;
    };
    if !remainder.starts_with('/') {
        return false;
    }
    Path::new(relative_path)
        .extension()
        .and_then(std::ffi::OsStr::to_str)
        .is_some_and(|current| current == extension)
}

fn is_supported_users_guide_text_doc(relative_path: &str) -> bool {
    let components = relative_path
        .split('/')
        .filter(|component| !component.is_empty());
    let has_users_guide = components
        .clone()
        .any(|component| component == "UsersGuide");
    if !has_users_guide {
        return false;
    }
    matches!(
        Path::new(relative_path)
            .extension()
            .and_then(std::ffi::OsStr::to_str),
        Some("md" | "rst" | "qmd")
    )
}

fn has_extension(relative_path: &str, extension: &str) -> bool {
    Path::new(relative_path)
        .extension()
        .and_then(std::ffi::OsStr::to_str)
        .is_some_and(|current| current.eq_ignore_ascii_case(extension))
}

fn contains_hidden_component(relative_path: &str) -> bool {
    relative_path
        .split('/')
        .filter(|component| !component.is_empty())
        .any(|component| component.starts_with('.'))
}

fn is_git_internal_path(relative_path: &str) -> bool {
    relative_path
        .split('/')
        .next()
        .is_some_and(|component| component == ".git")
}
