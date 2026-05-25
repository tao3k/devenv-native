pub(super) fn path_has_extension(path: &str, extension: &str) -> bool {
    std::path::Path::new(path)
        .extension()
        .is_some_and(|candidate| candidate.eq_ignore_ascii_case(extension))
}

pub(super) fn path_has_double_extension(path: &str, first: &str, second: &str) -> bool {
    let Some(stem) = std::path::Path::new(path)
        .file_stem()
        .and_then(|stem| stem.to_str())
    else {
        return false;
    };
    path_has_extension(path, second) && path_has_extension(stem, first)
}

pub(super) fn is_markdown_path(path: &str) -> bool {
    path_has_extension(path, "md")
}

pub(super) fn is_package_source_path(path: &str) -> bool {
    path.starts_with("packages/") && !is_test_path(path)
}

pub(super) fn is_test_path(path: &str) -> bool {
    path.contains("/tests/")
        || path.starts_with("tests/")
        || path.contains("/test/")
        || path.ends_with("_test.rs")
        || path_has_double_extension(path, "test", "ts")
        || path_has_double_extension(path, "spec", "ts")
}
