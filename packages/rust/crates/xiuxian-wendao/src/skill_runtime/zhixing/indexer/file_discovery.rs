use std::path::{Path, PathBuf};
use walkdir::WalkDir;

pub(super) fn collect_markdown_files(dir: &Path) -> Vec<PathBuf> {
    if !dir.exists() || !dir.is_dir() {
        return Vec::new();
    }

    let mut files = Vec::new();
    for entry in WalkDir::new(dir)
        .into_iter()
        .filter_map(std::result::Result::ok)
    {
        if entry.file_type().is_file() {
            let path = entry.path().to_path_buf();
            let is_md = path
                .extension()
                .and_then(|ext| ext.to_str())
                .is_some_and(|ext| {
                    ext.eq_ignore_ascii_case("md") || ext.eq_ignore_ascii_case("markdown")
                });
            if is_md {
                files.push(path);
            }
        }
    }
    files.sort();
    files
}
