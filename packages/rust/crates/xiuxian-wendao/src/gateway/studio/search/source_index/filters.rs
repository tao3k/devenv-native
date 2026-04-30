use std::path::Path;

use walkdir::DirEntry;

pub(crate) fn should_skip_entry(entry: &DirEntry) -> bool {
    if !entry.file_type().is_dir() {
        return false;
    }

    matches!(
        entry.file_name().to_string_lossy().as_ref(),
        ".git"
            | ".cache"
            | ".devenv"
            | ".direnv"
            | ".run"
            | "target"
            | "node_modules"
            | "dist"
            | "coverage"
            | "__pycache__"
    )
}

pub(crate) fn is_markdown_path(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| {
            ext.eq_ignore_ascii_case("md")
                || ext.eq_ignore_ascii_case("markdown")
                || ext.eq_ignore_ascii_case("org")
        })
}
