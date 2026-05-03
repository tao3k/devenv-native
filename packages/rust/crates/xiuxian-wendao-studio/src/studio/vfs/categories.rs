use std::path::Path;

use crate::studio::types::VfsCategory;

pub(super) fn guess_category(path: &Path) -> VfsCategory {
    if path.is_dir() {
        return VfsCategory::Folder;
    }
    match path.extension().and_then(|extension| extension.to_str()) {
        Some("md" | "markdown") => VfsCategory::Doc,
        Some("skill") => VfsCategory::Skill,
        _ => VfsCategory::Other,
    }
}
