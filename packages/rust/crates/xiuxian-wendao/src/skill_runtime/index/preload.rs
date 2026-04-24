use std::path::Path;

use walkdir::WalkDir;

use super::SkillInventory;

pub(super) fn preload_reference_dir(
    index: &mut SkillInventory,
    semantic_name: &str,
    references_dir: &Path,
) {
    if !references_dir.exists() || !references_dir.is_dir() {
        return;
    }

    for entry in WalkDir::new(references_dir)
        .into_iter()
        .filter_map(Result::ok)
    {
        if !entry.file_type().is_file() {
            continue;
        }
        let path = entry.into_path();
        let Ok(relative) = path.strip_prefix(references_dir) else {
            continue;
        };
        let entity_name = relative.to_string_lossy().replace('\\', "/");
        let key = semantic_resource_uri_key(semantic_name, &entity_name);
        index.paths_by_uri.insert(key, path);
    }
}

pub(super) fn semantic_resource_uri_key(semantic_name: &str, entity_name: &str) -> String {
    format!(
        "wendao://skills/{}/references/{}",
        semantic_name.trim().to_ascii_lowercase(),
        entity_name.trim_start_matches('/')
    )
}
