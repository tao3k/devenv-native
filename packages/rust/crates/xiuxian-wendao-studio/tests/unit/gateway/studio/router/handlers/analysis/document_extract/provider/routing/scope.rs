use std::fs;
use std::path::{Path, PathBuf};

#[test]
fn document_extract_scope_forbids_relative_ancestor_visibility() -> Result<(), String> {
    let source_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src/studio/router/handlers/analysis/document_extract");
    let mut files = Vec::new();
    collect_rust_source_files(source_root.as_path(), &mut files)
        .map_err(|error| error.to_string())?;
    let mut violations = Vec::new();
    for path in &files {
        let content = fs::read_to_string(path)
            .map_err(|error| format!("read `{}`: {error}", path.display()))?;
        violations.extend(
            content
                .lines()
                .enumerate()
                .filter_map(|(line_index, line)| {
                    let declaration = line.trim();
                    declaration
                        .contains("pub(in super::")
                        .then(|| format!("{}:{} :: {declaration}", path.display(), line_index + 1))
                }),
        );
    }

    assert!(
        violations.is_empty(),
        "document_extract must not use relative ancestor visibility:\n{}",
        violations.join("\n")
    );
    Ok(())
}

fn collect_rust_source_files(dir: &Path, files: &mut Vec<PathBuf>) -> std::io::Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect_rust_source_files(path.as_path(), files)?;
        } else if path.extension().and_then(|value| value.to_str()) == Some("rs") {
            files.push(path);
        }
    }
    Ok(())
}
