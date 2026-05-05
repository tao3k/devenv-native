use std::path::Path;

pub(super) fn normalize_source_path(project_root: &Path, path: &str) -> String {
    let path = Path::new(path);
    if path.is_absolute() {
        return path.strip_prefix(project_root).map_or_else(
            |_| path.to_string_lossy().replace('\\', "/"),
            |relative| relative.to_string_lossy().replace('\\', "/"),
        );
    }

    path.to_string_lossy().replace('\\', "/")
}
