use std::path::{Path, PathBuf};

use walkdir::WalkDir;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DiscoveredMarkdownFile {
    pub(crate) surface: String,
    pub(crate) absolute_path: PathBuf,
    pub(crate) relative_path: String,
}

pub(crate) fn discover_bounded_work_markdown_files(
    root: &Path,
) -> Result<Vec<DiscoveredMarkdownFile>, String> {
    let mut files = Vec::new();
    for surface in ["blueprint", "plan"] {
        let surface_root = root.join(surface);
        if !surface_root.exists() {
            continue;
        }
        for entry in WalkDir::new(&surface_root) {
            let entry = entry.map_err(|error| {
                format!(
                    "failed to scan bounded work markdown files under `{}`: {error}",
                    surface_root.display()
                )
            })?;
            if !entry.file_type().is_file() || !is_supported_note(entry.path()) {
                continue;
            }
            let relative_path = normalize_relative_path(root, entry.path())?;
            files.push(DiscoveredMarkdownFile {
                surface: surface.to_string(),
                absolute_path: entry.path().to_path_buf(),
                relative_path,
            });
        }
    }
    files.sort_by(|left, right| {
        left.surface
            .cmp(&right.surface)
            .then_with(|| left.relative_path.cmp(&right.relative_path))
    });
    Ok(files)
}

fn normalize_relative_path(root: &Path, path: &Path) -> Result<String, String> {
    let relative = path.strip_prefix(root).map_err(|error| {
        format!(
            "bounded work markdown file `{}` is outside root `{}`: {error}",
            path.display(),
            root.display()
        )
    })?;
    Ok(relative.to_string_lossy().replace('\\', "/"))
}

fn is_supported_note(path: &Path) -> bool {
    path.extension()
        .and_then(|value| value.to_str())
        .is_some_and(|extension| {
            let lower = extension.to_lowercase();
            matches!(lower.as_str(), "md" | "markdown" | "mdx")
        })
}
